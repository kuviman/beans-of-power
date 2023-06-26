use super::*;

#[derive(ugli::Vertex)]
struct TileVertex {
    a_pos: vec2<f32>,
    a_side_distances: vec3<f32>,
    a_corner_distances: vec3<f32>,
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
    pub fn new(geng: &Geng, assets: &AssetsHandle, level: &Level) -> Self {
        let assets = assets.get();
        let surface_texture_height = |surface: &Surface| -> f32 {
            let surface_assets = &assets.surfaces[&surface.type_name];
            let texture = surface_assets
                .textures
                .front
                .as_ref()
                .or(surface_assets.textures.back.as_ref())
                .unwrap();
            texture.size().y as f32 / texture.size().x as f32
        };
        let surface_texture_radius =
            |surface: &Surface| -> f32 { surface_texture_height(surface) / 2.0 };
        let arc_len = |a: &Surface, b: &Surface| -> f32 {
            // assert_eq!(a.type_name, b.type_name);
            let n1 = a.normal();
            let n2 = b.normal();
            let angle = f32::atan2(vec2::skew(n1, n2), vec2::dot(n1, n2));
            let r = surface_texture_radius(a);
            (angle * r).abs()
        };
        Self {
            layers: level
                .layers
                .iter()
                .map(|layer| LayerMesh {
                    tiles: {
                        let mut vertex_data: HashMap<String, Vec<TileVertex>> = HashMap::new();
                        for tile in &layer.tiles {
                            let fadeout_distance =
                                assets.tiles[&tile.type_name].params.fadeout_distance
                                    * layer.texture_scale;
                            let data: [vec2<f32>; 3] = std::array::from_fn(|i| {
                                let mut vs = tile.vertices;
                                vs.rotate_left(i);
                                let p1 = vs[1];
                                let v1 = vs[0] - vs[1];
                                let p2 = vs[2];
                                let n2 = (vs[2] - vs[0]).rotate_90().normalize();
                                let p1 = p1 + v1.rotate_90().normalize() * fadeout_distance;
                                let p2 = p2 + n2 * fadeout_distance;
                                let t = ray_hit_time(p1, v1, p2, n2);
                                p1 + v1 * t
                            });
                            vertex_data
                                .entry(tile.type_name.clone())
                                .or_default()
                                .extend(data.into_iter().map(|v| TileVertex {
                                    a_pos: v,
                                    a_side_distances: {
                                        let distances: [f32; 3] = std::array::from_fn(|i| {
                                            let n = (tile.vertices[i] - tile.vertices[(i + 1) % 3])
                                                .rotate_90()
                                                .normalize();
                                            (vec2::dot(v, n) - vec2::dot(tile.vertices[i], n))
                                                / fadeout_distance
                                        });
                                        let [x, y, z] = distances;
                                        vec3(x, y, z)
                                    },
                                    a_corner_distances: {
                                        let distances: [f32; 3] = std::array::from_fn(|i| {
                                            let mut vs = tile.vertices;
                                            vs.rotate_left(i);
                                            let n1 = (vs[0] - vs[1]).rotate_90().normalize();
                                            let n2 = (vs[2] - vs[0]).rotate_90().normalize();
                                            let n = (n1 + n2).normalize();
                                            (vec2::dot(v, n) - vec2::dot(tile.vertices[i], n))
                                                / fadeout_distance
                                        });
                                        let [x, y, z] = distances;
                                        vec3(x, y, z)
                                    },
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
                            let mut this_pass = Vec::new();
                            while let Some(key) = queue.pop_front() {
                                this_pass.push(key);
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

                            let mut this_pass_vertex_data =
                                HashMap::<String, Vec<SurfaceVertex>>::new();

                            for i in this_pass {
                                let surface = &layer.surfaces[i];
                                let normal = (surface.p2 - surface.p1).normalize().rotate_90();
                                let start_t = *vertex_ts.get(&i).unwrap();
                                let len = (surface.p2 - surface.p1).len();
                                let end_t = start_t + len;

                                let prev = layer.surfaces.iter().find(|other| {
                                    other.p2 == surface.p1 && other.type_name == surface.type_name
                                });
                                let next = layer.surfaces.iter().find(|other| {
                                    other.p1 == surface.p2 && other.type_name == surface.type_name
                                });

                                // Rect
                                this_pass_vertex_data
                                    .entry(surface.type_name.clone())
                                    .or_default()
                                    .extend({
                                        let dir = surface.p2 - surface.p1;
                                        let start_ratio = prev
                                            .map_or(0.0, |prev| {
                                                ray_hit_time(
                                                    surface.p1
                                                        + surface.normal()
                                                            * surface_texture_height(surface),
                                                    dir,
                                                    prev.p1
                                                        + prev.normal()
                                                            * surface_texture_height(prev),
                                                    prev.normal(),
                                                )
                                            })
                                            .max(0.0);
                                        let end_ratio = next
                                            .map_or(1.0, |next| {
                                                ray_hit_time(
                                                    surface.p1
                                                        + surface.normal()
                                                            * surface_texture_height(surface),
                                                    dir,
                                                    next.p1
                                                        + next.normal()
                                                            * surface_texture_height(next),
                                                    next.normal(),
                                                )
                                            })
                                            .min(1.0);
                                        let p1 = surface.p1 + dir * start_ratio;
                                        let p2 = surface.p1 + dir * end_ratio;
                                        let vs = [
                                            SurfaceVertex {
                                                a_pos: p1,
                                                a_normal: normal,
                                                a_flow: surface.flow,
                                                a_vt: vec2(start_t, 0.0),
                                            },
                                            SurfaceVertex {
                                                a_pos: p2,
                                                a_normal: normal,
                                                a_flow: surface.flow,
                                                a_vt: vec2(end_t, 0.0),
                                            },
                                            SurfaceVertex {
                                                a_pos: p2
                                                    + surface_texture_height(surface) * normal,
                                                a_normal: normal,
                                                a_flow: surface.flow,
                                                a_vt: vec2(end_t, 1.0),
                                            },
                                            SurfaceVertex {
                                                a_pos: p1
                                                    + surface_texture_height(surface) * normal,
                                                a_normal: normal,
                                                a_flow: surface.flow,
                                                a_vt: vec2(start_t, 1.0),
                                            },
                                        ];
                                        [vs[0], vs[1], vs[2], vs[0], vs[2], vs[3]]
                                    });

                                // Corner to the next segment
                                if let Some(next) = next {
                                    const R: usize = 100;
                                    fn lerp(a: vec2<f32>, b: vec2<f32>, t: f32) -> vec2<f32> {
                                        a * (1.0 - t) + b * t
                                    }
                                    let n1 = normal;
                                    let n2 = (next.p2 - next.p1).rotate_90().normalize_or_zero();
                                    struct Point {
                                        pos: vec2<f32>,
                                        normal: vec2<f32>,
                                        height: f32,
                                    }
                                    let mut vs = Vec::<Point>::new();
                                    if vec2::skew(surface.normal(), next.normal()) > EPS {
                                        let ray_start = surface.p1
                                            + surface.normal() * surface_texture_height(surface);
                                        let ray_dir = surface.p2 - surface.p1;
                                        let t = ray_hit_time(
                                            ray_start,
                                            ray_dir,
                                            next.p1 + next.normal() * surface_texture_height(next),
                                            next.normal(),
                                        );
                                        let mid = ray_start + ray_dir * t;
                                        let p1 = mid
                                            - surface.normal() * surface_texture_height(surface);
                                        let p2 = mid - next.normal() * surface_texture_height(next);
                                        for (p1, p2) in [(p1, surface.p2), (surface.p2, p2)] {
                                            for j in 0..=R {
                                                let pos = lerp(p1, p2, j as f32 / R as f32);
                                                if vs.last().map(|p| p.pos) == Some(pos) {
                                                    // LUL
                                                    continue;
                                                }
                                                vs.push(Point {
                                                    pos,
                                                    normal: (mid - pos).normalize(),
                                                    height: (mid - pos).len(),
                                                });
                                            }
                                        }
                                    } else {
                                        let a = n1.arg().as_radians();
                                        let b = n2.arg().as_radians();
                                        let mut angle = b - a;
                                        if angle > 0.0 {
                                            angle -= 2.0 * f32::PI;
                                        }
                                        for j in 0..=R {
                                            vs.push(Point {
                                                pos: surface.p2,
                                                normal: vec2(1.0, 0.0).rotate(Angle::from_radians(
                                                    a + angle * j as f32 / R as f32,
                                                )),
                                                height: surface_texture_height(surface),
                                            });
                                        }
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
                                                    + (end_t - start_t) * i as f32
                                                        / vs.len() as f32,
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
                                                a_pos: p2.pos + p2.height * p2.normal,
                                                a_normal: p2.normal,
                                                a_flow: surface.flow,
                                                a_vt: vec2(end_t, 1.0),
                                            },
                                            SurfaceVertex {
                                                a_pos: p1.pos + p1.height * p1.normal,
                                                a_normal: p1.normal,
                                                a_flow: surface.flow,
                                                a_vt: vec2(start_t, 1.0),
                                            },
                                        ];
                                        this_pass_vertex_data
                                            .entry(surface.type_name.clone())
                                            .or_default()
                                            .extend([vs[0], vs[1], vs[2], vs[0], vs[2], vs[3]]);
                                    }
                                } else {
                                    log::warn!("Not connected????");
                                }
                            }

                            let max = this_pass_vertex_data
                                .values()
                                .flatten()
                                .map(|v| r32(v.a_vt.x))
                                .max()
                                .unwrap();
                            let min = this_pass_vertex_data
                                .values()
                                .flatten()
                                .map(|v| r32(v.a_vt.x))
                                .min()
                                .unwrap();
                            let total_len = (max - min).raw();
                            let rounded_len = total_len.round().max(1.0);
                            for (name, mut data) in this_pass_vertex_data {
                                for v in &mut data {
                                    v.a_vt.x *= rounded_len / total_len;
                                }
                                vertex_data.entry(name).or_default().extend(data);
                            }
                        }
                        vertex_data
                            .into_iter()
                            .map(|(type_name, data)| {
                                let data = {
                                    // TODO should be handled differently
                                    let surface_assets = &assets.surfaces[&type_name];
                                    let texture = surface_assets
                                        .textures
                                        .front
                                        .as_ref()
                                        .or(surface_assets.textures.back.as_ref())
                                        .unwrap();
                                    let height = texture.size().y as f32 / texture.size().x as f32;
                                    let mut data = data;
                                    for vertex in &mut data {
                                        vertex.a_pos -= vertex.a_normal
                                            * height
                                            * surface_assets.params.texture_underground;
                                    }
                                    data
                                };
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
                log::debug!("Creating level mesh");
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
            let texture_shift =
                surface_assets.params.texture_speed * self.simulation_time * texture_move_direction;
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
                        u_layer_color: level.layers[layer_index].color,
                        u_reveal_radius: level.layers[layer_index].reveal_radius,
                    },
                    camera.uniforms(self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::straight_alpha()),
                    ..default()
                },
            );
        }
    }

    fn draw_tiles(&self, framebuffer: &mut ugli::Framebuffer, level: &Level, layer_index: usize) {
        let assets = self.assets.get();
        let camera = geng::Camera2d {
            center: self.camera.center * level.layers[layer_index].parallax,
            ..self.camera
        };
        let mesh = self.get_mesh(level);

        for (type_name, data) in &mesh.layers[layer_index].tiles {
            let tile_assets = &assets.tiles[type_name];
            let texture_scale = vec2(tile_assets.texture.size().map(|x| x as f32).aspect(), 1.0)
                * tile_assets.params.texture_scale
                * level.layers[layer_index].texture_scale;
            for draw_index in 0..tile_assets.params.draw_times {
                let texture_shift = vec2(
                    self.noise(
                        draw_index as f32,
                        tile_assets.params.texture_movement_frequency,
                    ),
                    self.noise(
                        draw_index as f32,
                        tile_assets.params.texture_movement_frequency,
                    ),
                ) * tile_assets.params.texture_movement_amplitude
                    + vec2(
                        self.noise(draw_index as f32, 0.0),
                        self.noise(draw_index as f32, 0.0),
                    );
                let texture_rotation = tile_assets.params.texture_rotation * f32::PI / 180.0;
                ugli::draw(
                    framebuffer,
                    &assets.shaders.tile,
                    ugli::DrawMode::Triangles,
                    data,
                    (
                        ugli::uniforms! {
                            u_texture: &*tile_assets.texture,
                            u_simulation_time: self.simulation_time,
                            u_texture_shift: texture_shift,
                            u_reveal_radius: level.layers[layer_index].reveal_radius,
                            u_texture_scale: texture_scale,
                            u_layer_color: level.layers[layer_index].color,
                            u_texture_rotation: texture_rotation,
                        },
                        camera.uniforms(self.framebuffer_size),
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::straight_alpha()),
                        ..default()
                    },
                );
            }
        }
    }

    pub fn draw_portals(&self, level: &Level, framebuffer: &mut ugli::Framebuffer) {
        let assets = self.assets.get();
        for portal in &level.portals {
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::TexturedQuad::unit_colored(&assets.portal, portal.color)
                    .scale_uniform(self.config.portal.size)
                    .rotate(Angle::from_radians(self.real_time))
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
        self.draw_tiles(framebuffer, level, layer_index);
        {
            for obj in &level.layers[layer_index].objects {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    &self.camera,
                    &draw2d::TexturedQuad::unit(&assets.objects[&obj.type_name])
                        .transform(mat3::rotate(Angle::from_radians(
                            if obj.fart_type().is_some() {
                                self.real_time
                            } else {
                                0.0
                            },
                        )))
                        .scale_uniform(0.6)
                        .translate(obj.pos),
                );
            }
        }
        self.draw_surfaces(
            level,
            layer_index,
            framebuffer,
            |assets| assets.textures.back.as_ref(),
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
        self.draw_surfaces(
            level,
            layer_index,
            framebuffer,
            |assets| assets.textures.front.as_ref(),
            -1.0,
        );
        level
            .cannon
            .draw(&self.geng, &self.assets.get(), framebuffer, &self.camera);
    }
}
