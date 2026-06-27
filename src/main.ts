import "./styles.css";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, PhysicalPosition, LogicalSize } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";

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

const REPO = "CodeWhatD/screen-hopper";

const bar = document.querySelector<HTMLDivElement>("#bar")!;
const updateBox = document.querySelector<HTMLDivElement>("#update")!;

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
  refresh.addEventListener("click", () => {
    render();
    checkUpdate();
  });
  bar.appendChild(refresh);
}

async function switchTo(index: number) {
  try {
    await invoke("set_primary", { index });
    const w = getCurrentWindow();
    // Windows shifts the desktop coordinate origin ASYNCHRONOUSLY after the
    // display change. If we reposition immediately we land on the old layout
    // and get carried off-screen with the old primary. Re-assert the position
    // a few times across ~750ms so a later pass lands on the NEW primary
    // (always at physical 0,0) once the coordinate system has settled.
    // PhysicalPosition avoids cross-monitor DPI-scaling ambiguity.
    for (let i = 0; i < 5; i++) {
      await new Promise((r) => setTimeout(r, 150));
      await w.setPosition(new PhysicalPosition(40, 40));
    }
    await w.setFocus();
    await render();
  } catch (e) {
    alert("切换失败: " + e);
  }
}

/** Compare semver-ish tags ("v0.2.0" vs "0.1.1"). Returns 1 if a>b, -1 if a<b, 0 if equal. */
function cmpVersion(a: string, b: string): number {
  const pa = a.replace(/^v/, "").split(".").map(Number);
  const pb = b.replace(/^v/, "").split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    const d = (pa[i] || 0) - (pb[i] || 0);
    if (d !== 0) return d > 0 ? 1 : -1;
  }
  return 0;
}

async function showUpdate(latest: string, url: string) {
  updateBox.innerHTML = "";
  const btn = document.createElement("button");
  btn.textContent = `🔔 发现新版 ${latest} → 点击下载`;
  btn.addEventListener("click", () => openUrl(url));
  updateBox.appendChild(btn);
  updateBox.classList.add("show");
  // Grow the window so the notice row is visible (default height is 64).
  await getCurrentWindow().setSize(new LogicalSize(360, 96));
}

/** Ask GitHub for the latest release; show a notice if it's newer than us.
 *  Any network/parse failure is swallowed — the tool must work offline. */
async function checkUpdate() {
  try {
    const current = await invoke<string>("app_version");
    const resp = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!resp.ok) return;
    const data = await resp.json();
    const latest: string = data.tag_name;
    const url: string = data.html_url;
    if (latest && cmpVersion(latest, current) > 0) {
      await showUpdate(latest, url);
    }
  } catch {
    // 网络不通就静默跳过
  }
}

render();
checkUpdate();
