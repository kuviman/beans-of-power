use super::*;

impl Game {
    fn draw_level_impl(
        &self,
        level: &Level,
        framebuffer: &mut ugli::Framebuffer,
        texture: impl Fn(&SurfaceAssets) -> Option<&Texture>,
    ) {
        for surface in &level.surfaces {
            let assets = &self.assets.surfaces[&surface.type_name];
            let texture = match texture(assets) {
                Some(texture) => texture,
                None => continue,
            };
            let normal = (surface.p2 - surface.p1).normalize().rotate_90();
            let len = (surface.p2 - surface.p1).len();
            let height = texture.size().y as f32 / texture.size().x as f32;
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedPolygon::new(
                    vec![
                        draw_2d::TexturedVertex {
                            a_pos: surface.p1,
                            a_color: Rgba::WHITE,
                            a_vt: vec2(0.0, 0.0),
                        },
                        draw_2d::TexturedVertex {
                            a_pos: surface.p2,
                            a_color: Rgba::WHITE,
                            a_vt: vec2(len, 0.0),
                        },
                        draw_2d::TexturedVertex {
                            a_pos: surface.p2 + normal * height,
                            a_color: Rgba::WHITE,
                            a_vt: vec2(len, 1.0),
                        },
                        draw_2d::TexturedVertex {
                            a_pos: surface.p1 + normal * height,
                            a_color: Rgba::WHITE,
                            a_vt: vec2(0.0, 1.0),
                        },
                    ],
                    texture,
                ),
            );
        }
    }

    pub fn draw_level_back(&self, level: &Level, framebuffer: &mut ugli::Framebuffer) {
        for tile in &level.tiles {
            let assets = &self.assets.tiles[&tile.type_name];
            if !assets.params.background {
                continue;
            }
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedPolygon::new(
                    tile.vertices
                        .into_iter()
                        .map(|v| draw_2d::TexturedVertex {
                            a_pos: v,
                            a_color: Rgba::WHITE,
                            a_vt: v - tile.flow * self.simulation_time,
                        })
                        .collect(),
                    &assets.texture,
                ),
            );
        }
        self.geng.draw_2d(
            framebuffer,
            &self.camera,
            &draw_2d::TexturedQuad::unit(&self.assets.closed_outhouse).translate(level.spawn_point),
        );
        self.geng.draw_2d(
            framebuffer,
            &self.camera,
            &draw_2d::TexturedQuad::unit(&self.assets.golden_toilet).translate(level.finish_point),
        );
        {
            for obj in &level.objects {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&self.assets.objects[&obj.type_name])
                        .transform(Mat3::rotate(if obj.type_name == "unicorn" {
                            self.real_time
                        } else {
                            0.0
                        }))
                        .scale_uniform(0.6)
                        .translate(obj.pos),
                );
            }
        }
        self.draw_level_impl(level, framebuffer, |assets| assets.back_texture.as_ref());
    }

    pub fn draw_level_front(&self, level: &Level, framebuffer: &mut ugli::Framebuffer) {
        for tile in &level.tiles {
            let assets = &self.assets.tiles[&tile.type_name];
            if assets.params.background {
                continue;
            }
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedPolygon::new(
                    tile.vertices
                        .into_iter()
                        .map(|v| draw_2d::TexturedVertex {
                            a_pos: v,
                            a_color: Rgba::WHITE,
                            a_vt: v - tile.flow * self.simulation_time
                                + vec2(
                                    self.noise(assets.params.texture_movement_frequency),
                                    self.noise(assets.params.texture_movement_frequency),
                                ) * assets.params.texture_movement_amplitude,
                        })
                        .collect(),
                    &assets.texture,
                ),
            );
        }
        self.draw_level_impl(level, framebuffer, |assets| assets.front_texture.as_ref());
    }
}
