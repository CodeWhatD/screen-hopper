# screen-hopper 代码导读（Rust 新手向）

> 这份文档带你从零看懂整个项目。假设你**写过前端 / 其他语言，但没碰过 Rust**。
> 配合源码一起看，每段都标了对应文件和行号。

---

## 0. 一句话：这个项目是干嘛的

公司电脑接了 3 块显示器，但 toDesk 免费版只能串流**主显示器**那一块。
本工具用一个小横条，点一下就把某块屏切成"主显示器"，远程画面随之切过去——
等于免费实现了 toDesk 的付费多屏切换功能。

核心原理：调用 Windows 系统 API `ChangeDisplaySettingsEx` 改主屏。就这一招。

---

## 1. 整体架构：Tauri = 网页当界面 + Rust 当后端

这是个 **Tauri** 应用。可以这样理解它：

```
┌─────────────────────────────────────────────┐
│  一个程序窗口（screen-hopper.exe）            │
│                                               │
│  ┌─────────────────┐   invoke()   ┌────────┐ │
│  │  前端 (WebView)  │ ───────────► │  Rust  │ │
│  │  HTML+CSS+TS     │ ◄─────────── │  后端  │ │
│  │  画那个小横条     │   返回数据    │ 调系统 │ │
│  └─────────────────┘              └────────┘ │
└─────────────────────────────────────────────┘
```

- **前端**：就是网页（`index.html` + `src/`）。负责画界面、响应点击。跑在系统自带的
  WebView2 里（Win10/11 都预装了，所以 exe 不用打包浏览器，才这么小）。
- **后端**：Rust 代码（`src-tauri/`）。负责干"网页干不了的事"——调用 Windows 系统 API
  去枚举显示器、切主屏。
- **桥梁**：前端用 `invoke("命令名", 参数)` 调用后端；后端用 `#[tauri::command]` 把
  函数暴露成可被调用的"命令"。这是你要理解的**最关键的一条连接线**。

> 类比：前端像浏览器页面，后端像一个本地的 API 服务器，`invoke` 像 `fetch`。
> 只不过它们打包在同一个 exe 里，调用是进程内的，没有网络。

---

## 2. 目录结构逐个看

```
screen-hopper/
├── index.html              # 前端入口页面（就一个空横条 div）
├── package.json            # 前端依赖清单（npm）
├── vite.config.ts          # 前端构建工具 Vite 的配置
├── tsconfig.json           # TypeScript 配置
│
├── src/                    # ===== 前端源码 =====
│   ├── main.ts             # ★ 前端全部逻辑（画横条、点击切屏、查更新）
│   └── styles.css          # 横条的样式（深色 Catppuccin 风）
│
├── src-tauri/              # ===== Rust 后端 =====
│   ├── Cargo.toml          # ★ Rust 依赖清单（相当于后端的 package.json）
│   ├── Cargo.lock          # 锁定依赖精确版本（自动生成，别手改）
│   ├── build.rs            # 构建脚本（Tauri 自动生成，基本不用动）
│   ├── tauri.conf.json     # ★ 应用配置：窗口大小、标题、版本号、权限
│   ├── capabilities/
│   │   └── default.json    # ★ 权限白名单：前端允许调哪些窗口 API
│   ├── icons/              # 应用图标（各种尺寸，Tauri 模板自带）
│   └── src/                # ----- Rust 代码 -----
│       ├── main.rs         # 程序入口（6 行，只调 lib.rs 的 run）
│       ├── lib.rs          # ★ 注册所有命令、启动 Tauri
│       ├── layout.rs       # ★ 纯计算：算每块屏切换后的新坐标（带单元测试）
│       └── display.rs      # ★ 调 Win32 API：枚举显示器 + 切主屏
│
├── CHANGELOG.md            # 版本变更记录
├── README.md               # 项目说明 + 用法
└── docs/CODE-GUIDE.md      # 你正在看的这份
```

打 ★ 的 7 个文件是项目的全部核心，加起来才 ~380 行。其余都是脚手架自动生成的配置。

**建议的阅读顺序**（从易到难）：
`layout.rs`（纯逻辑最好懂）→ `display.rs`（系统调用）→ `lib.rs`（怎么连起来）→ `main.ts`（前端）。

---

## 3. Rust 新手：看这个项目你需要知道的概念

不用系统学 Rust，先掌握这几个够你读懂代码的点。

### 3.1 函数与可见性
```rust
pub fn enumerate() -> Vec<Monitor> { ... }
```
- `fn` 定义函数；`pub` 表示"公开"（别的文件能用），不写 `pub` 就是本文件私有。
- `-> Vec<Monitor>` 是返回类型：返回一个 `Monitor` 的动态数组（`Vec` = vector = 列表）。
- **函数最后一行不写 `return`、不加分号，就是返回值**。这是 Rust 特色。

### 3.2 结构体 struct（相当于对象/类）
```rust
pub struct Monitor {
    pub index: usize,
    pub label: String,
    pub width: i32,
    ...
}
```
- `struct` 像 TS 的 `interface` + 数据。字段也要 `pub` 才能外部访问。
- 类型：`usize` 无符号整数（常用于下标/数量）、`i32` 32 位有符号整数、`String` 字符串、
  `bool` 布尔。

