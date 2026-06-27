import "./styles.css";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";

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

render();
