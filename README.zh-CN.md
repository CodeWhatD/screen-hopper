# screen-hopper

[English](./README.md) | **简体中文**

一个 Windows 小工具：可视化切换“主显示器”，让 toDesk / AnyDesk / RustDesk / 向日葵等
**只串流一块屏**的远程软件，能查看被控电脑的任意一块显示器。

## 原理
这些远程软件免费版通常只传“主显示器”。本工具调用 Win32 `ChangeDisplaySettingsEx`
切换主显示器，远程画面随之切到对应屏。切换后工具窗口自动移到新主屏，始终可见可点。

## 用法
1. 把 `screen-hopper.exe` 拷到**被控电脑**（公司电脑）运行。
2. 远程连上后，点小横条上的屏号按钮即可切换当前查看的屏；🔄 重新检测显示器。

到 [Releases 页面](https://github.com/CodeWhatD/screen-hopper/releases/latest) 下载最新的
`screen-hopper.exe`。

## 依赖
- Windows 10/11（自带 WebView2 运行时）。

## 开发
```bash
npm install
npm run tauri dev                    # 开发运行
npm run tauri build -- --no-bundle   # 打包出免安装 exe（src-tauri/target/release/screen-hopper.exe）
cd src-tauri && cargo test           # 运行纯逻辑单元测试
```

新手（或没用过 Rust）？看[代码导读](./docs/CODE-GUIDE.md)。

## 许可
MIT
