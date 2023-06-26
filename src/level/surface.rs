use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Surface {
    pub p1: vec2<f32>,
    pub p2: vec2<f32>,
    #[serde(default)]
    pub flow: f32,
    pub type_name: String,
}

impl Surface {
    pub fn normal(&self) -> vec2<f32> {
        (self.p2 - self.p1).rotate_90().normalize_or_zero()
    }
    pub fn vector_from(&self, point: vec2<f32>) -> vec2<f32> {
        if vec2::dot(point - self.p1, self.p2 - self.p1) < 0.0 {
            return self.p1 - point;
        }
        if vec2::dot(point - self.p2, self.p1 - self.p2) < 0.0 {
            return self.p2 - point;
        }
        let n = (self.p2 - self.p1).rotate_90();
        // dot(point + n * t - p1, n) = 0
        // dot(point - p1, n) + dot(n, n) * t = 0
        let t = vec2::dot(self.p1 - point, n) / vec2::dot(n, n);
        n * t
    }
}

#[derive(geng::asset::Load, Deserialize, Debug)]
#[load(serde = "json")]
pub struct SurfaceParams {
    #[serde(default)]
    pub non_collidable: bool,
    pub bounciness: f32,
    #[serde(default)]
    pub min_bounce_vel: f32,
    pub friction: f32,
    #[serde(default)]
    pub speed_friction: f32,
    #[serde(default)]
    pub rotation_friction: f32,
    pub front: bool,
    pub back: bool,
    pub sound: bool,
    #[serde(default)]
    pub flex_frequency: f32,
    #[serde(default)]
    pub flex_amplitude: f32,
    #[serde(default)]
    pub texture_speed: f32,
    #[serde(default)]
    pub stick_strength: f32,
    #[serde(default)]
    pub max_stick_force: f32,
    pub fallthrough_speed: Option<f32>,
    #[serde(default = "default_snow_falloff")]
    pub snow_falloff: f32,
    #[serde(default)]
    pub svg: bool,
    #[serde(default)]
    pub texture_underground: f32,
}

fn default_snow_falloff() -> f32 {
    1.0
}

#[derive(geng::asset::Load)]
#[load(sequential)]
pub struct SurfaceAssets {
    pub params: SurfaceParams,
    #[load(load_with = "params.load_textures(&manager, &base_path)")]
    pub textures: SurfaceTextures,
    #[load(if = "params.sound")]
    pub sound: Option<geng::Sound>,
}

pub struct SurfaceTextures {
    pub front: Option<Texture>,
    pub back: Option<Texture>,
}

impl SurfaceParams {
    async fn load_textures(
        &self,
        manager: &geng::asset::Manager,
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<SurfaceTextures> {
        let path = path.as_ref();
        if self.svg {
            let svg = svg::load(path.join("texture.svg")).await?;
            let node_texture = |id: &str| -> Option<Texture> {
                svg.tree.node_by_id(id).map(|node| {
                    let mut texture = svg::render(manager.ugli(), &svg.tree, Some(&node));
                    texture.set_wrap_mode_separate(ugli::WrapMode::Repeat, ugli::WrapMode::Clamp);
                    // TODO: instead do premultiplied alpha
                    texture.set_filter(ugli::Filter::Nearest);
                    texture.into()
                })
            };
            Ok(SurfaceTextures {
                front: node_texture("front"),
                back: node_texture("back"),
            })
        } else {
            let load = |path| async {
                let mut texture: Texture = manager.load(path).await?;
                texture.set_wrap_mode_separate(ugli::WrapMode::Repeat, ugli::WrapMode::Clamp);
                Ok::<_, anyhow::Error>(texture)
            };
            Ok(SurfaceTextures {
                front: if self.front {
                    Some(load(path.join("front.png")).await?)
                } else {
                    None
                },
                back: if self.back {
                    Some(load(path.join("back.png")).await?)
                } else {
                    None
                },
            })
        }
    }
}
