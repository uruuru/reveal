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

    s.push_str("Directories:\n");
    s.push_str(&format!(
        "\tHome: {}\n",
        path_result_to_str(app_handle.path().home_dir())
    ));
    s.push_str(&format!(
        "\tDocuments: {}\n",
        path_result_to_str(app_handle.path().document_dir())
    ));
    s.push_str(&format!(
        "\tPictures: {}\n",
        path_result_to_str(app_handle.path().picture_dir())
    ));
    s.push_str(&format!(
        "\tData: {}\n",
        path_result_to_str(app_handle.path().data_dir())
    ));

    s.push_str("\nMonitors:\n");
    if let Ok(monitors) = app_handle.available_monitors() {
        let unav = "Unavailable".to_owned();
        for m in monitors {
            s.push_str(&format!(
                "\t{}: {}x{}\n",
                m.name().unwrap_or(&unav),
                m.size().width,
                m.size().height,
            ));
        }
    }

    s.push_str("\nWebView Windows:\n");
    for w in app_handle.webview_windows() {
        let inner_size = w.1.inner_size().unwrap_or_default();
        s.push_str(&format!(
            "\t{}: {}x{}\n",
            w.0, inner_size.width, inner_size.height
        ));
    }

    s.push_str("\nOS:\n");
    s.push_str(&format!("\tPlatform: {}\n", tauri_plugin_os::platform()));
    s.push_str(&format!("\tArch: {}\n", tauri_plugin_os::arch()));
    s.push_str(&format!("\tFamily: {}\n", tauri_plugin_os::family()));
    s.push_str(&format!("\tVersion: {}\n", tauri_plugin_os::version()));

    let pck = app_handle.package_info();
    s.push_str("\nPackage:\n");
    s.push_str(&format!("\tCrate: {}\n", pck.crate_name));
    s.push_str(&format!("\tVersion: {}\n", pck.version));

    log::debug!("Assembled debug string:\n{s}");
    s
}
