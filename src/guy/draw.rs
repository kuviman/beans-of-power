use super::*;

impl Game {
    pub fn draw_guys(&self, framebuffer: &mut ugli::Framebuffer) {
        for guy in itertools::chain![
            self.guys.iter().filter(|guy| guy.id != self.client_id),
            self.guys.iter().filter(|guy| guy.id == self.client_id),
        ] {
            if guy.snow_layer != 0.0 {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(guy.ball.pos, guy.radius(), Rgba::WHITE),
                );
            }

            if self
                .assets
                .guy
                .custom
                .get(&guy.customization.name)
                .is_none()
            {
                if let Some(growl_progress) = guy.animation.growl_progress {
                    let shift = vec2(self.noise(10.0), self.noise(10.0)) * 0.1;
                    let scale = 1.0 - (growl_progress * 2.0 - 1.0).sqr();
                    let scale = self.config.growl_min_scale * (1.0 - scale)
                        + self.config.growl_scale * scale;
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit_colored(
                            &self.assets.guy.growl_bottom,
                            guy.customization.colors.bottom,
                        )
                        .translate(shift)
                        .scale_uniform(guy.ball.radius * scale)
                        .transform(mat3::rotate(guy.ball.rot))
                        .translate(guy.ball.pos),
                    );
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit_colored(
                            &self.assets.guy.growl_top,
                            guy.customization.colors.top,
                        )
                        .translate(shift)
                        .scale_uniform(guy.ball.radius * scale)
                        .transform(mat3::rotate(guy.ball.rot))
                        .translate(guy.ball.pos),
                    );
                }
            }

            let (eyes, closed_eyes, cheeks, cheeks_color) =
                if let Some(custom) = self.assets.guy.custom.get(&guy.customization.name) {
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit(&custom.body)
                            .scale_uniform(guy.ball.radius)
                            .transform(mat3::rotate(guy.ball.rot))
                            .translate(guy.ball.pos),
                    );
                    (
                        &custom.eyes,
                        &custom.closed_eyes,
                        &custom.cheeks,
                        Rgba::WHITE,
                    )
                } else {
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit_colored(
                            &self.assets.guy.clothes_bottom,
                            guy.customization.colors.bottom,
                        )
                        .scale_uniform(guy.ball.radius)
                        .transform(mat3::rotate(guy.ball.rot))
                        .translate(guy.ball.pos),
                    );
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit_colored(
                            &self.assets.guy.clothes_top,
                            guy.customization.colors.top,
                        )
                        .scale_uniform(guy.ball.radius)
                        .transform(mat3::rotate(guy.ball.rot))
                        .translate(guy.ball.pos),
                    );
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit_colored(
                            &self.assets.guy.hair,
                            guy.customization.colors.hair,
                        )
                        .scale_uniform(guy.ball.radius)
                        .transform(mat3::rotate(guy.ball.rot))
                        .translate(guy.ball.pos),
                    );
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit_colored(
                            &self.assets.guy.skin,
                            guy.customization.colors.skin,
                        )
                        .scale_uniform(guy.ball.radius)
                        .transform(mat3::rotate(guy.ball.rot))
                        .translate(guy.ball.pos),
                    );
                    (
                        &self.assets.guy.eyes,
                        &self.assets.guy.closed_eyes,
                        &self.assets.guy.cheeks,
                        guy.customization.colors.skin,
                    )
                };
            let fart_progress = guy.fart_state.fart_pressure / self.config.max_fart_pressure;

            if false {
                // Visualize fart pressure
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Quad::new(
                        Aabb2::point(guy.ball.pos + vec2(0.0, guy.ball.radius * 1.2))
                            .extend_symmetric(vec2(0.5, 0.02)),
                        Rgba::BLACK,
                    ),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Quad::new(
                        Aabb2::point(
                            guy.ball.pos + vec2(-0.5 + fart_progress, guy.ball.radius * 1.2),
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
                    &draw_2d::TexturedQuad::unit_colored(
                        closed_eyes,
                        guy.customization.colors.skin,
                    )
                    .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * fart_progress)
                    .scale_uniform(guy.ball.radius * (0.8 + 0.6 * fart_progress))
                    .transform(mat3::rotate(guy.ball.rot))
                    .translate(guy.ball.pos),
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
                    .scale_uniform(guy.ball.radius * (0.8 + 0.6 * fart_progress))
                    .transform(mat3::rotate(guy.ball.rot))
                    .translate(guy.ball.pos),
                );
            }
            if guy.fart_state.fart_pressure >= self.config.fart_pressure_released {
                let progress = (guy.fart_state.fart_pressure - self.config.fart_pressure_released)
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
                    .scale_uniform(guy.ball.radius * (0.8 + 0.7 * progress))
                    .transform(mat3::rotate(guy.ball.rot))
                    .translate(guy.ball.pos),
                );
            }
            if Some(guy.id) == self.my_guy || self.show_names {
                self.assets.font.draw(
                    framebuffer,
                    &self.camera,
                    &guy.customization.name,
                    guy.ball.pos + vec2(0.0, guy.ball.radius * 1.1),
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
                        .translate(guy.ball.pos + vec2(0.0, guy.ball.radius * 2.0)),
                );
            }
        }
    }
}
