mod common;
mod image_loading;
#[cfg(target_os = "ios")]
mod ios;
mod plane_covering;
mod questions;
#[cfg(target_os = "android")]
mod reveal_plugin_android;
mod utils;

use common::{Polygon, RevealObject, RevealSettings, RevealState};
use questions::simple_year_question;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
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
        .or_else(|e| {
            // Note that if the state does not contain any paths,
            // the 'get_image' method will already return exemplary data.
            // Hence, here we handle errors that have occurred during the
            // actual loading process.
            app_handle
                .dialog()
                .message(format!(
                    "Couldn't load an image. We'll show examples.\n\nDetails:\n{}",
                    e
                ))
                .kind(MessageDialogKind::Warning)
                .title("Loading image failed.")
                .show(|_| {});
            Ok::<_, String>(image_loading::example())
        })
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
fn get_image_paths(force_selection: bool, folder: bool, verbose: bool, app: AppHandle) -> String {
    let permissions_available;
    #[cfg(target_os = "android")]
    {
        use reveal_plugin_android::{PermissionResponse, RevealAndroidExt};
        permissions_available = match app.reveal_android().check_and_request_permissions() {
            Ok(PermissionResponse { value: Some(code) })
                if code == reveal_plugin_android::PERMISSION_GRANTED =>
            {
                true
            }
            _ => false,
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        permissions_available = true;
    }

    if !permissions_available {
        app.dialog()
            .message(
                "We do not have the permissions to access images \
                and hence will be showing exemplary images instead."
                    .to_string(),
            )
            .kind(MessageDialogKind::Warning)
            .title("Permissions missing.")
            .blocking_show();
        app.emit("image-paths-failed", "NoPermissions").unwrap();
        return "".into();
    }

    tauri::async_runtime::spawn(async move {
        match image_loading::get_image_paths(force_selection, folder, &app, verbose) {
            Ok((container, paths)) => {
                let img_cnt = paths.len();
                log::debug!("Found {} images.", img_cnt);
                log::trace!("Final set of image paths: {:?}.", paths);

                let state = app.state::<Mutex<RevealState>>();
                let mut state = state.lock().unwrap();
                state.images = paths;
                state.image_index = 0;

                if img_cnt > 0 {
                    app.emit("image-paths-updated", (container, img_cnt))
                        .unwrap();
                } else {
                    app.emit("image-paths-failed", "NoImages").unwrap();
                }
            }
            Err(message) => {
                // TODO return proper errors and differentiate accordingly here
                if message != "User canceled manual selection." || !force_selection {
                    let state = app.state::<Mutex<RevealState>>();
                    let mut state = state.lock().unwrap();
                    state.images.clear();
                    state.image_index = 0;

                    app.dialog()
                        .message(format!(
                            "We'll be showing exemplary images.\n\nWhat we have tried:\n{}",
                            message
                        ))
                        .kind(MessageDialogKind::Warning)
                        .title("Could not find images.")
                        .blocking_show();

                    app.emit("image-paths-failed", message).unwrap();
                }
            }
        }
    });

    "ok".to_string()
}

#[tauri::command]
fn load_covering(
    width: f64,
    height: f64,
    n: usize,
    object_type: String,
) -> Result<Vec<Polygon>, String> {
    let mut covering = match object_type.as_str() {
        "Rectangles" => plane_covering::cover_rectangles(n, width, height),
        _ => plane_covering::cover_triangles(n, width, height),
    };

    covering.shuffle(&mut thread_rng());
    Ok(covering)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_opener::init());

    #[cfg(target_os = "android")]
    {
        builder = builder.plugin(reveal_plugin_android::init());
    }

    builder
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
