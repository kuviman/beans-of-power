use super::*;

impl Game {
    pub fn draw_guys(&self, framebuffer: &mut ugli::Framebuffer) {
        for guy in itertools::chain![
            self.guys.iter().filter(|guy| guy.id != self.client_id),
            self.guys.iter().filter(|guy| guy.id == self.client_id),
        ] {
            if let Some(growl_progress) = guy.growl_progress {
                let shift = vec2(self.noise(10.0), self.noise(10.0)) * 0.1;
                let scale = 1.0 - (growl_progress * 2.0 - 1.0).sqr();
                let scale =
                    self.config.growl_min_scale * (1.0 - scale) + self.config.growl_scale * scale;
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(
                        &self.assets.guy.growl_bottom,
                        guy.colors.bottom,
                    )
                    .translate(shift)
                    .scale_uniform(self.config.guy_radius * scale)
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(
                        &self.assets.guy.growl_top,
                        guy.colors.top,
                    )
                    .translate(shift)
                    .scale_uniform(self.config.guy_radius * scale)
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
                );
            }

            let (eyes, closed_eyes, cheeks, cheeks_color) = if let Some(custom) =
                self.assets.guy.custom.get(&guy.name)
            {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&custom.body)
                        .scale_uniform(self.config.guy_radius)
                        .transform(Mat3::rotate(guy.rot))
                        .translate(guy.pos),
                );
                (
                    &custom.eyes,
                    &self.assets.guy.closed_eyes, // TODO custom
                    &custom.cheeks,
                    Rgba::WHITE,
                )
            } else {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(
                        &self.assets.guy.clothes_bottom,
                        guy.colors.bottom,
                    )
                    .scale_uniform(self.config.guy_radius)
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(
                        &self.assets.guy.clothes_top,
                        guy.colors.top,
                    )
                    .scale_uniform(self.config.guy_radius)
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(&self.assets.guy.hair, guy.colors.hair)
                        .scale_uniform(self.config.guy_radius)
                        .transform(Mat3::rotate(guy.rot))
                        .translate(guy.pos),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(&self.assets.guy.skin, guy.colors.skin)
                        .scale_uniform(self.config.guy_radius)
                        .transform(Mat3::rotate(guy.rot))
                        .translate(guy.pos),
                );
                (
                    &self.assets.guy.eyes,
                    &self.assets.guy.closed_eyes,
                    &self.assets.guy.cheeks,
                    guy.colors.skin,
                )
            };
            let fart_progress = guy.fart_pressure / self.config.max_fart_pressure;

            if false {
                // Visualize fart pressure
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Quad::new(
                        AABB::point(guy.pos + vec2(0.0, self.config.guy_radius * 1.2))
                            .extend_symmetric(vec2(0.5, 0.02)),
                        Rgba::BLACK,
                    ),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Quad::new(
                        AABB::point(
                            guy.pos + vec2(-0.5 + fart_progress, self.config.guy_radius * 1.2),
                        )
                        .extend_uniform(0.04),
                        Rgba::BLACK,
                    ),
                );
            }

            if guy.input.force_fart {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(closed_eyes, guy.colors.skin)
                        .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * fart_progress)
                        .scale_uniform(self.config.guy_radius * (0.8 + 0.6 * fart_progress))
                        .transform(Mat3::rotate(guy.rot))
                        .translate(guy.pos),
                );
            } else {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(eyes, {
                        let k = 0.8;
                        let t = ((fart_progress - k) / (1.0 - k)).clamp(0.0, 1.0) * 0.5;
                        Rgba::new(1.0, 1.0 - t, 1.0 - t, 1.0)
                    })
                    .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * fart_progress)
                    .scale_uniform(self.config.guy_radius * (0.8 + 0.6 * fart_progress))
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
                );
            }
            if guy.fart_pressure >= self.config.fart_pressure_released {
                let progress = (guy.fart_pressure - self.config.fart_pressure_released)
                    / (self.config.max_fart_pressure - self.config.fart_pressure_released);
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(
                        cheeks,
                        Rgba {
                            a: (0.5 + 1.0 * progress).min(1.0),
                            ..cheeks_color
                        },
                    )
                    .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * progress)
                    .scale_uniform(self.config.guy_radius * (0.8 + 0.7 * progress))
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
                );
            }
            if Some(guy.id) == self.my_guy || self.show_names {
                self.assets.font.draw(
                    framebuffer,
                    &self.camera,
                    &guy.name,
                    guy.pos + vec2(0.0, self.config.guy_radius * 1.1),
                    geng::TextAlign::CENTER,
                    0.1,
                    Rgba::BLACK,
                );
            }
        }

        // Emotes
        for &(_, id, emote) in &self.emotes {
            if let Some(guy) = self.guys.get(&id) {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&self.assets.emotes[emote])
                        .scale_uniform(0.1)
                        .translate(guy.pos + vec2(0.0, self.config.guy_radius * 2.0)),
                );
            }
        }
    }
}
