use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tile {
    pub vertices: [vec2<f32>; 3],
    #[serde(default = "zero_vec")]
    pub flow: vec2<f32>,
    pub type_name: String,
}

#[derive(Deserialize)]
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

pub struct TileAssets {
    pub name: String,
    pub params: TileParams,
    pub texture: Texture,
}

pub fn load_tile_assets(
    geng: &Geng,
    path: &std::path::Path,
) -> geng::AssetFuture<HashMap<String, TileAssets>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("config.json")).await?;
        let config: std::collections::BTreeMap<String, TileParams> =
            serde_json::from_str(&json).unwrap();
        future::join_all(config.into_iter().map(|(name, params)| {
            let geng = geng.clone();
            let path = path.clone();
            async move {
                let mut texture =
                    <Texture as geng::LoadAsset>::load(&geng, &path.join(format!("{}.png", name)))
                        .await?;
                texture.0.set_wrap_mode(ugli::WrapMode::Repeat);
                Ok((
                    name.clone(),
                    TileAssets {
                        name,
                        params,
                        texture,
                    },
                ))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<_, anyhow::Error>>()
    }
    .boxed_local()
}
