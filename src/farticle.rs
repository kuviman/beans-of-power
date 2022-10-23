use super::*;

pub struct Farticle {
    pub size: f32,
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub color: Rgba<f32>,
    pub rot: f32,
    pub w: f32,
    pub t: f32,
}

impl Game {
    pub fn update_farticles(&mut self, delta_time: f32) {
        for farticle in &mut self.farticles {
            farticle.t -= delta_time;
            farticle.pos += farticle.vel * delta_time;
            farticle.rot += farticle.w * delta_time;

            let level = if self.customization.postjam {
                &self.levels.1
            } else {
                &self.levels.0
            };
            for surface in &level.surfaces {
                let v = surface.vector_from(farticle.pos);
                let penetration = self.config.farticle_size / 2.0 - v.len();
                if penetration > EPS && Vec2::dot(v, farticle.vel) > 0.0 {
                    let normal = -v.normalize_or_zero();
                    farticle.pos += normal * penetration;
                    farticle.vel -= normal * Vec2::dot(farticle.vel, normal);
                }
            }
        }
        self.farticles.retain(|farticle| farticle.t > 0.0);
    }

    pub fn draw_farticles(&self, framebuffer: &mut ugli::Framebuffer) {
        for farticle in &self.farticles {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(
                    &self.assets.farticle,
                    Rgba {
                        a: farticle.color.a * farticle.t,
                        ..farticle.color
                    },
                )
                .transform(Mat3::rotate(farticle.rot))
                .scale_uniform(self.config.farticle_size * farticle.size)
                .translate(farticle.pos),
            )
        }
    }
}