### 3.3 `#[derive(...)]` 和 `#[tauri::command]` —— 方括号里的"注解"
```rust
#[derive(serde::Serialize, Clone, Debug)]
pub struct Monitor { ... }
```
- `#[...]` 是**属性/宏**，给下面的东西自动加能力，类似装饰器。
- `Serialize` 让这个结构体能被自动转成 JSON 发给前端（关键！前端才能收到显示器列表）。
- `Clone` 让它能被复制，`Debug` 让它能被打印调试。
- `#[tauri::command]` 把一个普通函数标记成"前端可以 invoke 的命令"。

### 3.4 `Result` 和错误处理 —— Rust 没有异常
```rust
pub fn set_primary(target: usize) -> Result<(), String> { ... }
```
- `Result<T, E>` 表示"要么成功返回 T，要么失败返回错误 E"。
- `Result<(), String>`：成功时返回 `()`（空，相当于 void），失败时返回 `String` 错误信息。
- 成功用 `Ok(值)` 包装，失败用 `Err(信息)` 包装。
- `?` 问号运算符：`compute_layout(&geom, target)?` 意思是"如果这步出错，立刻把错误
  return 出去；没出错就取出里面的值继续"。省去手写 if 判断。

### 3.5 借用 `&` —— Rust 最有名的概念，这里只需浅尝
```rust
pub fn compute_layout(monitors: &[MonitorGeom], target: usize)
```
- `&` 表示"借用/引用"：我只是借来读，不拿走所有权。`&[MonitorGeom]` = 借一个切片（数组片段）来读。
- 现在你只要知道：**`&` 大多表示"传引用，不复制、不夺走"**。深入的所有权规则以后再学。

### 3.6 `unsafe` —— 调系统 API 时的"我知道我在干嘛"
```rust
let ok = unsafe { EnumDisplayDevicesW(...) };
```
- 调 Windows 原生 API（裸指针、可能崩）时，Rust 要求你用 `unsafe {}` 包起来，
  表示"编译器不保证这段安全，我自己负责"。本项目的 `unsafe` 都集中在 `display.rs`，
  全是调 Win32 函数，属于正常用法。

### 3.7 模块 `mod`
```rust
mod layout;   // 引入同目录的 layout.rs
mod display;
```
- `mod xxx;` 把 `xxx.rs` 文件作为模块挂进来。之后用 `display::enumerate()`、
  `layout::compute_layout()`（`::` 是路径分隔符，像 TS 的 `.`）。

掌握以上 7 点，本项目的 Rust 代码你就能读懂 90%。

---

## 4. 三大功能怎么实现的（顺着数据流看）

### 功能 A：列出所有显示器（横条上的"屏1/屏2/屏3"按钮）

**调用链**：前端启动 → `invoke("list_monitors")` → Rust `display::enumerate()` → 返回列表 → 前端画按钮。

1. **前端** `src/main.ts` 的 `render()`：
   ```ts
   const monitors = await invoke<Monitor[]>("list_monitors");
   // 拿到列表后，给每块屏建一个 <button>
   ```
2. **后端** `src-tauri/src/lib.rs`：
   ```rust
   #[tauri::command]
   fn list_monitors() -> Vec<display::Monitor> { display::enumerate() }
   ```
3. **真正干活** `src-tauri/src/display.rs` 的 `enumerate()`：用 Win32 的
   `EnumDisplayDevicesW` 循环枚举每个显示设备，再用 `EnumDisplaySettingsW` 读它的
   分辨率和坐标，组装成 `Monitor` 列表。因为 `Monitor` 加了 `#[derive(Serialize)]`，
   Tauri 自动把它转成 JSON 交给前端。

### 功能 B：切换主显示器（点按钮的那一下）

**调用链**：点按钮 → `invoke("set_primary", {index})` → Rust 改主屏 → 前端把窗口挪到新主屏。

1. **纯计算** `src-tauri/src/layout.rs` 的 `compute_layout()`：
   这是全项目最该先读的函数。它不碰系统，只做数学——把你选中的那块屏平移到原点 (0,0)，
   其他屏跟着平移相同距离，保持相对位置不变。
   ```rust
   // 选中屏的坐标是 (dx, dy)，所有屏都减去 (dx, dy)，选中屏就落到 (0,0)
   x: m.x - dx,
   y: m.y - dy,
   is_primary: m.index == target,
   ```
   **为什么单独抽出来**：纯逻辑能写单元测试（文件底部 `mod tests` 有 4 个测试），
   不依赖真实显示器就能验证算得对不对。这是个很好的工程习惯——**把"算"和"做"分开**。

2. **真正切屏** `src-tauri/src/display.rs` 的 `set_primary()`：
   拿 `compute_layout` 算出的新坐标，对每块屏调 `ChangeDisplaySettingsExW`，
   选中的那块加 `CDS_SET_PRIMARY` 标志。用的是 Windows 标准的"三步提交"：
   先逐块暂存位置（`CDS_NORESET`），最后一个 NULL 调用一次性生效。

