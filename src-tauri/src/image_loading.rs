use base64::engine::{general_purpose, Engine as _};
use chrono::NaiveDateTime;
use exif::{In, Reader, Tag};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use serde_json::json;
use std::{path::PathBuf, sync::Mutex};
use tauri::Emitter;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FilePath;
use tauri_plugin_fs::FsExt;
use tauri_plugin_store::StoreExt;

use crate::common::{ImageWithMeta, RevealState};

#[derive(Debug)]
enum FolderOrFiles {
    Folder(FilePath),
    #[allow(dead_code)]
    Files(Vec<FilePath>),
}

#[cfg(desktop)]
const SUPPORTED_IMAGE_EXTENSIONS: [&str; 6] = ["jpg", "jpeg", "png", "webp", "gif", "svg"];
#[cfg(not(desktop))]
const SUPPORTED_IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

fn exists_is_dir_and_non_empty(path: &PathBuf) -> bool {
    path.exists()
        && path.is_dir()
        && path
            .read_dir()
            .ok()
            .map(|mut dir| dir.next().is_some())
            .unwrap_or(false)
}

fn get_image_paths_automatic(app: &AppHandle, verbose: bool) -> Result<FolderOrFiles, String> {
    // Look for previous path in settings
    let custom_path = app
        .get_store("settings.json")
        .unwrap()
        .get("loaded_from_folder")
        .ok_or(String::from("No folder saved in local settings."))
        .and_then(|ps| serde_json::from_value::<PathBuf>(ps).map_err(|e| e.to_string()))
        .and_then(|pb| {
            // The user may have deleted the folder since last execution.
            if exists_is_dir_and_non_empty(&pb) {
                Ok(pb)
            } else {
                Err("Folder from local settings does not exist (anymore).".into())
            }
        })
        .map(|pb| FolderOrFiles::Folder(FilePath::from(pb)));

    // We use tauri's FilePath instead of a PathBuf, even for folders,
    // to allow for consistent use across target_oses.
    let folder_or_files: Result<FolderOrFiles, String> = custom_path
        .or_else(|e| {
            log::debug!("Trying 'reveal' in user's pictures folder ...");
            app.path()
                .picture_dir()
                .map_err(|tauri_err| {
                    e.clone()
                        + "\nNo user-specific pictures folder ("
                        + tauri_err.to_string().as_str()
                        + ")."
                })
                .map(|path| path.join("reveal"))
                .and_then(|path| {
                    if exists_is_dir_and_non_empty(&path) {
                        Ok(path)
                    } else {
                        Err(e + "\nNo 'reveal' folder in user's pictures.")
                    }
                })
                .map(FilePath::from)
                .map(|f| FolderOrFiles::Folder(f))
        })
        .or_else(|e| {
            if cfg!(target_os = "android") {
                log::debug!("Trying 'reveal' in user's pictures folder for android ...");
                let android_pictures = PathBuf::from("/storage/emulated/0/Pictures/reveal");
                if exists_is_dir_and_non_empty(&android_pictures) {
                    Ok(FolderOrFiles::Folder(FilePath::from(android_pictures)))
                } else {
                    Err(e + "\nNo 'reveal' folder in user's pictures (android).")
                }
            } else if cfg!(target_os = "ios") {
                log::debug!("Trying 'reveal' in user's documents folder for ios ...");
                app.path()
                    .document_dir()
                    .map_err(|tauri_err| e.clone() + "\n" + tauri_err.to_string().as_str())
                    .map(|path| path.join("reveal"))
                    .and_then(|path| {
                        if exists_is_dir_and_non_empty(&path) {
                            Ok(path)
                        } else {
                            Err(e + "\nNo 'reveal' folder in user's local folder (iOS).")
                        }
                    })
                    .map(FilePath::from)
                    .map(|f| FolderOrFiles::Folder(f))
            } else if cfg!(desktop) {
                log::debug!("Trying 'reveal_images' next to the executable ...");
                std::env::current_exe()
                    .map_err(|inner| e.clone() + "\n" + inner.to_string().as_str())
                    .and_then(|exe_path| {
                        exe_path
                            .parent()
                            .map(ToOwned::to_owned)
                            .ok_or(e.clone() + "\nUnable to access exe's parent folder.".into())
                    })
                    .map(|exe_dir| exe_dir.join("reveal_images"))
                    .and_then(|reveal_dir| {
                        if exists_is_dir_and_non_empty(&reveal_dir) {
                            Ok(FolderOrFiles::Folder(FilePath::from(reveal_dir)))
                        } else {
                            Err(e + "\nNo 'reveal_images' folder next to the executable.".into())
                        }
                    })
            } else {
                Err(e)
            }
        })
        .or_else(|e| {
            log::debug!("Asking the user to select a folder ...");
            get_image_paths_user(app, false, true, verbose)
                .map_err(|inner| e + "\n" + inner.as_str())
        });

    folder_or_files
}

