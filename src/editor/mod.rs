use super::*;

pub struct EditorState {
    pub next_autosave: f32,
    pub start_drag: Option<Vec2<f32>>,
    pub face_points: Vec<Vec2<f32>>,
    pub selected_surface: String,
    pub selected_tile: String,
    pub wind_drag: Option<(usize, Vec2<f32>)>,
    pub selected_object: String,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            next_autosave: 0.0,
            start_drag: None,
            face_points: vec![],
            selected_surface: "".to_owned(),
            selected_tile: "".to_owned(),
            selected_object: "".to_owned(),
            wind_drag: None,
        }
    }
}
