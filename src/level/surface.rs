use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Surface {
    pub p1: Vec2<f32>,
    pub p2: Vec2<f32>,
    pub type_name: String,
}

impl Surface {
    pub fn vector_from(&self, point: Vec2<f32>) -> Vec2<f32> {
        if Vec2::dot(point - self.p1, self.p2 - self.p1) < 0.0 {
            return self.p1 - point;
        }
        if Vec2::dot(point - self.p2, self.p1 - self.p2) < 0.0 {
            return self.p2 - point;
        }
        let n = (self.p2 - self.p1).rotate_90();
        // dot(point + n * t - p1, n) = 0
        // dot(point - p1, n) + dot(n, n) * t = 0
        let t = Vec2::dot(self.p1 - point, n) / Vec2::dot(n, n);
        n * t
    }
}

#[derive(Deserialize, Debug)]
pub struct SurfaceParams {
    #[serde(default)]
    pub non_collidable: bool,
    pub bounciness: f32,
    #[serde(default)]
    pub min_bounce_vel: f32,
    pub friction: f32,
    pub front: bool,
    pub back: bool,
    pub sound: bool,
    #[serde(default)]
    pub flex_frequency: f32,
    #[serde(default)]
    pub flex_amplitude: f32,
    #[serde(default)]
    pub texture_speed: f32,
}

pub struct SurfaceAssets {
    pub name: String,
    pub params: SurfaceParams,
    pub front_texture: Option<Texture>,
    pub back_texture: Option<Texture>,
    pub sound: Option<geng::Sound>,
}

pub fn load_surface_assets(
    geng: &Geng,
    path: &std::path::Path,
) -> geng::AssetFuture<HashMap<String, SurfaceAssets>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("config.json")).await?;
        let config: std::collections::BTreeMap<String, SurfaceParams> =
            serde_json::from_str(&json).unwrap();
        future::join_all(config.into_iter().map(|(name, params)| {
            let geng = geng.clone();
            let path = path.clone();
            async move {
                let load = |file| {
                    let geng = geng.clone();
                    let path = path.clone();
                    async move {
                        let mut texture =
                            <Texture as geng::LoadAsset>::load(&geng, &path.join(file)).await?;
                        texture.0.set_wrap_mode(ugli::WrapMode::Repeat);
                        Ok::<_, anyhow::Error>(texture)
                    }
                };
                let back_texture = if params.back {
                    Some(load(format!("{}_back.png", name)).await?)
                } else {
                    None
                };
                let front_texture = if params.front {
                    Some(load(format!("{}_front.png", name)).await?)
                } else {
                    None
                };
                let sound = if params.sound {
                    Some(geng::LoadAsset::load(&geng, &path.join(format!("{}.wav", name))).await?)
                } else {
                    None
                };
                Ok((
                    name.clone(),
                    SurfaceAssets {
                        name,
                        params,
                        front_texture,
                        back_texture,
                        sound,
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