fn get_image_paths_user(
    app: &AppHandle,
    force_user_selection: bool,
    folder: bool,
    verbose: bool,
) -> Result<FolderOrFiles, String> {
    // Folder picker currently not implemented for mobile, hence we work around it ...
    // We need to use the cfg attributes here, since 'blocking_pick_folder' is not available for compilation.
    let folder_or_images = if folder { "a folder" } else { "images" };
    let selection;
    #[cfg(desktop)]
    {
        if verbose && !force_user_selection {
            app.dialog()
                .message(format!(
                    "Please select {folder_or_images} in the next dialog \
                    from which supported images will be loaded."
                ))
                .title(format!("Select {folder_or_images}."))
                .blocking_show();
        }

        if folder {
            selection = app
                .dialog()
                .file()
                .blocking_pick_folder()
                .map(|f| FolderOrFiles::Folder(f))
                .ok_or("User canceled manual selection.".to_string());
        } else {
            selection = app
                .dialog()
                .file()
                .blocking_pick_files()
                // TODO We could add file filter here to narrow selection to supported types.
                .map(|f| FolderOrFiles::Files(f))
                .ok_or("User canceled manual selection.".to_string())
        }
    }
    #[cfg(not(desktop))]
    {
        if verbose && !force_user_selection {
            app.dialog()
                .message(format!(
                    "Please select all images in the next dialog \
                    that shall be loaded."
                ))
                .title("Select images.")
                .blocking_show();
        }

        let mut picker = app.dialog().file();

        // TODO revisit the filter in the future, currently it behaves inconsistently
        // for both iOS and android.
        // There's an open PR on this topic https://github.com/tauri-apps/plugins-workspace/issues/1578
        #[cfg(target_os = "ios")]
        {
            // NOTE ios requires mimetypes, not extensions here at this point.
            // With this behavior:
            if folder {
                // "image/jpeg", "text/plain" --> file picker (mixed types)
                // at this point, must list all supported types
                picker = picker.add_filter(
                    "images_file_picker",
                    &["image/jpeg", "image/png", "image/webp", "text/plain"],
                );
            } else {
                // "image/jpeg" --> photo picker (also allows pngs and webp at this point)
                picker = picker.add_filter("images_photo_picker", &["image/jpeg"]);
            }
        }

        selection = picker
            .blocking_pick_files()
            .map(|f| FolderOrFiles::Files(f))
            .ok_or("User canceled manual selection.".to_string());
    }
    selection
}

pub fn get_image_paths(
    force_user_selection: bool,
    folder: bool,
    app: &AppHandle,
    verbose: bool,
) -> Result<(Option<FilePath>, Vec<FilePath>), String> {
    let folder_or_files = if force_user_selection {
        get_image_paths_user(app, force_user_selection, folder, verbose)
    } else {
        get_image_paths_automatic(app, verbose)
    };

    log::debug!("Final path(s) or error: {:?}", folder_or_files);

    match folder_or_files {
        Ok(FolderOrFiles::Folder(folder)) => {
            if verbose {
                app.dialog()
                    .message(format!(
                        "We'll collect all images within this folder:\n{}",
                        folder.to_string()
                    ))
                    .blocking_show();
            }
            let store = app.get_store("settings.json").unwrap();
            store.set("loaded_from_folder", json!(folder.as_path().unwrap()));

            let filtered_and_shuffled_paths = match folder.clone() {
                FilePath::Path(pb) => Some(pb)
                    .map(load_from_folder)
                    .map(|t| filter_to_supported_images(app, &t))
                    .map(|mut v| {
                        shuffle(&mut v);
                        v
                    })
                    .unwrap(),
                FilePath::Url(_url) => unimplemented!(), // TODO
            };

            Ok((Some(folder.clone()), filtered_and_shuffled_paths))
        }
        Ok(FolderOrFiles::Files(files)) => {
            let number_of_selected = files.len();
            let filtered_and_shuffled_paths = Some(files)
                .map(|t| filter_to_supported_images(app, &t))
                .map(|mut v| {
                    shuffle(&mut v);
                    v
                })
                .unwrap();

            if verbose {
                let plural = (filtered_and_shuffled_paths.len() > 1)
                    .then(|| "s")
                    .unwrap_or("");
                let info_message = if number_of_selected != filtered_and_shuffled_paths.len() {
                    format!(
                        "We'll use {} of the {} selected image{}. The others are not supported.",
                        filtered_and_shuffled_paths.len(),
                        number_of_selected,
                        plural
                    )
                } else {
                    format!(
                        "We'll use all of the {} selected image{}.",
                        number_of_selected, plural
                    )
                };
                app.dialog().message(info_message).blocking_show();
            }

            Ok((None, filtered_and_shuffled_paths))
        }
        Err(message) => Err(message),
    }
}

