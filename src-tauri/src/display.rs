use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP,
    DISPLAY_DEVICE_PRIMARY_DEVICE, DEVMODEW, ENUM_CURRENT_SETTINGS,
};

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
