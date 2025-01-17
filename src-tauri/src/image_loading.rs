use std::path::Path;
use std::{fs::File, path::PathBuf, str::FromStr, sync::Mutex};

use base64::engine::{general_purpose, Engine as _};
use rand::seq::SliceRandom;
use rand::thread_rng;
use tauri::image;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_fs::FilePath;
use tauri_plugin_fs::FsExt;

use crate::common::{ImageWithMeta, RevealState};

#[derive(Debug)]
enum FolderOrFiles {
    Folder(FilePath),
    Files(Vec<FilePath>),
}

pub fn get_image_paths(app: &AppHandle) -> Vec<FilePath> {
    // TODO get custom path from settings
    let custom_path: Option<FolderOrFiles> = None;

    // We use tauri's FilePath instead of a PathBuf, even for folders,
    // to allow for consistent use across target_oses.
    let folder_or_files: Result<FolderOrFiles, String> = custom_path
        .ok_or(String::new())
        .or_else(|e| {
            log::debug!("No image path in settings, trying 'reveal' in user's pictures folder ...");
            app.path()
                .picture_dir()
                .map_err(|tauri_err| e.clone() + "\n" + tauri_err.to_string().as_str())
                .map(|path| path.join("reveal"))
                .and_then(|path| {
                    if path.exists() {
                        Ok(path)
                    } else {
                        Err(e + "\nDefault path does not exist.")
                    }
                })
                .map(FilePath::from)
                .map(|f| FolderOrFiles::Folder(f))
        })
        .or_else(|e| {
            if cfg!(target_os = "android") {
                log::debug!("Trying 'reveal' in user's pictures folder for android ...");
                let android_pictures = PathBuf::from("/storage/emulated/0/Pictures/reveal");
                if android_pictures.exists() {
                    Ok(FolderOrFiles::Folder(FilePath::from(android_pictures)))
                } else {
                    Err(e + "\nAndroid default path does not exist.")
                }
            } else if cfg!(target_os = "ios") {
                log::debug!("Trying 'reveal' in user's documents folder for ios ...");
                app.path()
                    .document_dir()
                    .map_err(|tauri_err| e.clone() + "\n" + tauri_err.to_string().as_str())
                    .map(|path| path.join("reveal"))
                    .and_then(|path| {
                        if path.exists() {
                            Ok(path)
                        } else {
                            Err(e + "\niOS default path does not exist.")
                        }
                    })
                    .map(FilePath::from)
                    .map(|f| FolderOrFiles::Folder(f))
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
                    .map(|f| FolderOrFiles::Folder(f))
                    .ok_or(e + "\nUser canceled.")
            }
            #[cfg(not(desktop))]
            {
                app.dialog()
                    .file()
                    .blocking_pick_files()
                    .map(|f| FolderOrFiles::Files(f))
                    .ok_or(e + "\nUser canceled.")
            }
        });

    log::debug!("Final path(s) or error: {:?}", folder_or_files);

    match folder_or_files {
        Ok(FolderOrFiles::Folder(folder)) => {
            app.dialog()
                .message(format!(
                    "We'll collect all images within this folder:\n{:?}",
                    folder
                ))
                .blocking_show();
            collect_image_paths(folder)
        }
        Ok(FolderOrFiles::Files(files)) => {
            app.dialog()
                .message(format!(
                    "We'll use the images that you selected:\n{:?}",
                    files
                ))
                .blocking_show();
            files
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

fn collect_image_paths(folder_path: FilePath) -> Vec<FilePath> {
    // TODO make constant
    let image_extensions = ["jpg", "jpeg", "png", "gif", "webp", "svg"];
    // TODO error handling unwrap
    let mut image_paths = std::fs::read_dir(&folder_path.as_path().unwrap())
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
        // TODO error handling
        .filter_map(|e| std::fs::canonicalize(e.path()).ok())
        .map(FilePath::from)
        .collect::<Vec<_>>();

    image_paths.shuffle(&mut thread_rng());

    log::debug!("Found {} images.", image_paths.len());
    log::debug!("Final set of image paths: {:?}.", image_paths);

    image_paths
}

pub fn get_image(
    update_index: isize,
    app: &AppHandle,
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

    match image_path {
        FilePath::Path(pb) => std::fs::read(pb),
        FilePath::Url(_url) => app.fs().read(image_path.clone()),
    }
    .map(|bytes| general_purpose::STANDARD.encode(&bytes))
    .map(|base64| ImageWithMeta {
        base64: base64,
        image_type: match image_path {
            FilePath::Path(pb) => Ok(pb.clone()),
            FilePath::Url(_url) => image_path.clone().into_path(), // TODO error handling should happen here?!
        }
        .map_err(|e| e.to_string())
        .and_then(|pb| pb.extension().map(|s| s.to_owned()).ok_or("err".into()))
        .and_then(|s| s.to_str().map(|s| s.to_owned()).ok_or("err2".into()))
        .map(|image_type| {
            match image_type.as_str() {
                "jpg" => "jpeg".into(), // IANA only knows jpeg, not jpg
                "svg" => "svg+xml".into(),
                _ => image_type,
            }
        })
        .unwrap_or("png".into()), // Browsers are somewhat forgiving, try with png
    })
    .map_err(|e| e.to_string())
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
