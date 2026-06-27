# screen-hopper 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 做一个跑在 Windows 被控电脑上的 Tauri 小工具，可视化切换"主显示器"，让 toDesk 等单屏远程软件能查看任意一块屏。

**Architecture:** Rust 后端用 Win32 `ChangeDisplaySettingsEx` 切主屏，把"坐标重排"这段纯逻辑（`compute_layout`）抽出来做单元测试；TS/HTML 前端渲染按钮、调用命令、并在切换后把自身窗口移到新主屏。

**Tech Stack:** Tauri 2.x、Rust（`windows` crate）、TypeScript + Vite、WebView2。

---

## 文件结构

```
screen-hopper/
├── index.html                  # 前端入口（Vite + Tauri 模板）
├── package.json
├── src/
│   ├── main.ts                 # 前端逻辑：渲染按钮、调用命令、窗口跟随
│   └── styles.css              # 小横条样式
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json         # 窗口：无边框/置顶/不占任务栏
    └── src/
        ├── main.rs             # 调用 lib.rs 的 run()
        ├── lib.rs              # 注册 list_monitors / set_primary 命令
        ├── layout.rs           # 纯逻辑 compute_layout + 单元测试
        └── display.rs          # Win32：枚举显示器 + 应用切主屏
```

**测试策略：** `layout.rs` 是纯函数，走真正的 TDD（`cargo test`）。`display.rs` 涉及 Win32 副作用、依赖真实多屏硬件，无法在 CI 里单测——它的验证是**在多屏机器上手动运行确认**。计划中对这两类任务分别标注。

---

## Task 1: 安装工具链 + 初始化 Tauri 工程

**Files:**
- Create: 整个 `screen-hopper/` Tauri 骨架（在已存在的本地 git 仓库内）

- [ ] **Step 1: 安装 Microsoft C++ Build Tools（Rust MSVC 工具链依赖）**

Tauri 在 Windows 上需要 MSVC 链接器。装"Desktop development with C++"工作负载：

Run（管理员 PowerShell 或让用户手动装）：
```bash
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```
Expected: 安装完成。若已装 Visual Studio 带 C++ 则可跳过。

- [ ] **Step 2: 安装 Rust（rustup，默认 msvc 工具链）**

Run:
```bash
winget install Rustlang.Rustup
```
然后**新开一个终端**让 PATH 生效，验证：
```bash
rustc --version && cargo --version
```
Expected: 都打印版本号（如 `rustc 1.8x.x`）。

- [ ] **Step 3: 用 create-tauri-app 在仓库内生成骨架**

在 `C:\Users\19116\Documents\screen-hopper`（已是 git 仓库）内生成。create-tauri-app 要求目录为空或用当前目录：
```bash
cd /c/Users/19116/Documents/screen-hopper
npm create tauri-app@latest . -- --template vanilla-ts --manager npm --yes
```
若因 `.git`/`docs` 已存在而拒绝，改用临时目录生成再拷入：
```bash
cd /c/Users/19116/Documents
npm create tauri-app@latest sh-tmp -- --template vanilla-ts --manager npm --yes
cp -r sh-tmp/* sh-tmp/.gitignore screen-hopper/ && rm -rf sh-tmp
```
Expected: 出现 `src/`、`src-tauri/`、`index.html`、`package.json`。

- [ ] **Step 4: 安装依赖并跑通 dev**

