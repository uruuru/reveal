use serde::{Deserialize, Serialize};
use std::result::Result;
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    Manager, Runtime,
};

pub const PERMISSION_GRANTED: isize = 0;
#[allow(dead_code)]
pub const PERMISSION_DENIED: isize = -1;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PermissionResponse {
    pub value: Option<isize>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MimeRequestResponse {
    pub value: Option<String>,
}

pub struct RevealAndroid<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> RevealAndroid<R> {
    pub fn check_and_request_permissions(&self) -> Result<PermissionResponse, String> {
        self.0
            .run_mobile_plugin("checkAndRequestPermissions", ())
            .map_err(|e| e.to_string())
    }

    pub fn get_mime_type(&self, url: MimeRequestResponse) -> Result<MimeRequestResponse, String> {
        self.0
            .run_mobile_plugin("getMimeType", url)
            .map_err(|e| e.to_string())
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`]
pub trait RevealAndroidExt<R: Runtime> {
    fn reveal_android(&self) -> &RevealAndroid<R>;
}

impl<R: Runtime, T: Manager<R>> RevealAndroidExt<R> for T {
    fn reveal_android(&self) -> &RevealAndroid<R> {
        self.state::<RevealAndroid<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("reveal_android_plugin")
        .setup(|app, api| {
            let handle = api.register_android_plugin("li.oiu.reveal", "RevealPlugin")?;
            app.manage(RevealAndroid(handle));
            Ok(())
        })
        .build()
}
