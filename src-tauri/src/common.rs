use serde::Serialize;
use std::path::PathBuf;
use ts_rs::TS;

pub struct RevealState {
    pub images: Vec<PathBuf>,
    pub image_index: usize,
}

#[derive(Default, Serialize, TS)]
#[ts(export)]
pub enum CoveringType {
    #[default]
    Triangles,
    Rectangles,
}

#[derive(Default, Serialize, TS)]
#[ts(export)]
pub enum UncoveringStrategy {
    #[default]
    Manual,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct RevealSettings {
    pub image_source: Option<PathBuf>,
    pub covering_type: CoveringType,
    pub uncovering_strategy: UncoveringStrategy,
    pub show_control_buttons: bool,
}

impl Default for RevealSettings {
    fn default() -> Self {
        RevealSettings {
            image_source: None,
            covering_type: CoveringType::Triangles,
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
    pub covering: String, // Custom format
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
