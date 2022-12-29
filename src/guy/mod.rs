use super::*;

mod draw;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Input {
    pub roll_direction: f32, // -1 to +1
    pub force_fart: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, HasId)]
pub struct Guy {
    pub name: String,
    pub colliding_water: bool,
    pub growl_progress: Option<f32>,
    pub id: Id,
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub rot: f32,
    pub w: f32,
    pub farting: bool,
    pub input: Input,
    pub fart_pressure: f32,
    pub finished: bool,
    pub colors: GuyColors,
    pub progress: f32,
    pub best_progress: f32,
    pub best_time: Option<f32>,
    pub next_farticle: f32,
    pub touched_a_unicorn: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuyColors {
    pub top: Rgba<f32>,
    pub bottom: Rgba<f32>,
    pub hair: Rgba<f32>,
    pub skin: Rgba<f32>,
}

impl Guy {
    pub fn new(id: Id, pos: Vec2<f32>, rng: bool) -> Self {
        let random_hue = || {
            let hue = thread_rng().gen_range(0.0..1.0);
            Hsva::new(hue, 1.0, 1.0, 1.0).into()
        };
        Self {
            farting: false,
            growl_progress: None,
            colliding_water: false,
            name: "".to_owned(),
            id,
            pos: pos
                + if rng {
                    vec2(thread_rng().gen_range(-1.0..=1.0), 0.0)
                } else {
                    Vec2::ZERO
                },
            vel: Vec2::ZERO,
            rot: if rng {
                thread_rng().gen_range(-1.0..=1.0)
            } else {
                0.0
            },
            w: 0.0,
            input: Input::default(),
            fart_pressure: 0.0,
            finished: false,
            colors: GuyColors {
                top: random_hue(),
                bottom: random_hue(),
                hair: random_hue(),
                skin: {
                    let tone = thread_rng().gen_range(0.5..1.0);
                    Rgba::new(tone, tone, tone, 1.0)
                },
            },
            progress: 0.0,
            best_progress: 0.0,
            best_time: None,
            touched_a_unicorn: false,
            next_farticle: 0.0,
        }
    }
}

#[derive(geng::Assets)]
pub struct CustomGuyAssets {
    pub body: Texture,
    pub eyes: Texture,
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
