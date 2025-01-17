use std::{fs, path::PathBuf, sync::Mutex};

use base64::engine::{general_purpose, Engine as _};
use rand::seq::SliceRandom;
use rand::thread_rng;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::common::{ImageWithMeta, RevealState};


pub fn get_image_paths(app: &AppHandle) -> Vec<PathBuf> {
    // TODO get custom path from settings
    let custom_path: Option<PathBuf> = None;

    let path: Result<PathBuf, String> = custom_path
        .ok_or(String::new())
        .or_else(|e| {
            log::debug!("No image path in settings, trying 'reveal' in user's pictures folder ...");
            app.path()
                .picture_dir()
                .ok()
                .map(|path| path.join("reveal"))
                .filter(|pb| pb.exists())
                .ok_or_else(|| e + "\nDefault path does not exist.")
        })
        .or_else(|e| {
            if cfg!(target_os = "android") {
                log::debug!("Trying 'reveal' in user's pictures folder for android ...");
            let android_pictures = PathBuf::from("/storage/emulated/0/Pictures/reveal");
            if android_pictures.exists() {
                Ok(android_pictures)
                } else {
                    Err(e + "\nAndroid default path does not exist.")
                }
            } else if cfg!(target_os = "ios") {
                log::debug!("Trying 'reveal' in user's documents folder for ios ...");
                app.path()
                    .document_dir()
                    .ok()
                    .map(|path| path.join("reveal"))
                    .filter(|pb| pb.exists())
                    .ok_or_else(|| e + "\niOS default path does not exist.")
            } else {
                Err(e)
            }
        })
        .or_else(|e| {
            log::debug!("Asking the user to select a folder ...");
            // Folder picker currently not implemented for mobile, hence we work around it ...
            // ... by letting the user select an image within the desired folder.
            // We need to use the cfg attributes here, since 'blocking_pick_folder' is not available for compilation.
            #[cfg(desktop)]
            {
                app.dialog()
                    .file()
                    .blocking_pick_folder()
                    .ok_or(e.clone() + "\nUser canceled.")
                    .and_then(|folder_path| {
                        folder_path
                            .into_path()
                            .map_err(|inner| e + "\n" + inner.to_string().as_str())
                    })
            }
            #[cfg(not(desktop))]
            {
                app.dialog()
                    .file()
                    .blocking_pick_file()
                    .ok_or(e.clone() + "\nUser canceled.")
                    .and_then(|file_path| {
                        file_path
                            .into_path()
                            .map_err(|inner| e + "\n" + inner.to_string().as_str())
                    })
            }
        });

    log::debug!("Final path or error: {:?}", path);

    match path {
        Ok(folder_path) => {
            app.dialog()
                .message(format!(
                    "We'll use this path to load the images: {:?}",
                    folder_path
                ))
                .blocking_show();

            collect_image_paths(folder_path)
        }
        Err(message) => {
        app.dialog()
            .message(format!("Could not find a location with images. Will be showing exemplary images.\n\n{}", message))
            .kind(MessageDialogKind::Warning)
            .title("☹")
            .blocking_show();

            Vec::new()
        }
    }
}

fn collect_image_paths(folder_path: PathBuf) -> Vec<PathBuf> {
    // TODO make constant
    let image_extensions = ["jpg", "jpeg", "png", "gif", "webp", "svg"];
    let mut image_paths = fs::read_dir(&folder_path)
        .ok()
        .unwrap() // TODO handle
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| image_extensions.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .filter_map(|e| fs::canonicalize(e.path()).ok())
        .collect::<Vec<_>>();

    image_paths.shuffle(&mut thread_rng());

    log::debug!("Found {} images.", image_paths.len());
    log::debug!("Final set of image paths: {:?}.", image_paths);

    image_paths
}

pub fn get_image(
    update_index: isize,
    state: &State<'_, Mutex<RevealState>>,
) -> Result<ImageWithMeta, String> {
    let mut state = state.lock().unwrap();

    if state.images.is_empty() {
        return Err("No images loaded.".to_string());
    }

    let new_index = (state.image_index as isize + update_index)
        .rem_euclid(state.images.len() as isize) as usize;
    log::debug!(
        "Updating index {} by {update_index}. New {}.",
        state.image_index,
        new_index
    );
    state.image_index = new_index;

    let image_path = &state.images[state.image_index];
    let base64_image = fs::read(image_path)
        .map(|bytes| general_purpose::STANDARD.encode(&bytes))
        .map_err(|e| format!("Error reading: {}", e));
    // TODO error handling

    Ok(ImageWithMeta {
        base64: base64_image.unwrap(),
        image_type: image_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|image_type| {
                match image_type {
                    "jpg" => "jpeg", // IANA only knows jpeg, not jpg
                    "svg" => "svg+xml",
                    _ => image_type,
                }
            })
            .unwrap_or("png") // Browsers are somewhat forgiving, try with png
            .into(),
    })
}

pub fn example() -> ImageWithMeta {
    // TODO add 2-3, select randomly
    // TODO make a constant
    let example = include_bytes!("../assets/example.png");
    ImageWithMeta {
        base64: general_purpose::STANDARD.encode(example),
        image_type: "png".into(),
    }
}

