mod common;
mod plane_covering;
mod utils;

use base64::engine::{general_purpose, Engine as _};
use common::{Polygon, RevealObject, RevealSettings, RevealState};
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

#[tauri::command]
fn get_settings(state: tauri::State<'_, Mutex<RevealState>>) -> RevealSettings {
    state.lock().unwrap().settings.clone()
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

#[tauri::command]
fn load_covering(width: f64, height: f64, n: usize) -> Result<Vec<Polygon>, String> {
    let covering = plane_covering::cover_rectangles(n, width, height);
    Ok(covering)
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
        .setup(|app| {
            app.manage(Mutex::new(RevealState::default()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            debug_infos,
            get_settings,
            example,
            load_covering,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