3. **窗口跟随**（功能 B 的关键体验）`src/main.ts` 的 `switchTo()`：
   切完主屏后，新主屏在坐标 (0,0)，但工具窗口可能还停在旧屏上——那旧屏现在 toDesk
   看不到了，窗口就"消失"了。所以切换后要把窗口挪回 (0,0) 附近：
   ```ts
   for (let i = 0; i < 5; i++) {
     await new Promise((r) => setTimeout(r, 150));
     await w.setPosition(new PhysicalPosition(40, 40));
   }
   ```
   **为什么循环 5 次**：Windows 重排坐标系是**异步**的，立刻挪一次可能挪到旧坐标系里
   又被带走（这正是 v0.1.1 修的 bug）。所以在 ~750ms 内反复挪几次，等坐标系稳定后
   总有一次落在新主屏上。这段注释在代码里也写得很清楚，重点体会。

### 功能 C：检查更新（v0.2.0 加的）

**调用链**：前端启动 → 读自己版本 → fetch GitHub API → 比版本 → 有新版就弹提示行。

1. 读自己的版本，`src-tauri/src/lib.rs`：
   ```rust
   #[tauri::command]
   fn app_version() -> String { env!("CARGO_PKG_VERSION").to_string() }
   ```
   `env!(...)` 是编译期宏，把 `Cargo.toml` 里的 `version` 直接"焊进"程序。
2. `src/main.ts` 的 `checkUpdate()`：直接用浏览器的 `fetch` 调
   `api.github.com/repos/.../releases/latest`，拿到最新 tag，用 `cmpVersion()`
   做三段数字比较；新版更大就调 `showUpdate()` 弹出 `🔔 发现新版 → 点击下载`，
   点击用 `openUrl()` 打开浏览器。失败全程 `try/catch` 静默——断网也不影响主功能。

---

## 5. 在公司电脑上跑起来学习

```bash
# 1. 装前端依赖（第一次）
npm install

# 2. 开发模式运行：改代码自动热重载，最适合边改边学
npm run tauri dev

# 3. 只跑 Rust 的纯逻辑单元测试（看 layout.rs 的测试怎么跑）
cd src-tauri && cargo test

# 4. 打包出免安装 exe
npm run tauri build -- --no-bundle
# 产物在 src-tauri/target/release/screen-hopper.exe
```

> 注意：第一次 `cargo` 编译会下载并编译很多依赖，慢是正常的（几分钟），之后就快了。
> 这台机器已配好国内镜像（rsproxy.cn），不用翻墙。

**上手练习建议**（改一处，立刻看效果，学得最快）：
1. 改 `src/styles.css` 里 `#89b4fa`（高亮蓝）换个颜色，`npm run tauri dev` 看横条变色。
2. 改 `src/main.ts` 里按钮文字 `${m.label} ${m.width}×${m.height}`，比如加上坐标。
3. 读 `src-tauri/src/layout.rs`，跑 `cargo test`，试着自己加一个测试用例。
4. 进阶：读 `display.rs` 的 `unsafe` 块，对照微软文档查 `ChangeDisplaySettingsExW`。

---

## 6. 一张图记住全局

```
点"屏2"按钮
   │  src/main.ts  switchTo(1)
   ▼
invoke("set_primary",{index:1})  ──────►  lib.rs  set_primary()
                                              │
                                              ▼
                                  layout.rs compute_layout()   ← 纯计算，有测试
                                  算出每块屏的新坐标
                                              │
                                              ▼
                                  display.rs set_primary()
                                  调 Win32 ChangeDisplaySettingsExW  ← unsafe
                                              │
   ┌──────────────────────────────────────────┘
   ▼  回到 src/main.ts
把窗口挪到新主屏 (0,0)，重新 render() 横条
```

读代码时，手指头按着这张图，走一遍 `屏2 按钮 → main.ts → lib.rs → layout.rs →
display.rs → 回 main.ts`，整个项目就通了。

---

## 7. 各文件一句话速查表

| 文件 | 一句话 |
|------|--------|
| `src/main.ts` | 前端全部逻辑：画横条、点击切屏、切完挪窗口、查更新 |
| `src/styles.css` | 横条深色样式 |
| `index.html` | 前端入口，就一个空 `#bar` 和 `#update` 容器 |
| `src-tauri/src/main.rs` | 程序入口，只调 `run()` |
| `src-tauri/src/lib.rs` | 注册 3 个命令、启动 Tauri |
| `src-tauri/src/layout.rs` | 纯坐标计算 + 单元测试（**最先读**） |
| `src-tauri/src/display.rs` | 调 Win32 枚举显示器 + 切主屏（`unsafe` 集中地） |
| `src-tauri/Cargo.toml` | Rust 依赖清单（windows / tauri / serde） |
| `src-tauri/tauri.conf.json` | 窗口尺寸、版本号、应用标识 |
| `src-tauri/capabilities/default.json` | 前端能调哪些窗口 API 的权限白名单 |

祝学习顺利。看不懂的地方对着第 3 节的概念表回查即可。
