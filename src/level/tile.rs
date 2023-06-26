use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tile {
    pub vertices: [vec2<f32>; 3],
    #[serde(default = "zero_vec")]
    pub flow: vec2<f32>,
    pub type_name: String,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "json")]
pub struct TileParams {
    #[serde(default)]
    pub svg: bool,
    #[serde(default = "one")]
    pub texture_scale: f32,
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
    #[serde(default = "default_draw_times")]
    pub draw_times: usize,
    #[serde(default)]
    pub fadeout_distance: f32,
    #[serde(default)]
    pub texture_rotation: f32,
}

fn default_draw_times() -> usize {
    1
}

fn one() -> f32 {
    1.0
}

#[derive(geng::asset::Load)]
#[load(sequential)]
pub struct TileAssets {
    pub params: TileParams,
    #[load(load_with = "load_tile_texture(&manager, &base_path, &params)")]
    pub texture: Texture,
}

async fn load_tile_texture(
    manager: &geng::asset::Manager,
    base_path: &std::path::Path,
    params: &TileParams,
) -> anyhow::Result<Texture> {
    let mut texture: Texture = manager
        .load(
            base_path
                .join("texture")
                .with_extension(if params.svg { "svg" } else { "png" }),
        )
        .await?;
    texture.set_filter(ugli::Filter::Nearest); // TODO premultiplied alpha instead
    make_repeated(&mut texture);
    Ok(texture)
}

fn make_repeated(texture: &mut Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
}
