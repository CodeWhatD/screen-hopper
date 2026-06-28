mod layout;
mod display;

#[tauri::command]
fn list_monitors() -> Vec<display::Monitor> {
    display::enumerate()
}

#[tauri::command]
fn set_primary(index: usize) -> Result<(), String> {
    display::set_primary(index)
}

/// Version baked from Cargo.toml at compile time; the frontend compares it
/// against the latest GitHub release tag to decide whether to show the update notice.
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![list_monitors, set_primary, app_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
