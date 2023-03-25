use super::*;

pub use noise::NoiseFn;
pub use std::collections::VecDeque;

pub const EPS: f32 = 1e-9;

pub type Id = i32;

#[derive(Deref, DerefMut)]
pub struct Texture(#[deref] ugli::Texture);

impl From<ugli::Texture> for Texture {
    fn from(texture: ugli::Texture) -> Self {
        Self(texture)
    }
}

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
        if path.extension() == Some("svg".as_ref()) {
            let geng = geng.clone();
            let path = path.to_owned();
            async move {
                let svg = svg::load(path).await?;
                Ok(Texture(svg::render(&geng, &svg.tree, None)))
            }
            .boxed_local()
        } else {
            let texture = <ugli::Texture as geng::LoadAsset>::load(geng, path);
            async move {
                let mut texture = texture.await?;
                texture.set_filter(ugli::Filter::Nearest);
                Ok(Texture(texture))
            }
            .boxed_local()
        }
    }

    const DEFAULT_EXT: Option<&'static str> = Some("png");
}

pub fn zero_vec() -> vec2<f32> {
    vec2::ZERO
}

impl Game {
    #[track_caller]
    pub fn noise(&self, phase: f32, frequency: f32) -> f32 {
        let caller = std::panic::Location::caller();
        let phase = caller.line() as f64 * 1000.0 + caller.column() as f64 + phase as f64;
        self.noise.get([(self.real_time * frequency) as f64, phase]) as f32
    }
}

pub fn random_hue() -> Rgba<f32> {
    let hue = thread_rng().gen_range(0.0..1.0);
    Hsva::new(hue, 1.0, 1.0, 1.0).into()
}

pub fn inside_triangle(p: vec2<f32>, tri: [vec2<f32>; 3]) -> bool {
    for i in 0..3 {
        let p1 = tri[i];
        let p2 = tri[(i + 1) % 3];
        if vec2::skew(p2 - p1, p - p1) < 0.0 {
            return false;
        }
    }
    true
}

pub fn circle_triangle_intersect_percentage(
    center: vec2<f32>,
    radius: f32,
    tri: [vec2<f32>; 3],
) -> f32 {
    static RNG: once_cell::sync::Lazy<Vec<vec2<f32>>> = once_cell::sync::Lazy::new(|| {
        (0..100)
            .map(|_| thread_rng().gen_circle(vec2::ZERO, 1.0))
            .collect()
    });
    RNG.iter()
        .filter(|&&p| inside_triangle(center + p * radius, tri))
        .count() as f32
        / RNG.len() as f32
}

pub fn circle_triangle_intersect_area(center: vec2<f32>, radius: f32, tri: [vec2<f32>; 3]) -> f32 {
    fn circle_point_area(r: f32, p: vec2<f32>) -> f32 {
        let cos = p.x / r;
        let sin = (1.0 - cos.sqr()).max(0.0).sqrt();
        let tri = cos * sin * r.sqr() / 2.0;

        let big_a = f32::atan2(p.y, p.x);
        let smol_a = f32::acos(p.x / r) * p.y.signum();
        let a = big_a - smol_a;
        let arc = a * r.sqr() / 2.0;

        tri + arc
    }
    fn circle_segment_area(r: f32, ps: [vec2<f32>; 2]) -> f32 {
        let v = (ps[1] - ps[0]).normalize_or_zero();
        let s = |p| circle_point_area(r, vec2(vec2::skew(v, p), vec2::dot(v, p)));
        s(ps[0]) + s(ps[1])
    }
    let mut sum = 0.0;
    for i in 0..3 {
        let a = tri[i] - center;
        let b = tri[(i + 1) % 3] - center;
        sum += circle_segment_area(radius, [a, b]);
    }
    sum
}

pub fn ray_hit_time(
    ray_start: vec2<f32>,
    ray_vel: vec2<f32>,
    line_p: vec2<f32>,
    line_normal: vec2<f32>,
) -> f32 {
    // dot(ray_start + ray_vel * t - line_p, line_normal) = 0
    vec2::dot(line_p - ray_start, line_normal) / vec2::dot(ray_vel, line_normal)
}
