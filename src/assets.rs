use super::*;

pub type AssetsHandle = Rc<Hot<Assets>>;

#[derive(geng::Assets)]
pub struct Assets {
    pub config: Rc<Config>,
    pub sfx: SfxAssets,
    #[asset(load_with = "load_fart_assets(&geng, &base_path.join(\"farts\"))")]
    pub farts: HashMap<String, FartAssets>,
    pub guy: GuyAssets,
    #[asset(load_with = "load_surface_assets(&geng, &base_path.join(\"surfaces\"))")]
    pub surfaces: HashMap<String, SurfaceAssets>,
    #[asset(load_with = "load_tile_assets(&geng, &base_path.join(\"tiles\"))")]
    pub tiles: HashMap<String, TileAssets>,
    #[asset(load_with = "load_objects_assets(&geng, &base_path.join(\"objects\"))")]
    pub objects: HashMap<String, Texture>,
    #[asset(load_with = "load_font(&geng, &base_path.join(\"Ludum-Dairy-0.2.0.ttf\"))")]
    pub font: geng::Font,
    #[asset(ext = "svg")]
    pub closed_outhouse: Texture,
    #[asset(ext = "svg")]
    pub golden_toilet: Texture,
    #[asset(listed_in = "_list.ron")]
    pub emotes: Vec<Texture>,
    pub shaders: Shaders,
    pub cannon: CannonAssets,
    pub portal: Texture,
    pub bubble: Texture,
}

#[derive(geng::Assets)]
pub struct Shaders {
    pub tile: ugli::Program,
    pub surface: ugli::Program,
}

#[derive(geng::Assets)]
pub struct CannonAssets {
    pub body: Texture,
    pub base: Texture,
    pub shot: geng::Sound,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CannonConfig {
    pub strength: f32,
    pub activate_distance: f32,
    pub shoot_time: f32,
    pub particle_size: f32,
    pub particle_count: usize,
    pub particle_colors: Rc<Vec<Rgba<f32>>>,
    pub particle_speed: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PortalConfig {
    pub size: f32,
}

#[derive(geng::Assets, Deserialize, Clone, Debug)]
#[asset(json)]
pub struct Config {
    pub volume: f32,
    pub sfx_time_scale_power: f64,
    pub snap_distance: f32,
    pub guy_radius: f32,
    pub growl_time: f32,
    pub growl_min_scale: f32,
    pub growl_scale: f32,
    pub angular_acceleration: f32,
    pub gravity: f32,
    pub max_angular_speed: f32, // TODO: maybe?

    pub fart_continued_force: f32,
    pub fart_continuation_pressure_speed: f32,
    pub force_fart_pressure_multiplier: f32,
    pub fart_strength: f32,
    pub max_fart_pressure: f32,
    pub fart_pressure_released: f32,

    pub background_color: Rgba<f32>,
    pub max_snow_layer: f32,
    pub snow_falloff_impulse_min: f32,
    pub snow_falloff_impulse_max: f32,
    pub snow_density: f32,
    pub snow_particle_colors: Rc<Vec<Rgba<f32>>>,
    pub cannon: CannonConfig,
    pub portal: PortalConfig,
    pub stick_force_fadeout_speed: f32,
    pub max_penetration: f32,
    pub bubble_time: f32,
    pub bubble_scale: f32,
    pub bubble_acceleration: f32,
    pub bubble_target_speed: f32,
    pub camera_fov: f32,
}

#[derive(geng::Assets)]
pub struct SfxAssets {
    pub fart_recharge: geng::Sound,
    pub water_splash: geng::Sound,
    #[asset(path = "music.mp3", postprocess = "make_looped")]
    pub old_music: geng::Sound,
    #[asset(path = "KuviFart.wav", postprocess = "make_looped")]
    pub new_music: geng::Sound,
}

fn make_looped(sound: &mut geng::Sound) {
    sound.looped = true;
}

fn load_font(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<geng::Font> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let data = <Vec<u8> as geng::LoadAsset>::load(&geng, &path).await?;
        geng::Font::new(
            &geng,
            &data,
            geng::ttf::Options {
                pixel_size: 64.0,
                max_distance: 0.1,
            },
        )
    }
    .boxed_local()
}

#[derive(Serialize, Deserialize)]
pub enum FartColors {
    Fixed(Rc<Vec<Rgba<f32>>>),
    RandomHue { alpha: f32 },
}

impl FartColors {
    pub fn get(&self) -> Rc<Vec<Rgba<f32>>> {
        match self {
            FartColors::Fixed(list) => list.clone(),
            FartColors::RandomHue { alpha } => Rc::new(vec![Rgba {
                a: *alpha,
                ..random_hue()
            }]),
        }
    }
}

#[derive(geng::Assets, Serialize, Deserialize)]
#[asset(json)]
pub struct FartConfig {
    #[serde(default = "one")]
    pub sfx_count: usize,
    pub colors: FartColors,

    pub long_fart_farticles_per_second: f32,
    pub long_fart_farticle_speed: f32,
    pub farticle_w: f32,
    pub farticle_size: f32,
    pub farticle_count: usize,
    pub farticle_additional_vel: f32,
    #[serde(default = "one_f32")]
    pub farticle_lifetime: f32,
    #[serde(default = "create_true")]
    pub farticle_random_rotation: bool,
}

fn one_f32() -> f32 {
    1.0
}

fn create_true() -> bool {
    true
}

fn one() -> usize {
    1
}

pub struct FartAssets {
    pub config: FartConfig,
    pub farticle_texture: Texture,
    pub sfx: Vec<geng::Sound>,
    pub long_sfx: geng::Sound,
}

pub fn load_fart_assets(
    geng: &Geng,
    path: &std::path::Path,
) -> geng::AssetFuture<HashMap<String, FartAssets>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let list: Vec<String> = file::load_json(path.join("_list.json"))
            .await
            .context("Failed to load _list.json")?;
        future::join_all(list.into_iter().map(|fart_type| async {
            let path = path.join(&fart_type);
            let config: FartConfig = file::load_json(path.join("config.json"))
                .await
                .context("Failed to load config.json")?;
            let farticle_texture = geng
                .load_asset(path.join("farticle.png"))
                .await
                .context("Failed to load farticle.png")?;
            let sfx_count = config.sfx_count;
            let sfx = future::join_all((0..sfx_count).map(|index| {
                let filename = if sfx_count == 1 {
                    "sfx.wav".to_owned()
                } else {
                    format!("sfx{}.wav", index + 1)
                };
                geng.load_asset::<geng::Sound>(path.join(&filename))
                    .map(move |result| result.context(format!("Failed to load {filename:?}")))
            }))
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<geng::Sound>>>()
            .context("Failed to load fart sfx")?;
            let long_sfx = geng
                .load_asset(path.join("long_sfx.wav"))
                .await
                .context("Failed to load long_sfx.wav")?;
            Ok((
                fart_type,
                FartAssets {
                    config,
                    farticle_texture,
                    sfx,
                    long_sfx,
                },
            ))
        }))
        .await
        .into_iter()
        .collect()
    }
    .boxed_local()
}
