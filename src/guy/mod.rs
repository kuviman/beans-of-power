use super::*;

mod draw;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Input {
    pub roll_direction: f32, // -1 to +1
    pub force_fart: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomizationOptions {
    pub name: String,
    pub colors: GuyColors,
}

impl CustomizationOptions {
    pub fn random() -> Self {
        Self {
            name: "".to_owned(),
            colors: GuyColors::random(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Progress {
    pub finished: bool,
    pub current: f32,
    pub best: f32,
    pub best_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ball {
    pub radius: f32,
    pub pos: vec2<f32>,
    pub vel: vec2<f32>,
    pub rot: f32,
    pub w: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FartState {
    pub long_farting: bool,
    pub fart_pressure: f32,
}

impl Default for FartState {
    fn default() -> Self {
        Self {
            long_farting: false,
            fart_pressure: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuyAnimationState {
    pub growl_progress: Option<f32>,
    pub next_farticle_time: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, HasId)]
pub struct Guy {
    pub id: Id,
    pub customization: CustomizationOptions,
    pub ball: Ball,
    pub fart_state: FartState,
    pub input: Input,
    pub animation: GuyAnimationState,
    pub progress: Progress,

    pub touched_a_unicorn: bool,
    pub snow_layer: f32,
    pub cannon_timer: Option<CannonTimer>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CannonTimer {
    pub cannon_index: usize,
    pub time: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuyColors {
    pub top: Rgba<f32>,
    pub bottom: Rgba<f32>,
    pub hair: Rgba<f32>,
    pub skin: Rgba<f32>,
}

impl GuyColors {
    pub fn random() -> Self {
        let random_hue = || {
            let hue = thread_rng().gen_range(0.0..1.0);
            Hsva::new(hue, 1.0, 1.0, 1.0).into()
        };
        Self {
            top: random_hue(),
            bottom: random_hue(),
            hair: random_hue(),
            skin: {
                let tone = thread_rng().gen_range(0.5..1.0);
                Rgba::new(tone, tone, tone, 1.0)
            },
        }
    }
}

impl Guy {
    pub fn new(id: Id, pos: vec2<f32>, rng: bool, config: &Config) -> Self {
        Self {
            id,
            customization: CustomizationOptions::random(),
            ball: Ball {
                radius: config.guy_radius,
                pos: pos
                    + if rng {
                        vec2(thread_rng().gen_range(-1.0..=1.0), 0.0)
                    } else {
                        vec2::ZERO
                    },
                vel: vec2::ZERO,
                rot: if rng {
                    thread_rng().gen_range(-1.0..=1.0)
                } else {
                    0.0
                },
                w: 0.0,
            },
            snow_layer: 0.0,
            fart_state: default(),
            input: Input::default(),
            progress: Progress {
                finished: false,
                current: 0.0,
                best: 0.0,
                best_time: None,
            },
            animation: GuyAnimationState {
                growl_progress: None,
                next_farticle_time: 0.0,
            },

            touched_a_unicorn: false,
            cannon_timer: None,
        }
    }

    pub fn radius(&self) -> f32 {
        self.ball.radius + self.snow_layer
    }

    pub fn mass(&self, config: &Config) -> f32 {
        1.0 + self.snow_layer * config.snow_density
    }
}

#[derive(geng::Assets)]
pub struct CustomGuyAssets {
    pub body: Texture,
    pub eyes: Texture,
    pub closed_eyes: Texture,
    pub cheeks: Texture,
}

fn load_custom_guy_assets(
    geng: &Geng,
    path: &std::path::Path,
) -> geng::AssetFuture<HashMap<String, CustomGuyAssets>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("_list.json")).await?;
        let list: Vec<String> = serde_json::from_str(&json).unwrap();
        future::join_all(list.into_iter().map(|name| {
            let geng = geng.clone();
            let path = path.clone();
            async move {
                let assets = geng::LoadAsset::load(&geng, &path.join(&name)).await?;
                Ok((name.to_uppercase(), assets))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<_, anyhow::Error>>()
    }
    .boxed_local()
}

#[derive(geng::Assets)]
pub struct GuyAssets {
    pub cheeks: Texture,
    pub eyes: Texture,
    pub closed_eyes: Texture,
    pub skin: Texture,
    pub growl_top: Texture,
    pub growl_bottom: Texture,
    pub clothes_top: Texture,
    pub clothes_bottom: Texture,
    pub hair: Texture,
    #[asset(load_with = "load_custom_guy_assets(&geng, &base_path.join(\"custom\"))")]
    pub custom: HashMap<String, CustomGuyAssets>,
}
