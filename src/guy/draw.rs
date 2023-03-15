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

            let guy_transform = mat3::translate(guy.ball.pos)
                * mat3::rotate(guy.ball.rot)
                * mat3::scale_uniform(guy.ball.radius);

            if guy.snow_layer != 0.0 {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(guy.ball.pos, guy.radius(), Rgba::WHITE),
                );
            }

            struct Params {
                scale: f32,
                shake: vec2<f32>,
                alpha: f32,
            }
            let mut mode_params: HashMap<GuyRenderLayerMode, Params> = default();

            if let Some(growl_progress) = guy.animation.growl_progress {
                let shake = vec2(self.noise(10.0), self.noise(10.0)) * 0.1;
                let scale = 1.0 - (growl_progress * 2.0 - 1.0).sqr();
                let scale =
                    self.config.growl_min_scale * (1.0 - scale) + self.config.growl_scale * scale;
                mode_params.insert(
                    GuyRenderLayerMode::Growl,
                    Params {
                        scale,
                        shake,
                        alpha: 1.0,
                    },
                );
            }
            let fart_shake = vec2(self.noise(10.0), self.noise(10.0))
                * 0.1
                * if guy.input.force_fart {
                    1.0
                } else {
                    fart_progress
                };
            mode_params.insert(
                GuyRenderLayerMode::Body,
                Params {
                    scale: 1.0,
                    shake: fart_shake,
                    alpha: 1.0,
                },
            );
            mode_params.insert(
                if guy.input.force_fart {
                    GuyRenderLayerMode::ForceFart
                } else {
                    GuyRenderLayerMode::Idle
                },
                Params {
                    scale: 0.8 + 0.6 * fart_progress,
                    shake: fart_shake,
                    alpha: 1.0,
                },
            );
            if guy.fart_state.fart_pressure >= self.config.fart_pressure_released {
                let progress = (guy.fart_state.fart_pressure - self.config.fart_pressure_released)
                    / (self.config.max_fart_pressure - self.config.fart_pressure_released);
                mode_params.insert(
                    GuyRenderLayerMode::Cheeks,
                    Params {
                        scale: 0.8 + 0.7 * progress,
                        shake: vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * progress,
                        alpha: (0.5 + 1.0 * progress).min(1.0),
                    },
                );
            }

            for layer in &assets.guy.guy.layers {
                if let Some(params) = mode_params.get(&layer.params.mode) {
                    let transform = guy_transform
                        * mat3::translate(layer.params.origin)
                        * mat3::scale_uniform(
                            params.scale * layer.params.scale + 1.0 - layer.params.scale,
                        )
                        * mat3::translate(-layer.params.origin)
                        * mat3::translate(params.shake * layer.params.shake)
                        * mat3::translate(vec2(
                            layer.params.go_left * (-guy.input.roll_direction).min(0.0)
                                + layer.params.go_right * (-guy.input.roll_direction).max(0.0),
                            0.0,
                        ));
                    let mut color = if let Some(color) = &layer.params.color {
                        match color.as_str() {
                            "eyes" => eyes_color,
                            "clothes-top" => guy.customization.colors.top,
                            "clothes-bottom" => guy.customization.colors.bottom,
                            "skin" => guy.customization.colors.skin,
                            "hair" => guy.customization.colors.hair,
                            _ => {
                                panic!("What is {color}????")
                            }
                        }
                    } else {
                        Rgba::WHITE
                    };
                    color.a *= params.alpha;
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit_colored(&layer.texture, color)
                            .transform(transform),
                    );
                }
            }

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
