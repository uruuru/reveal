use base64::engine::{general_purpose, Engine as _};
use common::{RevealObject, RevealSettings};
use tauri::AppHandle;

mod common;
mod utils;

#[tauri::command]
fn get_settings() -> RevealSettings {
    RevealSettings::default()
}

#[tauri::command]
fn debug_infos(app_handle: AppHandle) -> String {
    utils::debug_info(&app_handle)
}

#[tauri::command]
fn example() -> RevealObject {
    let example = include_bytes!("../assets/example.png");
    RevealObject {
        image: general_purpose::STANDARD.encode(example),
        image_type: "png".into(),
        covering: String::new(),
        question: None,
        answers: Vec::new(),
        correct_answer: 0,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![debug_infos, get_settings, example])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
