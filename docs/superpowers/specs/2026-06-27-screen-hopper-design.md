# screen-hopper 设计文档

- 日期：2026-06-27
- 仓库：https://github.com/CodeWhatD/screen-hopper
- 状态：设计已确认，待写实现计划

## 1. 背景与问题

公司电脑接了 **3 块显示器**，但用 toDesk（免费版）远程连接时**只能看到一块屏**（主显示器那块）。toDesk 的多屏切换是**收费功能**。本项目做一个免费的 DIY 替代：一个跑在被控电脑（公司电脑，Windows）上的可视化小工具，让你远程一键切换"当前看哪块屏"。

## 2. 核心原理

toDesk 免费版只串流**主显示器**的画面（已实测验证：在被控电脑把某块屏「设为主显示器」，toDesk 画面会自动跟着切过去）。

因此工具不需要做任何画面捕获/串流，只需：**修改被控电脑的主显示器**。你通过 toDesk 远程点一下工具上的按钮 → 工具把对应屏设为主屏 → toDesk 画面随之跳到那块屏。

> 该原理对所有"只传一块屏"的远程软件（AnyDesk、RustDesk、向日葵等）通用，故工具命名与实现都不绑定 toDesk。

## 3. 技术栈

- **Tauri 2.x**：Rust 后端 + TypeScript/HTML 前端。
- **前端**：原生 vanilla TS + HTML/CSS（仅几个按钮，不引入框架）。
- **后端原生调用**：`windows` crate 调用 Win32 显示 API。
- **产物**：几 MB 的 portable `.exe`，依赖 Windows 自带的 **WebView2 运行时**（Win10/11 默认预装）。
- **构建**：`cargo tauri build`，无需在被控电脑安装任何运行时。

选择 Tauri 而非 Electron 的原因：体积小（几 MB vs 70–100MB），更适合"拷到公司电脑直接跑"。代价是切主屏的原生逻辑用 Rust 写，可接受。

## 4. 界面设计

- 一个**无边框、置顶（always-on-top）、可拖动**的小横条。
- 每检测到一块显示器，渲染一个按钮，显示如 `屏1 1920×1080`。
- **当前主屏**对应的按钮**高亮**。
- 右侧一个 `🔄` 重新检测按钮。
- 窗口小巧，不占任务栏（`skipTaskbar`），尽量不挡远程操作。

## 5. 核心流程与接口

后端通过 Tauri command 暴露给前端两个命令：

### 5.1 `list_monitors() -> Vec<Monitor>`

用 Win32 `EnumDisplayDevices` + `EnumDisplaySettings`（或 `EnumDisplayMonitors` / `GetMonitorInfo`）枚举所有显示器，返回：

```ts
interface Monitor {
  index: number;      // 稳定序号，用于切换
  device_name: string;// 如 \\.\DISPLAY1，调 API 用
  label: string;      // 给用户看的名字，如 "屏1"
  width: number;
  height: number;
  x: number;          // 当前虚拟桌面坐标
  y: number;
  is_primary: boolean;
}
```

### 5.2 `set_primary(index: number) -> Result<(), String>`

把指定显示器设为主屏。Windows 设主屏的标准三步法（用 `ChangeDisplaySettingsEx`）：

1. 给**目标屏**设位置为 `(0,0)`，flag = `CDS_SET_PRIMARY | CDS_UPDATEREGISTRY | CDS_NORESET`。
2. 给**其余每块屏**按目标屏原偏移量平移（`new_x = old_x - target_old_x`，`new_y = old_y - target_old_y`），flag = `CDS_UPDATEREGISTRY | CDS_NORESET`。
3. 最后一次**空调用** `ChangeDisplaySettingsEx(NULL, NULL, NULL, 0, NULL)` 让全部改动统一生效。

返回成功/失败（失败时把错误码转成可读字符串）。

### 5.3 切换后的"窗口跟随"（前端逻辑）

`set_primary` 成功后：
1. 前端调用 Tauri 窗口 API `setPosition`，把工具窗口移到新主屏（新主屏现在位于 `(0,0)`，放左上角附近，如 `(40, 40)`）。
2. 重新调用 `list_monitors()` 刷新按钮高亮。

这样工具窗口永远漂在"你当前能看到的那块屏"上，随时能再点。这是必须实现的关键行为——否则切屏后工具会从 toDesk 画面里消失。

## 6. 边界情况

- **只有一块屏**：只渲染一个按钮（或提示"仅检测到一块显示器"）。
- **点击的就是当前主屏**：直接忽略，不调 API。
- **各屏分辨率/缩放不同**：用每块屏自己的实际分辨率参与坐标重排，不假设等宽。
- **`set_primary` 失败**：界面给出提示，不崩溃。
- **显示器热插拔**：靠 `🔄` 重新检测（v1 不做自动监听）。

## 7. v1 范围（YAGNI）

**做**：检测显示器 → 点击切换主屏 → 窗口自动跟随 → 当前主屏高亮 → 手动重新检测。

**v1 不做**（以后想加再加）：
- 全局快捷键切换
- "拼接显示全部屏"
- 记忆/恢复显示器布局
- 显示器热插拔自动监听
- 开机自启

## 8. 分发方式

`cargo tauri build` 产出 portable `.exe` → 拷到公司电脑双击运行。被控电脑只需有 WebView2（Win10/11 默认有）。

## 9. 已知前置条件（实现前需处理）

- 本机**未安装 Rust**（rustc/cargo），需先用 rustup 安装 + 安装 Tauri CLI。Node v24 / npm 已就绪。
- 本机命令行连 GitHub 不稳定（https 被重置）。本地开发不受影响，push 需等网络通或挂代理。
