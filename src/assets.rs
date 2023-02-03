use super::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub config: Rc<Config>,
    pub sfx: SfxAssets,
    pub guy: GuyAssets,
    #[asset(load_with = "load_surface_assets(&geng, &base_path.join(\"surfaces\"))")]
    pub surfaces: HashMap<String, SurfaceAssets>,
    #[asset(load_with = "load_tile_assets(&geng, &base_path.join(\"tiles\"))")]
    pub tiles: HashMap<String, TileAssets>,
    #[asset(load_with = "load_objects_assets(&geng, &base_path.join(\"objects\"))")]
    pub objects: HashMap<String, Texture>,
    pub farticle: Texture,
    #[asset(load_with = "load_font(&geng, &base_path.join(\"Ludum-Dairy-0.2.0.ttf\"))")]
    pub font: geng::Font,
    pub closed_outhouse: Texture,
    pub golden_toilet: Texture,
    #[asset(
        range = "[\"poggers\", \"fuuuu\", \"kekw\", \"eesBoom\"].into_iter()",
        path = "emotes/*.png"
    )]
    pub emotes: Vec<Texture>,
    pub shaders: Shaders,
    pub cannon: CannonAssets,
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
    pub particle_color: Rgba<f32>,
    pub particle_speed: f32,
}

#[derive(geng::Assets, Deserialize, Clone, Debug)]
#[asset(json)]
pub struct Config {
    pub volume: f32,
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
    pub long_fart_farticles_per_second: f32,
    pub long_fart_farticle_speed: f32,
    pub fart_strength: f32,
    pub max_fart_pressure: f32,
    pub fart_pressure_released: f32,
    pub fart_color: Rgba<f32>,
    pub bubble_fart_color: Rgba<f32>,
    pub farticle_w: f32,
    pub farticle_size: f32,
    pub farticle_count: usize,
    pub farticle_additional_vel: f32,
    pub background_color: Rgba<f32>,
    pub max_snow_layer: f32,
    pub snow_falloff_impulse_min: f32,
    pub snow_falloff_impulse_max: f32,
    pub snow_density: f32,
    pub cannon: CannonConfig,
}

#[derive(geng::Assets)]
pub struct SfxAssets {
    #[asset(range = "1..=3", path = "fart/*.wav")]
    pub fart: Vec<geng::Sound>,
    #[asset(range = "1..=1", path = "bubble_fart/*.wav")]
    pub bubble_fart: Vec<geng::Sound>,
    #[asset(range = "1..=1", path = "rainbow_fart/*.wav")]
    pub rainbow_fart: Vec<geng::Sound>,
    #[asset(path = "fart/long.wav")]
    pub long_fart: geng::Sound,
    #[asset(path = "bubble_fart/long.wav")]
    pub bubble_long_fart: geng::Sound,
    #[asset(path = "rainbow_fart/long.wav")]
    pub rainbow_long_fart: geng::Sound,
    pub fart_recharge: geng::Sound,
    pub water_splash: geng::Sound,
    #[asset(path = "music.mp3")]
    pub old_music: geng::Sound,
    #[asset(path = "KuviFart.wav")]
    pub new_music: geng::Sound,
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

impl Assets {
    pub fn process(&mut self) {
        self.sfx.old_music.looped = true;
        self.sfx.new_music.looped = true;
    }
}
