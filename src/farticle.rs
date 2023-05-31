use super::*;

pub struct Farticle {
    pub size: f32,
    pub pos: vec2<f32>,
    pub vel: vec2<f32>,
    pub colors: Rc<Vec<Rgba<f32>>>,
    pub rot: f32,
    pub w: f32,
    pub t: f32,
}

impl Farticle {
    pub fn new(config: &FartConfig, pos: vec2<f32>, vel: vec2<f32>) -> Self {
        Self {
            size: config.farticle_size,
            pos,
            vel: thread_rng().gen_circle(vel, config.farticle_additional_vel),
            rot: if config.farticle_random_rotation {
                thread_rng().gen_range(0.0..2.0 * f32::PI)
            } else {
                0.0
            },
            w: thread_rng().gen_range(-1.0..1.0) * config.farticle_w,
            colors: config.colors.get(),
            t: 1.0,
        }
    }
}

pub struct System {
    geng: Geng,
    farticles: HashMap<HashRc<FartAssets>, Vec<Farticle>>,
}

impl System {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            farticles: default(),
        }
    }

    pub fn spawn(&mut self, assets: &Rc<FartAssets>, pos: vec2<f32>, vel: vec2<f32>) {
        self.farticles
            .entry(HashRc::from(assets.clone()))
            .or_default()
            .extend(
                std::iter::repeat_with(|| Farticle::new(&assets.config, pos, vel))
                    .take(assets.config.farticle_count),
            );
    }

    // TODO: reduce copypasting
    pub fn spawn_single(&mut self, assets: &Rc<FartAssets>, pos: vec2<f32>, vel: vec2<f32>) {
        self.farticles
            .entry(HashRc::from(assets.clone()))
            .or_default()
            .push(Farticle::new(&assets.config, pos, vel));
    }

    pub fn push(&mut self, assets: &Rc<FartAssets>, farticle: Farticle) {
        self.farticles
            .entry(HashRc::from(assets.clone()))
            .or_default()
            .push(farticle);
    }

    pub fn update(&mut self, delta_time: f32, level: &LevelInfo) {
        for (assets, farticles) in &mut self.farticles {
            for farticle in farticles.iter_mut() {
                farticle.t -= delta_time / assets.config.farticle_lifetime;
                farticle.pos += farticle.vel * delta_time;
                farticle.rot += farticle.w * delta_time;

                for surface in level.gameplay_surfaces() {
                    let v = surface.vector_from(farticle.pos);
                    let penetration = assets.config.farticle_size / 2.0 - v.len();
                    if penetration > EPS && vec2::dot(v, farticle.vel) > 0.0 {
                        let normal = -v.normalize_or_zero();
                        farticle.pos += normal * penetration;
                        farticle.vel -= normal * vec2::dot(farticle.vel, normal);
                    }
                }
            }
            farticles.retain(|farticle| farticle.t > 0.0);
        }
        self.farticles.retain(|_, farticles| !farticles.is_empty());
    }

    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer, camera: &Camera2d) {
        for (assets, farticles) in &self.farticles {
            let texture = &assets.farticle_texture;
            // TODO: use instancing
            for farticle in farticles {
                let color = {
                    let t = (1.0 - farticle.t) * farticle.colors.len() as f32;
                    let index = (t.floor() as usize).min(farticle.colors.len() - 1);
                    let t = t.fract();
                    let color1 = farticle.colors[index];
                    let color2 = farticle.colors[(index + 1).min(farticle.colors.len() - 1)];
                    Rgba::lerp(color1, color2, t)
                };
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::TexturedQuad::unit_colored(
                        texture,
                        Rgba {
                            a: color.a * farticle.t,
                            ..color
                        },
                    )
                    .transform(mat3::rotate(farticle.rot))
                    .scale_uniform(farticle.size)
                    .translate(farticle.pos),
                );
            }
        }
    }
}
