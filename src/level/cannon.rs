use super::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Cannon {
    pub pos: vec2<f32>,
    pub rot: f32,
}