Run:
```bash
cd /c/Users/19116/Documents/screen-hopper
npm install
npm run tauri dev
```
Expected: 弹出默认 Tauri 窗口。确认能起来后 `Ctrl+C` 关掉。

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore: scaffold tauri vanilla-ts project"
```

---

## Task 2: 纯逻辑 `compute_layout`（TDD）

把"给定各屏几何 + 目标屏 → 算出每块屏的新左上角坐标，目标屏落到 (0,0)"这段纯逻辑独立出来测。

**Files:**
- Create: `src-tauri/src/layout.rs`
- Modify: `src-tauri/src/lib.rs`（声明 `mod layout;`）

- [ ] **Step 1: 写失败的测试**

创建 `src-tauri/src/layout.rs`：
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorGeom {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geom(index: usize, x: i32, y: i32) -> MonitorGeom {
        MonitorGeom { index, x, y, width: 1920, height: 1080 }
    }

    #[test]
    fn promotes_target_to_origin_and_shifts_others() {
        let mons = vec![geom(0, 0, 0), geom(1, 1920, 0), geom(2, 3840, 0)];
        let out = compute_layout(&mons, 1).unwrap();
        assert_eq!(out, vec![
            Placement { index: 0, x: -1920, y: 0, is_primary: false },
            Placement { index: 1, x: 0,     y: 0, is_primary: true  },
            Placement { index: 2, x: 1920,  y: 0, is_primary: false },
        ]);
    }

    #[test]
    fn target_already_primary_is_unchanged() {
        let mons = vec![geom(0, 0, 0), geom(1, 1920, 0)];
        let out = compute_layout(&mons, 0).unwrap();
        assert_eq!(out[0], Placement { index: 0, x: 0, y: 0, is_primary: true });
        assert_eq!(out[1], Placement { index: 1, x: 1920, y: 0, is_primary: false });
    }

    #[test]
    fn handles_negative_and_vertical_offsets() {
        let mons = vec![geom(0, -1920, -200), geom(1, 0, 0)];
        let out = compute_layout(&mons, 0).unwrap();
        assert_eq!(out, vec![
            Placement { index: 0, x: 0,    y: 0,   is_primary: true  },
            Placement { index: 1, x: 1920, y: 200, is_primary: false },
        ]);
    }

    #[test]
    fn unknown_target_errors() {
        let mons = vec![geom(0, 0, 0)];
        assert!(compute_layout(&mons, 9).is_err());
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run:
```bash
cd src-tauri && cargo test layout
```
Expected: 编译失败 / `cannot find function compute_layout`。

- [ ] **Step 3: 写最小实现**

在 `layout.rs` 顶部（结构体下方、`#[cfg(test)]` 上方）加：
```rust
/// 把 `target` 屏挪到 (0,0)，其余屏保持相对偏移。
pub fn compute_layout(monitors: &[MonitorGeom], target: usize) -> Result<Vec<Placement>, String> {
    let t = monitors
        .iter()
        .find(|m| m.index == target)
        .ok_or_else(|| format!("monitor index {target} not found"))?;
    let (dx, dy) = (t.x, t.y);
    Ok(monitors
        .iter()
        .map(|m| Placement {
            index: m.index,
            x: m.x - dx,
            y: m.y - dy,
            is_primary: m.index == target,
        })
        .collect())
}
```

- [ ] **Step 4: 声明模块并运行测试确认通过**

在 `src-tauri/src/lib.rs` 顶部加一行 `mod layout;`（若 lib.rs 还没有该行）。
Run:
```bash
cd src-tauri && cargo test layout
```
Expected: `test result: ok. 4 passed`。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/layout.rs src-tauri/src/lib.rs
git commit -m "feat: add pure compute_layout with unit tests"
```

---

## Task 3: Win32 枚举显示器 → `list_monitors` 命令

**Files:**
- Create: `src-tauri/src/display.rs`
- Modify: `src-tauri/Cargo.toml`（加 `windows` 依赖）、`src-tauri/src/lib.rs`（注册命令）

- [ ] **Step 1: 加 windows crate 依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 下加：
```toml
[dependencies.windows]
version = "0.58"
features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
]
```
（`serde`、`tauri` 模板已有。）

- [ ] **Step 2: 写枚举实现**

创建 `src-tauri/src/display.rs`：
```rust
use crate::layout::{compute_layout, MonitorGeom};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    ChangeDisplaySettingsExW, EnumDisplayDevicesW, EnumDisplaySettingsW, CDS_NORESET,
    CDS_SET_PRIMARY, CDS_TYPE, CDS_UPDATEREGISTRY, DEVMODEW, DISPLAY_DEVICEW,
    DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_PRIMARY_DEVICE, DISP_CHANGE_SUCCESSFUL,
    DM_POSITION, ENUM_CURRENT_SETTINGS,
};

