use super::*;

#[derive(ugli::Vertex)]
struct TileVertex {
    a_pos: vec2<f32>,
    a_flow: vec2<f32>,
}

#[derive(ugli::Vertex, Copy, Clone)]
struct SurfaceVertex {
    a_pos: vec2<f32>,
    a_normal: vec2<f32>,
    a_vt: vec2<f32>,
    a_flow: f32,
}

pub struct LayerMesh {
    tiles: HashMap<String, ugli::VertexBuffer<TileVertex>>,
    surfaces: HashMap<String, ugli::VertexBuffer<SurfaceVertex>>,
}

pub struct LevelMesh {
    layers: Vec<LayerMesh>,
}

impl LevelMesh {
    pub fn new(geng: &Geng, assets: &Assets, level: &Level) -> Self {
        let assets = assets.get();
        let surface_texture_radius = |surface: &Surface| -> f32 {
            let surface_assets = &assets.surfaces[&surface.type_name];
            let texture = surface_assets
                .front_texture
                .as_ref()
                .or(surface_assets.back_texture.as_ref())
                .unwrap();
            let texture_height = texture.size().y as f32 / texture.size().x as f32;
            texture_height / 2.0
        };
        let arc_len = |a: &Surface, b: &Surface| -> f32 {
            // assert_eq!(a.type_name, b.type_name);
            let n1 = a.normal();
            let n2 = b.normal();
            let mut angle = -f32::atan2(vec2::skew(n1, n2), vec2::dot(n1, n2));
            if angle < 0.0 {
                angle += 2.0 * f32::PI;
            }
            let r = surface_texture_radius(a);
            angle * r
        };
        Self {
            layers: level
                .layers
                .iter()
                .map(|layer| LayerMesh {
                    tiles: {
                        let mut vertex_data: HashMap<String, Vec<TileVertex>> = HashMap::new();
                        for tile in &layer.tiles {
                            vertex_data
                                .entry(tile.type_name.clone())
                                .or_default()
                                .extend(tile.vertices.into_iter().map(|v| TileVertex {
                                    a_pos: v,
                                    a_flow: tile.flow,
                                }));
                        }
                        vertex_data
                            .into_iter()
                            .map(|(type_name, data)| {
                                (type_name, ugli::VertexBuffer::new_static(geng.ugli(), data))
                            })
                            .collect()
                    },
                    surfaces: {
                        let mut vertex_data: HashMap<String, Vec<SurfaceVertex>> = HashMap::new();

                        type Key = usize;
                        let mut vertex_ts: HashMap<Key, f32> = default();
                        let mut queue = std::collections::VecDeque::<Key>::new();
                        for key in 0..layer.surfaces.len() {
                            if vertex_ts.contains_key(&key) {
                                continue;
                            }
                            vertex_ts.insert(key, 0.0);
                            queue.push_back(key);
                            while let Some(key) = queue.pop_front() {
                                let surface = &layer.surfaces[key];
                                let start_t = *vertex_ts.get(&key).unwrap();
                                let end_t = start_t + (surface.p2 - surface.p1).len();
                                for (i, other) in layer.surfaces.iter().enumerate() {
                                    if other.type_name != surface.type_name {
                                        continue;
                                    }
                                    let mut push = |key: Key, t: f32| {
                                        vertex_ts.entry(key).or_insert_with(|| {
                                            queue.push_back(key);
                                            t
                                        });
                                    };
                                    for (t, p) in [(start_t, surface.p1), (end_t, surface.p2)] {
                                        if p == other.p1 {
                                            push(i, t + arc_len(surface, other));
                                        }
                                        if p == other.p2 {
                                            push(
                                                i,
                                                t - (other.p2 - other.p1).len()
                                                    - arc_len(other, surface),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        for (i, surface) in layer.surfaces.iter().enumerate() {
                            let normal = (surface.p2 - surface.p1).normalize().rotate_90();
                            let start_t = *vertex_ts.get(&i).unwrap();
                            let len = (surface.p2 - surface.p1).len();
                            let end_t = start_t + len;

                            // Rect
                            vertex_data
                                .entry(surface.type_name.clone())
                                .or_default()
                                .extend({
                                    let vs = [
                                        SurfaceVertex {
                                            a_pos: surface.p1,
                                            a_normal: normal,
                                            a_flow: surface.flow,
                                            a_vt: vec2(start_t, 0.0),
                                        },
                                        SurfaceVertex {
                                            a_pos: surface.p2,
                                            a_normal: normal,
                                            a_flow: surface.flow,
                                            a_vt: vec2(end_t, 0.0),
                                        },
                                        SurfaceVertex {
                                            a_pos: surface.p2,
                                            a_normal: normal,
                                            a_flow: surface.flow,
                                            a_vt: vec2(end_t, 1.0),
                                        },
                                        SurfaceVertex {
                                            a_pos: surface.p1,
                                            a_normal: normal,
                                            a_flow: surface.flow,
                                            a_vt: vec2(start_t, 1.0),
                                        },
                                    ];
                                    [vs[0], vs[1], vs[2], vs[0], vs[2], vs[3]]
                                });

                            let next = layer.surfaces.iter().find(|other| {
                                other.p1 == surface.p2 && other.type_name == surface.type_name
                            });

                            // Corner to the next segment
                            if let Some(next) = next {
                                const R: usize = 100;
                                let n1 = normal;
                                let n2 = (next.p2 - next.p1).rotate_90().normalize_or_zero();
                                let mut vs = Vec::new();
                                struct Point {
                                    pos: vec2<f32>,
                                    normal: vec2<f32>,
                                }
                                for j in 0..=R {
                                    fn lerp(a: vec2<f32>, b: vec2<f32>, t: f32) -> vec2<f32> {
                                        a * (1.0 - t) + b * t
                                    }
                                    fn slerp(a: vec2<f32>, b: vec2<f32>, t: f32) -> vec2<f32> {
                                        lerp(a, b, t).normalize()
                                    }
                                    vs.push(Point {
                                        pos: surface.p2,
                                        normal: slerp(n1, n2, j as f32 / R as f32),
                                    });
                                }
                                let (start_t, end_t) = {
                                    let start_t = end_t;
                                    (start_t, start_t + arc_len(surface, next))
                                };
                                for (i, seg) in vs.windows(2).enumerate() {
                                    let p1 = &seg[0];
                                    let p2 = &seg[1];
                                    let (start_t, end_t) = {
                                        (
                                            start_t
                                                + (end_t - start_t) * i as f32 / vs.len() as f32,
                                            start_t
                                                + (end_t - start_t) * (i + 1) as f32
                                                    / vs.len() as f32,
                                        )
                                    };
                                    let vs = [
                                        SurfaceVertex {
                                            a_pos: p1.pos,
                                            a_normal: p1.normal,
                                            a_flow: surface.flow,
                                            a_vt: vec2(start_t, 0.0),
                                        },
                                        SurfaceVertex {
                                            a_pos: p2.pos,
                                            a_normal: p2.normal,
                                            a_flow: surface.flow,
                                            a_vt: vec2(end_t, 0.0),
                                        },
                                        SurfaceVertex {
                                            a_pos: p2.pos,
                                            a_normal: p2.normal,
                                            a_flow: surface.flow,
                                            a_vt: vec2(end_t, 1.0),
                                        },
                                        SurfaceVertex {
                                            a_pos: p1.pos,
                                            a_normal: p1.normal,
                                            a_flow: surface.flow,
                                            a_vt: vec2(start_t, 1.0),
                                        },
                                    ];
                                    vertex_data
                                        .entry(surface.type_name.clone())
                                        .or_default()
                                        .extend([vs[0], vs[1], vs[2], vs[0], vs[2], vs[3]]);
                                }
                            } else {
                                warn!("Not connected????");
                            }
                        }
                        vertex_data
                            .into_iter()
                            .map(|(type_name, data)| {
                                (type_name, ugli::VertexBuffer::new_static(geng.ugli(), data))
                            })
                            .collect()
                    },
                })
                .collect(),
        }
    }
}

impl Game {
    fn get_mesh<'a>(&self, level: &'a Level) -> Ref<'a, LevelMesh> {
        {
            let mut mesh = level.mesh.borrow_mut();
            if mesh.is_none() {
                *mesh = Some(LevelMesh::new(&self.geng, &self.assets, level));
                debug!("Creating level mesh");
            };
        }
        Ref::map(level.mesh.borrow(), |opt| opt.as_ref().unwrap())
    }

    fn draw_surfaces(
        &self,
        level: &Level,
        layer_index: usize,
        framebuffer: &mut ugli::Framebuffer,
        texture: impl Fn(&SurfaceAssets) -> Option<&Texture>,
        texture_shift: f32,
        texture_move_direction: f32,
    ) {
        let assets = self.assets.get();
        let camera = geng::Camera2d {
            center: self.camera.center * level.layers[layer_index].parallax,
            ..self.camera
        };
        let mesh = self.get_mesh(level);

        for (type_name, data) in &mesh.layers[layer_index].surfaces {
            let surface_assets = &assets.surfaces[type_name];
            let texture = match texture(surface_assets) {
                Some(texture) => texture,
                None => continue,
            };
            let texture_shift = texture_shift
                + surface_assets.params.texture_speed
                    * self.simulation_time
                    * texture_move_direction;
            ugli::draw(
                framebuffer,
                &assets.shaders.surface,
                ugli::DrawMode::Triangles,
                data,
                (
                    ugli::uniforms! {
                        u_texture: &**texture,
                        u_height: texture.size().y as f32 / texture.size().x as f32,
                        u_simulation_time: self.simulation_time,
                        u_flex_frequency: surface_assets.params.flex_frequency,
                        u_flex_amplitude: surface_assets.params.flex_amplitude,
                        u_texture_shift: texture_shift,
                    },
                    geng::camera2d_uniforms(&camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    ..default()
                },
            );
        }
    }

    fn draw_tiles(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        level: &Level,
        layer_index: usize,
        background: bool,
    ) {
        let assets = self.assets.get();
        let camera = geng::Camera2d {
            center: self.camera.center * level.layers[layer_index].parallax,
            ..self.camera
        };
        let mesh = self.get_mesh(level);

        for (type_name, data) in &mesh.layers[layer_index].tiles {
            let tile_assets = &assets.tiles[type_name];
            if tile_assets.params.background != background {
                continue;
            }
            ugli::draw(
                framebuffer,
                &assets.shaders.tile,
                ugli::DrawMode::Triangles,
                data,
                (
                    ugli::uniforms! {
                        u_texture: &*tile_assets.texture,
                        u_simulation_time: self.simulation_time,
                        u_texture_shift: vec2(
                            self.noise(tile_assets.params.texture_movement_frequency),
                            self.noise(tile_assets.params.texture_movement_frequency),
                        ) * tile_assets.params.texture_movement_amplitude,
                        u_reveal_radius: level.layers[layer_index].reveal_radius,
                    },
                    geng::camera2d_uniforms(&camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    ..default()
                },
            );
        }
    }

    pub fn draw_cannons(&self, level: &Level, framebuffer: &mut ugli::Framebuffer) {
        let assets = self.assets.get();
        for cannon in &level.cannons {
            let mut scale = vec2(1.0, 1.0);
            if cannon.rot > f32::PI / 2.0 || cannon.rot < -f32::PI / 2.0 {
                scale.x = -scale.x;
            }
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit(&assets.cannon.body)
                    .rotate(cannon.rot)
                    .translate(cannon.pos),
            );
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit(&assets.cannon.base)
                    .scale(scale)
                    .translate(cannon.pos),
            );
        }
    }

    pub fn draw_portals(&self, level: &Level, framebuffer: &mut ugli::Framebuffer) {
        let assets = self.assets.get();
        for portal in &level.portals {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(&assets.portal, portal.color)
                    .scale_uniform(self.config.portal.size)
                    .rotate(self.real_time)
                    .translate(portal.pos),
            );
        }
    }

    pub fn draw_layer_back(
        &self,
        level: &Level,
        layer_index: usize,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let assets = self.assets.get();
        self.draw_tiles(framebuffer, level, layer_index, true);
        {
            for obj in &level.layers[layer_index].objects {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&assets.objects[&obj.type_name])
                        .transform(mat3::rotate(if obj.fart_type().is_some() {
                            self.real_time
                        } else {
                            0.0
                        }))
                        .scale_uniform(0.6)
                        .translate(obj.pos),
                );
            }
        }
        self.draw_surfaces(
            level,
            layer_index,
            framebuffer,
            |assets| assets.back_texture.as_ref(),
            43756.0,
            1.0,
        );
        self.draw_portals(level, framebuffer);
    }

    pub fn draw_layer_front(
        &self,
        level: &Level,
        layer_index: usize,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_tiles(framebuffer, level, layer_index, false);
        self.draw_surfaces(
            level,
            layer_index,
            framebuffer,
            |assets| assets.front_texture.as_ref(),
            -123.0,
            -1.0,
        );
        self.draw_cannons(level, framebuffer);
    }
}
