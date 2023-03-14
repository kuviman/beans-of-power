use super::*;

impl Game {
    pub fn draw_guys(&self, framebuffer: &mut ugli::Framebuffer) {
        let assets = self.assets.get();
        for guy in itertools::chain![
            self.guys.iter().filter(|guy| guy.id != self.client_id),
            self.guys.iter().filter(|guy| guy.id == self.client_id),
        ] {
            let fart_progress = guy.fart_state.fart_pressure / self.config.max_fart_pressure;
            let eyes_color = {
                let k = 0.8;
                let t = ((fart_progress - k) / (1.0 - k)).clamp(0.0, 1.0) * 0.5;
                Rgba::new(1.0, 1.0 - t, 1.0 - t, 1.0)
            };

            if guy.snow_layer != 0.0 {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(guy.ball.pos, guy.radius(), Rgba::WHITE),
                );
            }

            let mut draw = |layers: &[GuyRenderLayer], transform: mat3<f32>, alpha: f32| {
                for layer in layers {
                    let mut color = match layer.color.as_str() {
                        "eyes" => eyes_color,
                        "clothes-top" => guy.customization.colors.top,
                        "clothes-bottom" => guy.customization.colors.bottom,
                        "skin" => guy.customization.colors.skin,
                        "hair" => guy.customization.colors.hair,
                        color => {
                            panic!("What is {color}????")
                        }
                    };
                    color.a *= alpha;
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit_colored(&layer.texture, color)
                            .transform(transform),
                    );
                }
            };

            if let Some(growl_progress) = guy.animation.growl_progress {
                let shift = vec2(self.noise(10.0), self.noise(10.0)) * 0.1;
                let scale = 1.0 - (growl_progress * 2.0 - 1.0).sqr();
                let scale =
                    self.config.growl_min_scale * (1.0 - scale) + self.config.growl_scale * scale;
                draw(
                    &assets.guy.guy.growl,
                    mat3::translate(guy.ball.pos)
                        * mat3::rotate(guy.ball.rot)
                        * mat3::scale_uniform(guy.ball.radius * scale)
                        * mat3::translate(shift),
                    1.0,
                );
            }
            draw(
                &assets.guy.guy.body,
                mat3::translate(guy.ball.pos)
                    * mat3::rotate(guy.ball.rot)
                    * mat3::scale_uniform(guy.ball.radius),
                1.0,
            );

            draw(
                if guy.input.force_fart {
                    &assets.guy.guy.closed_eyes
                } else {
                    &assets.guy.guy.open_eyes
                },
                mat3::translate(guy.ball.pos)
                    * mat3::rotate(guy.ball.rot)
                    * mat3::scale_uniform(guy.ball.radius * (0.8 + 0.6 * fart_progress))
                    * mat3::translate(
                        vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * fart_progress,
                    ),
                1.0,
            );
            if guy.fart_state.fart_pressure >= self.config.fart_pressure_released {
                let progress = (guy.fart_state.fart_pressure - self.config.fart_pressure_released)
                    / (self.config.max_fart_pressure - self.config.fart_pressure_released);
                draw(
                    &assets.guy.guy.cheeks,
                    mat3::translate(guy.ball.pos)
                        * mat3::rotate(guy.ball.rot)
                        * mat3::scale_uniform(guy.ball.radius * (0.8 + 0.7 * progress))
                        * mat3::translate(
                            vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * progress,
                        ),
                    (0.5 + 1.0 * progress).min(1.0),
                );
            }

            mem::drop(draw);

            if self.opt.editor {
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

            if Some(guy.id) == self.my_guy || self.show_names {
                assets.font.draw(
                    framebuffer,
                    &self.camera,
                    &guy.customization.name,
                    guy.ball.pos + vec2(0.0, guy.ball.radius * 1.1),
                    geng::TextAlign::CENTER,
                    0.1,
                    Rgba::BLACK,
                );
            }

            if guy.bubble_timer.is_some() {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&assets.bubble)
                        .scale_uniform(guy.ball.radius * self.config.bubble_scale)
                        .translate(guy.ball.pos),
                );
            }
        }

        // Emotes
        for &(_, id, emote) in &self.emotes {
            if let Some(guy) = self.guys.get(&id) {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&assets.emotes[emote])
                        .scale_uniform(0.1)
                        .translate(guy.ball.pos + vec2(0.0, guy.ball.radius * 2.0)),
                );
            }
        }
    }
}
