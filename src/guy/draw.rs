use super::*;

impl Game {
    pub fn draw_guys(&self, framebuffer: &mut ugli::Framebuffer) {
        for guy in itertools::chain![
            self.guys.iter().filter(|guy| guy.id != self.client_id),
            self.guys.iter().filter(|guy| guy.id == self.client_id),
        ] {
            let (eyes, cheeks, cheeks_color) = if let Some(custom) =
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
                (&custom.eyes, &custom.cheeks, Rgba::WHITE)
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
                    &self.assets.guy.cheeks,
                    guy.colors.skin,
                )
            };
            let autofart_progress = guy.auto_fart_timer / self.config.auto_fart_interval;
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(eyes, {
                    let k = 0.8;
                    let t = ((autofart_progress - k) / (1.0 - k)).clamp(0.0, 1.0) * 0.5;
                    Rgba::new(1.0, 1.0 - t, 1.0 - t, 1.0)
                })
                .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * autofart_progress)
                .scale_uniform(self.config.guy_radius * (0.8 + 0.6 * autofart_progress))
                .transform(Mat3::rotate(guy.rot))
                .translate(guy.pos),
            );
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(
                    cheeks,
                    Rgba {
                        a: (0.5 + 1.0 * autofart_progress).min(1.0),
                        ..cheeks_color
                    },
                )
                .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * autofart_progress)
                .scale_uniform(self.config.guy_radius * (0.8 + 0.7 * autofart_progress))
                .transform(Mat3::rotate(guy.rot))
                .translate(guy.pos),
            );
            if Some(guy.id) == self.my_guy || self.show_names {
                self.assets.font.draw(
                    framebuffer,
                    &self.camera,
                    &guy.name,
                    guy.pos + vec2(0.0, self.config.guy_radius * 1.1),
                    geng::TextAlign::CENTER,
                    0.1,
                    if guy.postjam {
                        Rgba::BLACK
                    } else {
                        Rgba::new(0.0, 0.0, 0.0, 0.5)
                    },
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
