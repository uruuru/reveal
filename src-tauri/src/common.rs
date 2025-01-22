use chrono::NaiveDateTime;
use serde::Serialize;
use tauri_plugin_fs::FilePath;
use ts_rs::TS;

#[derive(Default)]
pub struct RevealState {
    pub images: Vec<FilePath>,
    pub image_index: usize,
    pub settings: RevealSettings,
}

#[derive(Clone, Default, Serialize, TS)]
#[ts(export)]
pub enum CoveringType {
    #[default]
    Triangles,
    Rectangles,
}

#[derive(Clone, Default, Serialize, TS)]
#[ts(export)]
pub enum UncoveringStrategy {
    #[default]
    Manual,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct RevealSettings {
    pub image_source: Option<String>,
    pub covering_type: CoveringType,
    /// Approximate number of objects to cover the image with.
    pub covering_object_count: usize,
    pub uncovering_strategy: UncoveringStrategy,
    pub show_control_buttons: bool,
}

impl Default for RevealSettings {
    fn default() -> Self {
        RevealSettings {
            image_source: None,
            covering_type: CoveringType::Rectangles,
            covering_object_count: 10,
            uncovering_strategy: UncoveringStrategy::Manual,
            show_control_buttons: true,
        }
    }
}

#[derive(Default, Serialize, TS)]
#[ts(export)]
pub struct RevealObject {
    /// Base64 encoded image
    pub image: String,
    /// MIME subtype of the image, e.g. 'png' or 'webp'
    pub image_type: String,
    pub question: Option<String>,
    pub answers: Vec<String>,
    pub correct_answer: usize,
}

#[derive(Debug, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Serialize)]
pub struct Polygon {
    pub pnts: Vec<Point>,
}

pub struct ImageWithMeta {
    pub base64: String,
    pub image_type: String,
    pub date_taken: Option<NaiveDateTime>,
}
