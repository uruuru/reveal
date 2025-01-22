mod common;
mod image_loading;
#[cfg(target_os = "ios")]
mod ios;
mod plane_covering;
mod questions;
mod utils;

use common::{Polygon, RevealObject, RevealSettings, RevealState};
use questions::simple_year_question;
use rand::seq::SliceRandom;
use rand::thread_rng;
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
fn get_image(
    u: isize,
    quiz_year: bool,
    app_handle: AppHandle,
    state: tauri::State<'_, Mutex<RevealState>>,
) -> RevealObject {
    image_loading::get_image(u, &app_handle, &state)
        .or_else(|_e| Ok::<_, String>(image_loading::example()))
        .map(|image_and_meta| {
            let mut reveal_object = RevealObject {
                image: image_and_meta.base64,
                image_type: image_and_meta.image_type,
                question: None,
                answers: Vec::new(),
                correct_answer: 0,
            };
            if quiz_year {
                log::debug!(
                    "Quiz year requested: {}",
                    image_and_meta
                        .date_taken
                        .map(|dt| dt.to_string())
                        .unwrap_or("Unknown date".into())
                );
                let qna = image_and_meta
                    .date_taken
                    .map(|dt| simple_year_question(&dt))
                    .unwrap_or_default();
                reveal_object.question = Some(qna.question);
                reveal_object.answers.extend(qna.answers);
                reveal_object.correct_answer = qna.idx_correct;
            }
            reveal_object
        })
        .unwrap()
}

/// Either detects image paths within a previously used path,
/// a default path, or a user selected path.
#[tauri::command]
fn get_image_paths(force_selection: bool, verbose: bool, app: AppHandle) -> String {
    tauri::async_runtime::spawn(async move {
        match image_loading::get_image_paths(force_selection, &app, verbose) {
            Ok((container, paths)) => {
                log::debug!("Found {} images.", paths.len());
                log::trace!("Final set of image paths: {:?}.", paths);

                let state = app.state::<Mutex<RevealState>>();
                let mut state = state.lock().unwrap();
                state.images = paths;
                state.image_index = 0;
                app.emit("image-paths-updated", container).unwrap();
            }
            Err(message) => {
                // TODO return proper errors and differentiate accordingly here
                if message != "User canceled." {
                    let state = app.state::<Mutex<RevealState>>();
                    let mut state = state.lock().unwrap();
                    state.images.clear();
                    state.image_index = 0;
                    app.emit("image-paths-failed", message).unwrap();
                }
            }
        }
    });
    "ok".to_string()
}

#[tauri::command]
fn load_covering(width: f64, height: f64, n: usize) -> Result<Vec<Polygon>, String> {
    let mut covering = plane_covering::cover_rectangles(n, width, height);
    covering.shuffle(&mut thread_rng());
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

            #[cfg(target_os = "ios")]
            {
                let handle = app.handle().to_owned();
                ios::mark_home_dir(&handle);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            debug_infos,
            get_settings,
            load_covering,
            get_image,
            get_image_paths,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