#[derive(serde::Serialize, Clone)]
pub struct Monitor {
    pub index: usize,
    pub device_name: String,
    pub label: String,
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

fn wide_to_string(w: &[u16]) -> String {
    let end = w.iter().position(|&c| c == 0).unwrap_or(w.len());
    String::from_utf16_lossy(&w[..end])
}

/// 枚举所有"已连接到桌面"的显示器，按枚举顺序赋 index。
pub fn enumerate() -> Vec<Monitor> {
    let mut out = Vec::new();
    let mut i = 0u32;
    loop {
        let mut dd = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        let ok = unsafe { EnumDisplayDevicesW(PCWSTR::null(), i, &mut dd, 0) };
        if !ok.as_bool() {
            break;
        }
        i += 1;
        if dd.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP.0 == 0 {
            continue;
        }

        let mut dm = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        let ok2 = unsafe {
            EnumDisplaySettingsW(PCWSTR(dd.DeviceName.as_ptr()), ENUM_CURRENT_SETTINGS, &mut dm)
        };
        if !ok2.as_bool() {
            continue;
        }

        let idx = out.len();
        out.push(Monitor {
            index: idx,
            device_name: wide_to_string(&dd.DeviceName),
            label: format!("屏{}", idx + 1),
            width: dm.dmPelsWidth as i32,
            height: dm.dmPelsHeight as i32,
            x: unsafe { dm.Anonymous1.Anonymous2.dmPosition.x },
            y: unsafe { dm.Anonymous1.Anonymous2.dmPosition.y },
            is_primary: dd.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE.0 != 0,
        });
    }
    out
}
```
> 注：`windows` crate 版本不同，union 字段路径（`Anonymous1.Anonymous2.dmPosition`）或常量是否带 `.0` 可能需微调；以 `cargo build` 报错为准对照该版本文档调整。

- [ ] **Step 3: 注册命令**

修改 `src-tauri/src/lib.rs`，确保有：
```rust
mod display;
mod layout;

#[tauri::command]
fn list_monitors() -> Vec<display::Monitor> {
    display::enumerate()
}
```
并在 `run()` 的 builder 链上把 handler 改成（保留模板已有的 plugin 行）：
```rust
.invoke_handler(tauri::generate_handler![list_monitors])
```

- [ ] **Step 4: 编译确认通过**

Run:
```bash
cd src-tauri && cargo build
```
Expected: 编译成功（有 warning 可忽略）。

- [ ] **Step 5: 手动验证枚举**

临时在前端 `src/main.ts` 顶部加 `import { invoke } from "@tauri-apps/api/core"; invoke("list_monitors").then(console.log);`，`npm run tauri dev`，在窗口 DevTools（右键→检查，或 F12）控制台看是否打印出你的显示器数组（数量、分辨率、`is_primary`）。看完把这行临时代码删掉。
Expected: 打印出与实际屏数一致的数组，有且仅有一块 `is_primary: true`。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/display.rs src-tauri/src/lib.rs
git commit -m "feat: enumerate monitors via Win32 (list_monitors command)"
```

---

## Task 4: Win32 应用切主屏 → `set_primary` 命令

**Files:**
- Modify: `src-tauri/src/display.rs`（加 `set_primary`）、`src-tauri/src/lib.rs`（注册命令）

- [ ] **Step 1: 写 set_primary 实现**

在 `src-tauri/src/display.rs` 末尾追加：
```rust
/// 把 index 对应的显示器设为主屏（标准三步法：逐屏 NORESET 暂存 → 空调用统一生效）。
pub fn set_primary(target: usize) -> Result<(), String> {
    let monitors = enumerate();
    if monitors.is_empty() {
        return Err("未检测到显示器".into());
    }
    let geom: Vec<MonitorGeom> = monitors
        .iter()
        .map(|m| MonitorGeom { index: m.index, x: m.x, y: m.y, width: m.width, height: m.height })
        .collect();
    let placements = compute_layout(&geom, target)?;

    for p in &placements {
        let m = &monitors[p.index];
        let name: Vec<u16> = m.device_name.encode_utf16().chain(std::iter::once(0)).collect();

        // 重新读当前模式，保留分辨率/刷新率，只改位置
        let mut dm = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        unsafe {
            EnumDisplaySettingsW(PCWSTR(name.as_ptr()), ENUM_CURRENT_SETTINGS, &mut dm);
            dm.dmFields = DM_POSITION;
            dm.Anonymous1.Anonymous2.dmPosition.x = p.x;
            dm.Anonymous1.Anonymous2.dmPosition.y = p.y;
        }

        let mut flags = CDS_UPDATEREGISTRY | CDS_NORESET;
        if p.is_primary {
            flags |= CDS_SET_PRIMARY;
        }

        let res = unsafe {
            ChangeDisplaySettingsExW(PCWSTR(name.as_ptr()), Some(&dm), HWND::default(), flags, None)
        };
        if res != DISP_CHANGE_SUCCESSFUL {
            return Err(format!("设置 {} 失败 (code {})", m.label, res.0));
        }
    }

    // 空调用：统一生效
    let res = unsafe {
        ChangeDisplaySettingsExW(PCWSTR::null(), None, HWND::default(), CDS_TYPE(0), None)
    };
    if res != DISP_CHANGE_SUCCESSFUL {
        return Err(format!("应用变更失败 (code {})", res.0));
    }
    Ok(())
}
```

- [ ] **Step 2: 注册命令**

在 `src-tauri/src/lib.rs` 加：
```rust
#[tauri::command]
fn set_primary(index: usize) -> Result<(), String> {
    display::set_primary(index)
}
```
并把 handler 改为：
```rust
.invoke_handler(tauri::generate_handler![list_monitors, set_primary])
```

- [ ] **Step 3: 编译确认通过**

Run:
```bash
cd src-tauri && cargo build
```
Expected: 编译成功。

- [ ] **Step 4: 手动验证切主屏（需多屏机器）**

`npm run tauri dev`，在 DevTools 控制台执行 `invoke("set_primary",{index:1})`（换不同 index）。
Expected: Windows 主屏切到对应显示器（任务栏/桌面图标跟着走）；用 toDesk 远程时画面跳到该屏。`compute_layout` 已被单测覆盖，此处只验证 Win32 副作用是否生效。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/display.rs src-tauri/src/lib.rs
git commit -m "feat: switch primary monitor via Win32 (set_primary command)"
```

---

## Task 5: 前端小横条界面 + 窗口跟随

**Files:**
- Modify: `index.html`、`src/main.ts`、`src/styles.css`、`src-tauri/tauri.conf.json`

- [ ] **Step 1: 配置窗口（无边框/置顶/不占任务栏/不可缩放）**

修改 `src-tauri/tauri.conf.json` 的 `app.windows[0]`，设为：
```json
{
  "title": "screen-hopper",
  "width": 360,
  "height": 64,
  "resizable": false,
  "decorations": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "transparent": false
}
```

- [ ] **Step 2: 写 HTML**

把 `index.html` 的 `<body>` 内容替换为：
```html
<body>
  <div id="bar" data-tauri-drag-region></div>
  <script type="module" src="/src/main.ts"></script>
</body>
```
`data-tauri-drag-region` 让用户能拖动无边框窗口。

- [ ] **Step 3: 写前端逻辑**

把 `src/main.ts` 整个替换为：
```ts
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalPosition } from "@tauri-apps/api/window";

