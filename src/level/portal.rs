use super::*;

#[derive(Serialize, Deserialize)]
pub struct Portal {
    pub pos: vec2<f32>,
    pub dest: Option<usize>,
    pub color: Rgba<f32>,
}
