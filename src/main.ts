import "./styles.css";
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
    // The new primary now sits at (0,0); move this window onto it so it stays
    // visible in the single-screen remote viewer.
    await getCurrentWindow().setPosition(new LogicalPosition(40, 40));
    await render();
  } catch (e) {
    alert("切换失败: " + e);
  }
}

render();
