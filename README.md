# screen-hopper

**English** | [简体中文](./README.zh-CN.md)

A tiny Windows utility that visually switches the **primary monitor**, so single-screen
remote-desktop tools — toDesk / AnyDesk / RustDesk / Sunlogin — can view **any** monitor
of the controlled PC.

## Why
The free tiers of these remote tools usually stream only the **primary** monitor.
screen-hopper calls the Win32 `ChangeDisplaySettingsEx` API to switch which monitor is
primary; the remote view follows. After switching, the tool window automatically moves
onto the new primary so it stays visible and clickable in the remote viewer.

## Usage
1. Copy `screen-hopper.exe` to the **controlled PC** (e.g. your office machine) and run it.
2. Once connected remotely, click a screen button on the little bar to switch the screen
   you're viewing — or press the **global hotkey `Ctrl+Alt+1 / 2 / 3 …`** to jump straight
   to that screen. 🔄 re-detects monitors.

Download the latest `screen-hopper.exe` from the
[Releases page](https://github.com/CodeWhatD/screen-hopper/releases/latest).

## Requirements
- Windows 10/11 (ships with the WebView2 runtime).

## Development
```bash
npm install
npm run tauri dev                    # run in dev mode
npm run tauri build -- --no-bundle   # build the portable exe (src-tauri/target/release/screen-hopper.exe)
cd src-tauri && cargo test           # run the pure-logic unit tests
```

New to the codebase (or to Rust)? See the [Code Guide](./docs/CODE-GUIDE.md).

## License
MIT
