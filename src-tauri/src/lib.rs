use common::RevealSettings;
use tauri::AppHandle;

mod common;
mod utils;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str, app_handle: AppHandle) -> String {
    format!(
        "Hello, {}! You've been greeted from Rust!\n {}",
        name,
        utils::debug_info(&app_handle)
    )
}

#[tauri::command]
fn get_settings() -> RevealSettings {
    RevealSettings::default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
