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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_monitors, set_primary])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
