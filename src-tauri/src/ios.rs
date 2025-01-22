use tauri::AppHandle;
use tauri::Manager;

#[cfg(target_os = "ios")]
pub fn mark_home_dir(app: &AppHandle) {
    match app.path().document_dir() {
        Ok(docs) => {
            let marker_dir = docs.join("reveal");
            if !marker_dir.exists() {
                std::fs::create_dir(&marker_dir).ok();
            }
        }
        Err(_) => {} // fail silently
    }
}

