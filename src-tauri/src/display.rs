use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    ChangeDisplaySettingsExW, CDS_NORESET, CDS_SET_PRIMARY, CDS_TYPE, CDS_UPDATEREGISTRY,
    DISP_CHANGE_SUCCESSFUL, DM_POSITION,
    EnumDisplayDevicesW, EnumDisplaySettingsW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP,
    DISPLAY_DEVICE_PRIMARY_DEVICE, DEVMODEW, ENUM_CURRENT_SETTINGS,
};
use crate::layout::{compute_layout, MonitorGeom};

#[derive(serde::Serialize, Clone, Debug)]
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

/// Enumerate all monitors attached to the desktop; index in enumeration order.
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
        if dd.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP == 0 {
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
            is_primary: dd.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE != 0,
        });
    }
    out
}

/// Make the monitor at `target` index the primary display.
/// Standard 3-step sequence: stage each monitor's position with NORESET + UPDATEREGISTRY
/// (the target also gets SET_PRIMARY at (0,0)), then a final NULL apply commits everything.
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

        // Re-read the current mode to preserve resolution/refresh; only change position.
        let mut dm = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        unsafe {
            let _ = EnumDisplaySettingsW(PCWSTR(name.as_ptr()), ENUM_CURRENT_SETTINGS, &mut dm);
            dm.dmFields = DM_POSITION;
            dm.Anonymous1.Anonymous2.dmPosition.x = p.x;
            dm.Anonymous1.Anonymous2.dmPosition.y = p.y;
        }

        let mut flags = CDS_UPDATEREGISTRY | CDS_NORESET;
        if p.is_primary {
            flags |= CDS_SET_PRIMARY;
        }

        let res = unsafe {
            ChangeDisplaySettingsExW(PCWSTR(name.as_ptr()), Some(&dm), HWND(std::ptr::null_mut()), flags, None)
        };
        if res != DISP_CHANGE_SUCCESSFUL {
            return Err(format!("设置 {} 失败 (code {})", m.label, res.0));
        }
    }

    // Final NULL call commits all staged changes at once.
    let res = unsafe {
        ChangeDisplaySettingsExW(PCWSTR::null(), None, HWND(std::ptr::null_mut()), CDS_TYPE(0), None)
    };
    if res != DISP_CHANGE_SUCCESSFUL {
        return Err(format!("应用变更失败 (code {})", res.0));
    }
    Ok(())
}