interface Monitor {
  index: number;
  device_name: string;
  label: string;
  width: number;
  height: number;
  x: number;
  y: number;
  is_primary: boolean;
}

const bar = document.querySelector<HTMLDivElement>("#bar")!;

async function render() {
  const monitors = await invoke<Monitor[]>("list_monitors");
  bar.innerHTML = "";
  for (const m of monitors) {
    const btn = document.createElement("button");
    btn.textContent = `${m.label} ${m.width}×${m.height}`;
    btn.className = m.is_primary ? "mon active" : "mon";
    btn.disabled = m.is_primary;
    btn.addEventListener("click", () => switchTo(m.index));
    bar.appendChild(btn);
  }
  const refresh = document.createElement("button");
  refresh.textContent = "🔄";
  refresh.className = "refresh";
  refresh.addEventListener("click", () => render());
  bar.appendChild(refresh);
}

async function switchTo(index: number) {
  try {
    await invoke("set_primary", { index });
    // 新主屏现在位于 (0,0)，把工具窗口移到它的左上角，保证仍可见可点
    await getCurrentWindow().setPosition(new LogicalPosition(40, 40));
    await render();
  } catch (e) {
    alert("切换失败: " + e);
  }
}

render();
```

- [ ] **Step 4: 写样式**

把 `src/styles.css` 整个替换为：
```css
* { box-sizing: border-box; margin: 0; }
body { font-family: "Segoe UI", system-ui, sans-serif; background: #1e1e2e; }
#bar {
  display: flex;
  gap: 6px;
  padding: 10px;
  align-items: center;
  height: 100vh;
}
button.mon, button.refresh {
  border: 1px solid #45475a;
  background: #313244;
  color: #cdd6f4;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 13px;
  cursor: pointer;
}
button.mon.active {
  background: #89b4fa;
  color: #1e1e2e;
  font-weight: 600;
  cursor: default;
}
button.mon:disabled { cursor: default; }
button.refresh { padding: 8px; }
button.mon:hover:not(:disabled), button.refresh:hover { border-color: #89b4fa; }
```

- [ ] **Step 5: 跑通并手动验证整套交互（多屏机器）**

Run:
```bash
npm run tauri dev
```
Expected: 出现小横条，每块屏一个按钮，当前主屏高亮且禁用；点别的屏 → 主屏切换、窗口跳到新主屏左上角、高亮转移；点 🔄 重新检测。

- [ ] **Step 6: Commit**

```bash
git add index.html src/main.ts src/styles.css src-tauri/tauri.conf.json
git commit -m "feat: monitor switcher bar UI with window-follow"
```

---

## Task 6: 打包 portable exe + README

**Files:**
- Modify: `src-tauri/tauri.conf.json`（产品名/标识）
- Create: `README.md`

- [ ] **Step 1: 设置应用标识**

确认 `src-tauri/tauri.conf.json` 里 `productName` 为 `screen-hopper`，`identifier` 为如 `com.codewhatd.screenhopper`（不能用默认的 `com.tauri.dev`，否则打包报错）。

- [ ] **Step 2: 打包**

Run:
```bash
npm run tauri build
```
Expected: 生成安装包于 `src-tauri/target/release/bundle/`；**免安装的 portable exe** 在 `src-tauri/target/release/screen-hopper.exe`（拷这个到公司电脑双击即可，前提是有 WebView2）。

- [ ] **Step 3: 写 README**

创建 `README.md`：
```markdown
# screen-hopper

一个 Windows 小工具：可视化切换"主显示器"，让 toDesk / AnyDesk / RustDesk / 向日葵等
**只串流一块屏**的远程软件，能查看被控电脑的任意一块显示器。

## 原理
这些远程软件免费版只传"主显示器"。本工具调用 Win32 `ChangeDisplaySettingsEx`
切换主显示器，远程画面随之切到对应屏。切换后工具窗口自动移到新主屏，始终可见可点。

## 用法
1. 把 `screen-hopper.exe` 拷到**被控电脑**（公司电脑）运行。
2. 远程连上后，点小横条上的屏号按钮即可切换当前查看的屏。

## 依赖
- Windows 10/11（自带 WebView2）。

## 开发
\`\`\`bash
npm install
npm run tauri dev      # 开发
npm run tauri build    # 打包，产物见 src-tauri/target/release/
cd src-tauri && cargo test   # 跑纯逻辑单测
\`\`\`
```

- [ ] **Step 4: Commit**

```bash
git add README.md src-tauri/tauri.conf.json
git commit -m "chore: portable build config and README"
```

- [ ] **Step 5: 推送到 GitHub（网络通时）**

Run:
```bash
git push -u origin main
```
Expected: 推送成功。若仍被重置，需挂代理或换 SSH remote。

---

## 自检（Self-Review）

- **Spec 覆盖**：原理→Task 3/4；界面→Task 5；`list_monitors`/`set_primary`→Task 3/4；窗口跟随→Task 5 Step 3；边界（单屏/点中当前主屏/失败提示）→前端 `disabled` + `try/catch`、`compute_layout` 错误分支；分发→Task 6；前置条件（装 Rust、网络）→Task 1 / Task 6 Step 5。均有对应任务。
- **占位符**：无 TBD；所有代码步骤含完整代码。
- **类型一致**：`Monitor`/`MonitorGeom`/`Placement` 字段在前后端一致；命令名 `list_monitors`/`set_primary` 与前端 `invoke` 调用一致；`set_primary(index)` 参数名 `index` 与前端 `invoke("set_primary",{index})` 一致。
- **已知风险**：`windows` crate union 字段路径/常量 `.0` 随版本可能微调（Task 3 已注明）；Win32 副作用类任务靠手动验证（已注明）。
