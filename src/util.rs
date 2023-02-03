use super::*;

pub use noise::NoiseFn;

pub const EPS: f32 = 1e-9;

pub type Id = i32;

#[derive(Deref)]
pub struct Texture(#[deref] pub ugli::Texture);

impl std::borrow::Borrow<ugli::Texture> for Texture {
    fn borrow(&self) -> &ugli::Texture {
        &self.0
    }
}
impl std::borrow::Borrow<ugli::Texture> for &'_ Texture {
    fn borrow(&self) -> &ugli::Texture {
        &self.0
    }
}

impl geng::LoadAsset for Texture {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let texture = <ugli::Texture as geng::LoadAsset>::load(geng, path);
        async move {
            let mut texture = texture.await?;
            texture.set_filter(ugli::Filter::Nearest);
            Ok(Texture(texture))
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("png");
}

pub fn zero_vec() -> vec2<f32> {
    vec2::ZERO
}

impl Game {
    #[track_caller]
    pub fn noise(&self, frequency: f32) -> f32 {
        let caller = std::panic::Location::caller();
        let phase = caller.line() as f64 * 1000.0 + caller.column() as f64;
        self.noise.get([(self.real_time * frequency) as f64, phase]) as f32
    }
}

pub fn random_hue() -> Rgba<f32> {
    let hue = thread_rng().gen_range(0.0..1.0);
    Hsva::new(hue, 1.0, 1.0, 1.0).into()
}
