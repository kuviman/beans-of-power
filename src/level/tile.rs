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
    pub svg: bool,
    #[serde(default = "one")]
    pub texture_scale: f32,
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

fn one() -> f32 {
    1.0
}

#[derive(geng::Assets)]
#[asset(sequential)]
pub struct TileAssets {
    pub params: TileParams,
    #[asset(load_with = "load_tile_texture(&geng, &base_path, &params)")]
    pub texture: Texture,
}

async fn load_tile_texture(
    geng: &Geng,
    base_path: &std::path::Path,
    params: &TileParams,
) -> anyhow::Result<Texture> {
    let mut texture = geng
        .load_asset(base_path.join("texture").with_extension(if params.svg {
            "svg"
        } else {
            "png"
        }))
        .await?;
    make_repeated(&mut texture);
    Ok(texture)
}

fn make_repeated(texture: &mut Texture) {
    texture.0.set_wrap_mode(ugli::WrapMode::Repeat);
}
