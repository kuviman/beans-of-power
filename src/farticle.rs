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

impl Game {
    pub fn update_farticles(&mut self, delta_time: f32) {
        let assets = self.assets.get();
        for (type_name, farticles) in &mut self.farticles {
            let fart_assets = &assets.farts[type_name];
            for farticle in &mut *farticles {
                farticle.t -= delta_time / fart_assets.config.farticle_lifetime;
                farticle.pos += farticle.vel * delta_time;
                farticle.rot += farticle.w * delta_time;

                for surface in self.level.gameplay_surfaces() {
                    let v = surface.vector_from(farticle.pos);
                    let penetration = fart_assets.config.farticle_size / 2.0 - v.len();
                    if penetration > EPS && vec2::dot(v, farticle.vel) > 0.0 {
                        let normal = -v.normalize_or_zero();
                        farticle.pos += normal * penetration;
                        farticle.vel -= normal * vec2::dot(farticle.vel, normal);
                    }
                }
            }
            farticles.retain(|farticle| farticle.t > 0.0);
        }
    }

    pub fn draw_farticles(&self, framebuffer: &mut ugli::Framebuffer) {
        let assets = self.assets.get();
        for (type_name, farticles) in &self.farticles {
            let fart_assets = &assets.farts[type_name];
            let texture = &fart_assets.farticle_texture;
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
                    &self.camera,
                    &draw2d::TexturedQuad::unit_colored(
                        texture,
                        Rgba {
                            a: color.a * farticle.t,
                            ..color
                        },
                    )
                    .transform(mat3::rotate(farticle.rot))
                    .scale_uniform(fart_assets.config.farticle_size * farticle.size)
                    .translate(farticle.pos),
                );
            }
        }
    }
}
