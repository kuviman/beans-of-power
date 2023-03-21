use super::*;

mod listed;

pub use listed::*;

pub type AssetsHandle = Rc<Hot<Assets>>;

#[derive(geng::Assets)]
pub struct Assets {
    pub config: Rc<Config>,
    pub sfx: SfxAssets,
    pub farts: Listed<FartAssets>,
    pub guy: GuyAssets,
    pub surfaces: Listed<SurfaceAssets>,
    pub tiles: Listed<TileAssets>,
    #[asset(
        load_with = "Listed::load_with_ext(&geng, &base_path.join(\"objects\"), Some(\"svg\"))"
    )]
    pub objects: Listed<Texture>,
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
    #[asset(ext = "svg")]
    pub arrow_key: Texture,
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

#[derive(geng::Assets)]
#[asset(sequential)]
pub struct FartAssets {
    pub config: FartConfig,
    #[asset(path = "farticle.png")]
    pub farticle_texture: Texture,
    #[asset(path = "sfx*.wav", list = "1..=config.sfx_count")]
    pub sfx: Vec<geng::Sound>,
    pub long_sfx: geng::Sound,
}
