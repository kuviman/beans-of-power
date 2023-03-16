use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tile {
    pub vertices: [vec2<f32>; 3],
    #[serde(default = "zero_vec")]
    pub flow: vec2<f32>,
    pub type_name: String,
}

#[derive(geng::Assets, Deserialize)]
#[asset(json)]
pub struct TileParams {
    #[serde(default)]
    pub background: bool,
    #[serde(default)]
    pub friction_along_flow: f32,
    #[serde(default)]
    pub friction: f32,
    #[serde(default)]
    pub texture_movement_frequency: f32,
    #[serde(default)]
    pub texture_movement_amplitude: f32,
    #[serde(default = "zero_vec")]
    pub additional_force: vec2<f32>,
    pub time_scale: Option<f32>,
}

#[derive(geng::Assets)]
pub struct TileAssets {
    pub params: TileParams,
    #[asset(postprocess = "make_repeated")]
    pub texture: Texture,
}

fn make_repeated(texture: &mut Texture) {
    texture.0.set_wrap_mode(ugli::WrapMode::Repeat);
}
