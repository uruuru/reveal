mod common;
mod image_loading;
mod plane_covering;
mod utils;

use base64::engine::{general_purpose, Engine as _};
use common::{Polygon, RevealObject, RevealSettings, RevealState};
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

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
fn get_image(
    u: isize,
    app_handle: AppHandle,
    state: tauri::State<'_, Mutex<RevealState>>,
) -> RevealObject {
    image_loading::get_image(u, &app_handle, &state)
        .or_else(|_e| Ok::<_, String>(image_loading::example()))
        .map(|image_and_meta| RevealObject {
            image: image_and_meta.base64,
            image_type: image_and_meta.image_type,
            covering: String::new(),
            question: None,
            answers: Vec::new(),
            correct_answer: 0,
        })
        .unwrap()
}

/// Either detects image paths within a previously used path,
/// a default path, or a user selected path.
#[tauri::command]
fn get_image_paths(force_selection: bool, app: AppHandle) -> String {
    tauri::async_runtime::spawn(async move {
        match image_loading::get_image_paths(force_selection, &app) {
            Ok((container, paths)) => {

                log::debug!("Found {} images.", paths.len());
                log::trace!("Final set of image paths: {:?}.", paths);

                let state = app.state::<Mutex<RevealState>>();
                let mut state = state.lock().unwrap();
                state.images = paths;
                state.image_index = 0;
                // TODO send the first reveal object, additionally? Or the state / loaded paths?
                app.emit("image-paths-updated", container).unwrap();
            }
            Err(message) => {
                app.emit("image-paths-failed", message).unwrap();
            }
        }
    });
    "ok".to_string()
}

#[tauri::command]
fn load_covering(width: f64, height: f64, n: usize) -> Result<Vec<Polygon>, String> {
    let covering = plane_covering::cover_rectangles(n, width, height);
    Ok(covering)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.store("settings.json")?;
            app.manage(Mutex::new(RevealState::default()));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            debug_infos,
            get_settings,
            example,
            load_covering,
            get_image,
            get_image_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
