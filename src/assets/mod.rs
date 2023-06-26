use super::*;

mod listed;
mod texture;

pub use listed::*;
pub use texture::*;

pub type AssetsHandle = Rc<Hot<Assets>>;

#[derive(geng::asset::Load)]
pub struct Assets {
    pub config: Rc<Config>,
    pub sfx: SfxAssets,
    pub farts: Listed<Rc<FartAssets>>,
    pub guy: GuyAssets,
    pub surfaces: Listed<SurfaceAssets>,
    pub tiles: Listed<TileAssets>,
    #[load(
        load_with = "Listed::load_with_ext(&manager, &base_path.join(\"objects\"), Some(\"svg\"))"
    )]
    pub objects: Listed<Texture>,
    #[load(load_with = "load_font(&manager, &base_path.join(\"Ludum-Dairy-0.2.0.ttf\"))")]
    pub font: geng::Font,
    #[load(ext = "svg")]
    pub closed_outhouse: Texture,
    #[load(ext = "svg")]
    pub golden_toilet: Texture,
    #[load(listed_in = "_list.ron")]
    pub emotes: Vec<Texture>,
    pub shaders: Shaders,
    pub cannon: features::cannon::Assets,
    pub portal: Texture,
    pub bubble: Texture,
    #[load(ext = "svg")]
    pub arrow_key: Texture,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub tile: ugli::Program,
    pub surface: ugli::Program,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PortalConfig {
    pub size: f32,
}

#[derive(geng::asset::Load, Deserialize, Clone, Debug)]
#[load(serde = "json")]
pub struct Config {
    pub volume: f32,
    pub sfx_time_scale_power: f64,
    pub snap_distance: f32,
    pub guy_radius: f32,
    pub growl_time: f32,
    pub angular_acceleration: f32,
    pub gravity: f32,
    pub max_angular_speed: f32, // TODO: maybe?

    pub default_fart_type: String,

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
    pub portal: PortalConfig,
    pub stick_force_fadeout_speed: f32,
    pub max_penetration: f32,
    pub bubble_time: f32,
    pub bubble_scale: f32,
    pub bubble_acceleration: f32,
    pub bubble_target_speed: f32,
    pub camera_fov: f32,

    pub cannon: features::cannon::Config,
}

#[derive(geng::asset::Load)]
pub struct SfxAssets {
    pub fart_recharge: geng::Sound,
    pub water_splash: geng::Sound,
    #[load(path = "music.mp3", postprocess = "make_looped")]
    pub old_music: geng::Sound,
    #[load(path = "KuviFart.wav", postprocess = "make_looped")]
    pub new_music: geng::Sound,
}

fn make_looped(sound: &mut geng::Sound) {
    sound.set_looped(true);
}

fn load_font(
    manager: &geng::asset::Manager,
    path: &std::path::Path,
) -> geng::asset::Future<geng::Font> {
    let manager = manager.clone();
    let path = path.to_owned();
    async move {
        let data: Vec<u8> = manager.load(&path).await?;
        geng::Font::new(
            manager.ugli(),
            &data,
            geng::font::Options {
                pixel_size: 64.0,
                max_distance: 0.1,
                antialias: true,
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

#[derive(geng::asset::Load, Serialize, Deserialize)]
#[load(serde = "json")]
pub struct FartConfig {
    #[serde(default = "one")]
    pub sfx_count: usize,
    pub colors: FartColors,

    #[serde(default = "create_true")]
    pub long_fart_sfx: bool,
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

#[derive(geng::asset::Load)]
#[load(sequential)]
pub struct FartAssets {
    pub config: FartConfig,
    #[load(path = "farticle.png")]
    pub farticle_texture: Texture,
    #[load(path = "sfx*.wav", list = "1..=config.sfx_count")]
    pub sfx: Vec<geng::Sound>,
    #[load(load_with = "load_long_sfx(&config, &manager, &base_path)")]
    pub long_sfx: Option<geng::Sound>,
}

async fn load_long_sfx(
    config: &FartConfig,
    manager: &geng::asset::Manager,
    base_path: &std::path::Path,
) -> anyhow::Result<Option<geng::Sound>> {
    Ok(if config.long_fart_sfx {
        Some(manager.load(base_path.join("long_sfx.wav")).await?)
    } else {
        None
    })
}
