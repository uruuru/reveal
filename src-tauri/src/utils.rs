use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn path_result_to_str(result: tauri::Result<PathBuf>) -> String {
    match result {
        Ok(pb) => pb.to_str().unwrap_or("Non-UTF8 path").to_owned(),
        Err(e) => format!("Not available ({e})").to_owned(),
    }
}

pub fn debug_info(app_handle: &AppHandle) -> String {
    let mut s = String::new();

    s.push_str("… Directories …\n");
    s.push_str(&format!(
        "Home: {}\n",
        path_result_to_str(app_handle.path().home_dir())
    ));
    s.push_str(&format!(
        "Documents: {}\n",
        path_result_to_str(app_handle.path().document_dir())
    ));
    s.push_str(&format!(
        "Pictures: {}\n",
        path_result_to_str(app_handle.path().picture_dir())
    ));
    s.push_str(&format!(
        "Data: {}\n",
        path_result_to_str(app_handle.path().data_dir())
    ));

    s.push_str("\n… Monitors …\n");
    match app_handle.available_monitors() {
        Ok(monitors) => {
            let unav = "Unavailable".to_owned();
            for m in monitors {
                s.push_str(&format!(
                    "{}: {}x{}\n",
                    m.name().unwrap_or(&unav),
                    m.size().width,
                    m.size().height,
                ));
            }
        }
        Err(err) => s.push_str(&format!("Not available ({})", err)),
    }

    s.push_str("\n… WebView Windows …\n");
    for w in app_handle.webview_windows() {
        let inner_size = w.1.inner_size().unwrap_or_default();
        s.push_str(&format!(
            "{}: {}x{}\n",
            w.0, inner_size.width, inner_size.height
        ));
    }

    s.push_str("\n… OS …\n");
    s.push_str(&format!("Platform: {}\n", tauri_plugin_os::platform()));
    s.push_str(&format!("Arch: {}\n", tauri_plugin_os::arch()));
    s.push_str(&format!("Family: {}\n", tauri_plugin_os::family()));
    s.push_str(&format!("Version: {}\n", tauri_plugin_os::version()));

    let pck = app_handle.package_info();
    s.push_str("\n… Package …\n");
    s.push_str(&format!("Crate: {}\n", pck.crate_name));
    s.push_str(&format!("Version: {}\n", pck.version));

    log::debug!("Assembled debug string:\n{s}");
    s
}