fn load_from_folder(folder_path: PathBuf) -> Vec<FilePath> {
    assert!(folder_path.is_dir() && folder_path.exists());

    match std::fs::read_dir(&folder_path) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| std::fs::canonicalize(entry.path()).ok())
            .map(FilePath::from)
            .collect(),
        Err(e) => {
            log::error!("Couldn't load from path '{:?}': {:?}", folder_path, e);
            Vec::new()
        }
    }
}

#[allow(unused_variables)]
fn filter_to_supported_images(app: &AppHandle, file_paths: &[FilePath]) -> Vec<FilePath> {
    file_paths
        .iter()
        .filter(|fp| match fp {
            FilePath::Path(path_buf) => path_buf.is_file() && path_buf.exists(),
            FilePath::Url(_url) => true, // TODO ...
        })
        .filter(|fp| match fp {
            FilePath::Path(path_buf) => path_buf
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false),
            FilePath::Url(url) => {
                log::debug!("Checking extension of {:?}", url);
                #[cfg(target_os = "android")]
                {
                    // Android content urls start with 'content://'
                    // and do _not_ necessarily include the original file name nor the file extension
                    // TODO hopefully the following is provided by 'tauri_plugin_fs' at some point.
                    use crate::reveal_plugin_android::{MimeRequestResponse, RevealAndroidExt};
                    let response = app.reveal_android().get_mime_type(MimeRequestResponse {
                        value: Some(url.to_string()),
                    });
                    log::debug!("Response from android mime request: {:?}", response);
                    match response {
                        Ok(MimeRequestResponse {
                            value: Some(mime_type),
                        }) => SUPPORTED_IMAGE_EXTENSIONS.contains(&mime_type.as_str()),
                        _ => false,
                    }
                }
                #[cfg(not(target_os = "android"))]
                {
                    // iOS urls start with file:// and contain the file extension
                    url.as_str()
                        .rfind('.')
                        .map(|pos| url.as_str()[pos + 1..].into())
                        .map(|ext| SUPPORTED_IMAGE_EXTENSIONS.contains(&ext))
                        .unwrap_or(false)
                }
            }
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn shuffle(image_paths: &mut [FilePath]) {
    image_paths.shuffle(&mut thread_rng());
}

pub fn get_image(
    update_index: isize,
    app: &AppHandle,
    state: &State<'_, Mutex<RevealState>>,
) -> Result<ImageWithMeta, String> {
    let mut state = state.lock().unwrap();

    if state.images.is_empty() {
        return Ok(example());
    }

    let new_index = (state.image_index as isize + update_index)
        .rem_euclid(state.images.len() as isize) as usize;
    log::debug!(
        "Updating index {} by {update_index}. New {}.",
        state.image_index,
        new_index
    );
    state.image_index = new_index;

    app.emit("image-index", (new_index, state.images.len()))
        .unwrap();

    let image_path = &state.images[state.image_index];

    match image_path {
        FilePath::Path(pb) => std::fs::read(pb),
        FilePath::Url(_url) => app.fs().read(image_path.clone()),
    }
    .map(|bytes| (general_purpose::STANDARD.encode(&bytes), read_exif(&bytes)))
    .map(|(base64, exif)| ImageWithMeta {
        base64: base64,
        date_taken: match exif {
            // TODO only do this if needed
            Ok(date_taken) => Some(date_taken),
            Err(msg) => {
                log::debug!("Could not load exif: {}", msg);
                None
            }
        },
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

const EXAMPLES: [(&[u8], &str); 5] = [
    (include_bytes!("../assets/examples/example_1.png"), "png"),
    (include_bytes!("../assets/examples/example_2.jpg"), "jpg"),
    (include_bytes!("../assets/examples/example_3.webp"), "webp"),
    (include_bytes!("../assets/examples/example_4.gif"), "gif"),
    (include_bytes!("../assets/examples/example_5.svg"), "svg"),
];

/// Returns a randomly selected exemplary image.
pub fn example() -> ImageWithMeta {
    let mut rng = rand::thread_rng();
    let supported_examples = EXAMPLES
        .iter()
        .filter(|(_, ext)| SUPPORTED_IMAGE_EXTENSIONS.contains(ext))
        .collect::<Vec<_>>();
    let selected = supported_examples[rng.gen_range(0..supported_examples.len())];
    ImageWithMeta {
        base64: general_purpose::STANDARD.encode(selected.0),
        image_type: selected.1.into(),
        date_taken: None,
    }
}

fn read_exif(bytes: &[u8]) -> Result<NaiveDateTime, String> {
    let mut bufreader = std::io::Cursor::new(bytes);
    Reader::new()
        .read_from_container(&mut bufreader)
        .map_err(|e| e.to_string())
        .and_then(|data| {
            data.get_field(Tag::DateTimeOriginal, In::PRIMARY)
                .ok_or("DateTimeOriginal not included".into())
                .map(|field| field.display_value().to_string())
        })
        .and_then(|s| {
            log::debug!("Exif str: {}", s);
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| format!("{} ({})", e.to_string(), s))
        })
}
