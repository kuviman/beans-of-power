use super::*;

#[derive(ugli::Vertex)]
struct TileVertex {
    a_pos: Vec2<f32>,
    a_flow: Vec2<f32>,
}

#[derive(ugli::Vertex, Copy, Clone)]
struct SurfaceVertex {
    a_pos: Vec2<f32>,
    a_normal: Vec2<f32>,
    a_vt: Vec2<f32>,
}

pub struct LevelMesh {
    tiles: ugli::VertexBuffer<TileVertex>,
    surfaces: ugli::VertexBuffer<SurfaceVertex>,
}

impl LevelMesh {
    pub fn new(geng: &Geng, level: &Level) -> Self {
        Self {
            tiles: ugli::VertexBuffer::new_static(
                geng.ugli(),
                level
                    .tiles
                    .iter()
                    .flat_map(|tile| {
                        tile.vertices.into_iter().map(|v| TileVertex {
                            a_pos: v,
                            a_flow: tile.flow,
                        })
                    })
                    .collect(),
            ),
            surfaces: ugli::VertexBuffer::new_static(
                geng.ugli(),
                level
                    .surfaces
                    .iter()
                    .flat_map(|surface| {
                        let normal = (surface.p2 - surface.p1).normalize().rotate_90();
                        let len = (surface.p2 - surface.p1).len();
                        let vs = [
                            SurfaceVertex {
                                a_pos: surface.p1,
                                a_normal: normal,
                                a_vt: vec2(0.0, 0.0),
                            },
                            SurfaceVertex {
                                a_pos: surface.p2,
                                a_normal: normal,
                                a_vt: vec2(len, 0.0),
                            },
                            SurfaceVertex {
                                a_pos: surface.p2,
                                a_normal: normal,
                                a_vt: vec2(len, 1.0),
                            },
                            SurfaceVertex {
                                a_pos: surface.p1,
                                a_normal: normal,
                                a_vt: vec2(0.0, 1.0),
                            },
                        ];
                        [vs[0], vs[1], vs[2], vs[0], vs[2], vs[3]]
                    })
                    .collect(),
            ),
        }
    }
}

impl Game {
    fn get_mesh<'a>(&self, level: &'a Level) -> Ref<'a, LevelMesh> {
        {
            let mut mesh = level.mesh.borrow_mut();
            if mesh.is_none() {
                *mesh = Some(LevelMesh::new(&self.geng, level));
                debug!("Creating level mesh");
            };
        }
        Ref::map(level.mesh.borrow(), |opt| opt.as_ref().unwrap())
    }

    fn draw_surfaces(
        &self,
        level: &Level,
        framebuffer: &mut ugli::Framebuffer,
        texture: impl Fn(&SurfaceAssets) -> Option<&Texture>,
    ) {
        let mesh = self.get_mesh(level);
        for (index, surface) in level.surfaces.iter().enumerate() {
            let assets = &self.assets.surfaces[&surface.type_name];
            let texture = match texture(assets) {
                Some(texture) => texture,
                None => continue,
            };
            ugli::draw(
                framebuffer,
                &self.assets.shaders.surface,
                ugli::DrawMode::Triangles,
                mesh.surfaces.slice(index * 6..index * 6 + 6),
                (
                    ugli::uniforms! {
                        u_texture: &**texture,
                        u_height: texture.size().y as f32 / texture.size().x as f32,
                        u_simulation_time: self.simulation_time,
                    },
                    geng::camera2d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::default()),
                    ..default()
                },
            );
        }
    }

    fn draw_tiles(&self, framebuffer: &mut ugli::Framebuffer, level: &Level, background: bool) {
        let mesh = self.get_mesh(level);
        for (index, tile) in level.tiles.iter().enumerate() {
            let assets = &self.assets.tiles[&tile.type_name];
            if assets.params.background != background {
                continue;
            }
            ugli::draw(
                framebuffer,
                &self.assets.shaders.tile,
                ugli::DrawMode::Triangles,
                mesh.tiles.slice(index * 3..index * 3 + 3),
                (
                    ugli::uniforms! {
                        u_texture: &*assets.texture,
                        u_simulation_time: self.simulation_time,
                        u_texture_shift: vec2(
                            self.noise(assets.params.texture_movement_frequency),
                            self.noise(assets.params.texture_movement_frequency),
                        ) * assets.params.texture_movement_amplitude,
                    },
                    geng::camera2d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::default()),
                    ..default()
                },
            );
        }
    }

    pub fn draw_level_back(&self, level: &Level, framebuffer: &mut ugli::Framebuffer) {
        self.draw_tiles(framebuffer, level, true);
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
        self.draw_surfaces(level, framebuffer, |assets| assets.back_texture.as_ref());
    }

    pub fn draw_level_front(&self, level: &Level, framebuffer: &mut ugli::Framebuffer) {
        self.draw_tiles(framebuffer, level, false);
        self.draw_surfaces(level, framebuffer, |assets| assets.front_texture.as_ref());
    }
}
