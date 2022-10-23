use super::*;

pub const CONTROLS_LEFT: [geng::Key; 2] = [geng::Key::A, geng::Key::Left];
pub const CONTROLS_RIGHT: [geng::Key; 2] = [geng::Key::D, geng::Key::Right];
pub const CONTROLS_FORCE_FART: [geng::Key; 3] = [geng::Key::W, geng::Key::Up, geng::Key::Space];

#[derive(Clone)]
enum UiMessage {
    Play,
    RandomizeSkin,
    TogglePostJam,
}

pub struct Game {
    best_time: Option<f32>,
    emotes: Vec<(f32, Id, usize)>,
    best_progress: f32,
    framebuffer_size: Vec2<f32>,
    prev_mouse_pos: Vec2<f64>,
    geng: Geng,
    config: Config,
    assets: Rc<Assets>,
    camera: geng::Camera2d,
    levels: (Level, Level),
    editor: Option<EditorState>,
    guys: Collection<Guy>,
    my_guy: Option<Id>,
    simulation_time: f32,
    remote_simulation_times: HashMap<Id, f32>,
    remote_updates: HashMap<Id, std::collections::VecDeque<(f32, Guy)>>,
    real_time: f32,
    noise: noise::OpenSimplex,
    opt: Opt,
    farticles: Vec<Farticle>,
    volume: f32,
    client_id: Id,
    connection: Connection,
    customization: Guy,
    ui_controller: ui::Controller,
    buttons: Vec<ui::Button<UiMessage>>,
    show_customizer: bool,
    old_music: geng::SoundEffect,
    new_music: geng::SoundEffect,
    show_names: bool,
    show_leaderboard: bool,
    follow: Option<Id>,
}

impl Drop for Game {
    fn drop(&mut self) {
        self.save_level();
    }
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        levels: (Level, Level),
        opt: Opt,
        client_id: Id,
        connection: Connection,
    ) -> Self {
        let mut result = Self {
            best_time: None,
            emotes: vec![],
            geng: geng.clone(),
            config: assets.config.clone(),
            assets: assets.clone(),
            camera: geng::Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: 5.0,
            },
            framebuffer_size: vec2(1.0, 1.0),
            editor: if opt.editor {
                Some(EditorState::new())
            } else {
                None
            },
            levels,
            guys: Collection::new(),
            my_guy: None,
            real_time: 0.0,
            noise: noise::OpenSimplex::new(),
            prev_mouse_pos: Vec2::ZERO,
            opt: opt.clone(),
            farticles: default(),
            volume: assets.config.volume,
            client_id,
            connection,
            simulation_time: 0.0,
            remote_simulation_times: HashMap::new(),
            remote_updates: default(),
            customization: {
                let mut guy = Guy::new(-1, vec2(0.0, 0.0), false);
                if opt.postjam {
                    guy.postjam = true;
                }
                guy
            },
            best_progress: 0.0,
            ui_controller: ui::Controller::new(geng, assets),
            buttons: vec![
                ui::Button::new("PLAY", vec2(0.0, -3.0), 1.0, 0.5, UiMessage::Play),
                ui::Button::new(
                    "randomize",
                    vec2(2.0, 0.0),
                    0.7,
                    0.0,
                    UiMessage::RandomizeSkin,
                ),
                ui::Button::new(
                    &format!("postjam ({})", if opt.postjam { "on" } else { "off" }),
                    vec2(0.0, -4.0),
                    0.7,
                    0.5,
                    UiMessage::TogglePostJam,
                ),
            ],
            show_customizer: !opt.editor,
            old_music: {
                let mut effect = assets.sfx.old_music.play();
                effect.set_volume(0.0);
                effect
            },
            new_music: {
                let mut effect = assets.sfx.new_music.play();
                effect.set_volume(0.0);
                effect
            },
            show_names: true,
            show_leaderboard: true,
            follow: None,
        };
        if !opt.editor {
            result.my_guy = Some(client_id);
            result.guys.insert(Guy::new(
                client_id,
                if result.customization.postjam {
                    result.levels.1.spawn_point
                } else {
                    result.levels.0.spawn_point
                },
                true,
            ));
        }
        result
    }

    pub fn snapped_cursor_position(&self) -> Vec2<f32> {
        self.snap_position(self.camera.screen_to_world(
            self.framebuffer_size,
            self.geng.window().mouse_pos().map(|x| x as f32),
        ))
    }

    pub fn snap_position(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let closest_point = itertools::chain![
            self.levels
                .1
                .surfaces
                .iter()
                .flat_map(|surface| [surface.p1, surface.p2]),
            self.levels.1.tiles.iter().flat_map(|tile| tile.vertices)
        ]
        .filter(|&p| (pos - p).len() < self.config.snap_distance)
        .min_by_key(|&p| r32((pos - p).len()));
        closest_point.unwrap_or(pos)
    }

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

    pub fn draw_level_impl(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: impl Fn(&SurfaceAssets) -> Option<&Texture>,
    ) {
        let level = if self.customization.postjam {
            &self.levels.1
        } else {
            &self.levels.0
        };
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

    pub fn draw_level_back(&self, framebuffer: &mut ugli::Framebuffer) {
        let level = if self.customization.postjam {
            &self.levels.1
        } else {
            &self.levels.0
        };
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
            &draw_2d::TexturedQuad::unit(&self.assets.closed_outhouse).translate(
                if self.customization.postjam {
                    self.levels.1.spawn_point
                } else {
                    self.levels.0.spawn_point
                },
            ),
        );
        self.geng.draw_2d(
            framebuffer,
            &self.camera,
            &draw_2d::TexturedQuad::unit(&self.assets.golden_toilet)
                .translate(self.levels.0.finish_point),
        );
        {
            let level = if self.customization.postjam {
                &self.levels.1
            } else {
                &self.levels.0
            };
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
        self.draw_level_impl(framebuffer, |assets| assets.back_texture.as_ref());
    }

    pub fn draw_level_front(&self, framebuffer: &mut ugli::Framebuffer) {
        let level = if self.customization.postjam {
            &self.levels.1
        } else {
            &self.levels.0
        };
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
        self.draw_level_impl(framebuffer, |assets| assets.front_texture.as_ref());
    }

    pub fn find_hovered_surface(&self) -> Option<usize> {
        let cursor = self.camera.screen_to_world(
            self.framebuffer_size,
            self.geng.window().mouse_pos().map(|x| x as f32),
        );
        self.levels
            .1
            .surfaces
            .iter()
            .enumerate()
            .filter(|(_index, surface)| {
                surface.vector_from(cursor).len() < self.config.snap_distance
            })
            .min_by_key(|(_index, surface)| r32(surface.vector_from(cursor).len()))
            .map(|(index, _surface)| index)
    }

    pub fn find_hovered_tile(&self) -> Option<usize> {
        let p = self.camera.screen_to_world(
            self.framebuffer_size,
            self.geng.window().mouse_pos().map(|x| x as f32),
        );
        'tile_loop: for (index, tile) in self.levels.1.tiles.iter().enumerate() {
            for i in 0..3 {
                let p1 = tile.vertices[i];
                let p2 = tile.vertices[(i + 1) % 3];
                if Vec2::skew(p2 - p1, p - p1) < 0.0 {
                    continue 'tile_loop;
                }
            }
            return Some(index);
        }
        None
    }

    pub fn draw_level_editor(&self, framebuffer: &mut ugli::Framebuffer) {
        if let Some(editor) = &self.editor {
            if let Some(p1) = editor.start_drag {
                let p2 = self.snapped_cursor_position();
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Segment::new(
                        Segment::new(p1, p2),
                        0.1,
                        Rgba::new(1.0, 1.0, 1.0, 0.5),
                    ),
                );
            }
            if let Some(index) = self.find_hovered_surface() {
                let surface = &self.levels.1.surfaces[index];
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Segment::new(
                        Segment::new(surface.p1, surface.p2),
                        0.2,
                        Rgba::new(1.0, 0.0, 0.0, 0.5),
                    ),
                );
            }
            if let Some(index) = self.find_hovered_tile() {
                let tile = &self.levels.1.tiles[index];
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Polygon::new(tile.vertices.into(), Rgba::new(0.0, 0.0, 1.0, 0.5)),
                );
            }
            for &p in &editor.face_points {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Quad::new(
                        AABB::point(p).extend_uniform(0.1),
                        Rgba::new(0.0, 1.0, 0.0, 0.5),
                    ),
                );
            }
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(self.snapped_cursor_position()).extend_uniform(0.1),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );

            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(self.levels.1.spawn_point).extend_uniform(0.1),
                    Rgba::new(1.0, 0.8, 0.8, 0.5),
                ),
            );
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(self.levels.1.finish_point).extend_uniform(0.1),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );

            for (i, &p) in self.levels.1.expected_path.iter().enumerate() {
                self.assets.font.draw(
                    framebuffer,
                    &self.camera,
                    &i.to_string(),
                    p,
                    geng::TextAlign::CENTER,
                    0.1,
                    Rgba::new(0.0, 0.0, 0.0, 0.5),
                );
            }

            if let Some((_, start)) = editor.wind_drag {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Segment::new(
                        Segment::new(
                            start,
                            self.camera.screen_to_world(
                                self.framebuffer_size,
                                self.geng.window().mouse_pos().map(|x| x as f32),
                            ),
                        ),
                        0.2,
                        Rgba::new(1.0, 0.0, 0.0, 0.5),
                    ),
                );
            }
        }
    }

    pub fn update_my_guy_input(&mut self) {
        if self.show_customizer {
            return;
        }
        let my_guy = match self.my_guy.map(|id| self.guys.get_mut(&id).unwrap()) {
            Some(guy) => guy,
            None => return,
        };
        let new_input = Input {
            roll_direction: {
                let mut direction = 0.0;
                if CONTROLS_LEFT
                    .iter()
                    .any(|&key| self.geng.window().is_key_pressed(key))
                {
                    direction += 1.0;
                }
                if CONTROLS_RIGHT
                    .iter()
                    .any(|&key| self.geng.window().is_key_pressed(key))
                {
                    direction -= 1.0;
                }
                direction
            },
            force_fart: CONTROLS_FORCE_FART
                .iter()
                .any(|&key| self.geng.window().is_key_pressed(key)),
        };
        if my_guy.input != new_input {
            my_guy.input = new_input;
            self.connection
                .send(ClientMessage::Update(self.simulation_time, my_guy.clone()));
        }
    }

    pub fn update_guys(&mut self, delta_time: f32) {
        for guy in &mut self.guys {
            if (guy.pos - self.levels.0.finish_point).len() < 1.5 {
                guy.finished = true;
            }
            if !guy.touched_a_unicorn {
                for object in &self.levels.1.objects {
                    if (guy.pos - object.pos).len() < 1.5 && object.type_name == "unicorn" {
                        guy.touched_a_unicorn = true;
                        guy.auto_fart_timer = self.config.auto_fart_interval;
                    }
                }
            }

            if guy.finished {
                guy.auto_fart_timer = 0.0;
                guy.force_fart_timer = 0.0;
                guy.rot -= delta_time;
                guy.pos = self.levels.0.finish_point
                    + (guy.pos - self.levels.0.finish_point)
                        .normalize_or_zero()
                        .rotate(delta_time)
                        * 1.0;
                continue;
            }

            guy.w += (guy.input.roll_direction.clamp(-1.0, 1.0)
                * self.config.angular_acceleration
                * delta_time)
                .clamp(
                    -(guy.w + self.config.max_angular_speed).max(0.0),
                    (self.config.max_angular_speed - guy.w).max(0.0),
                );
            guy.vel.y -= self.config.gravity * delta_time;

            let mut in_water = false;
            let butt = guy.pos + vec2(0.0, -self.config.guy_radius * 0.9).rotate(guy.rot);
            if self.customization.postjam {
                'tile_loop: for tile in self.levels.1.tiles.iter() {
                    for i in 0..3 {
                        let p1 = tile.vertices[i];
                        let p2 = tile.vertices[(i + 1) % 3];
                        if Vec2::skew(p2 - p1, guy.pos - p1) < 0.0 {
                            continue 'tile_loop;
                        }
                    }
                    let relative_vel = guy.vel - tile.flow;
                    let flow_direction = tile.flow.normalize_or_zero();
                    let relative_vel_along_flow = Vec2::dot(flow_direction, relative_vel);
                    let params = &self.assets.tiles[&tile.type_name].params;
                    let force_along_flow =
                        -flow_direction * relative_vel_along_flow * params.friction_along_flow;
                    let friction_force = -relative_vel * params.friction;
                    guy.vel +=
                        (force_along_flow + params.additional_force + friction_force) * delta_time;
                    guy.w -= guy.w * params.friction * delta_time;
                }
                'tile_loop: for tile in self.levels.1.tiles.iter() {
                    for i in 0..3 {
                        let p1 = tile.vertices[i];
                        let p2 = tile.vertices[(i + 1) % 3];
                        if Vec2::skew(p2 - p1, butt - p1) < 0.0 {
                            continue 'tile_loop;
                        }
                    }
                    if tile.type_name == "water" {
                        in_water = true;
                    }
                }
            }

            let mut farts = 0;
            guy.auto_fart_timer += delta_time;
            if guy.auto_fart_timer >= self.config.auto_fart_interval {
                guy.auto_fart_timer = 0.0;
                farts += 1;
            }
            let could_force_fart = guy.force_fart_timer >= self.config.force_fart_interval;
            guy.force_fart_timer += delta_time;
            if guy.force_fart_timer >= self.config.force_fart_interval && guy.input.force_fart {
                farts += 1;
                guy.force_fart_timer = 0.0;
            }
            if !could_force_fart && guy.force_fart_timer >= self.config.force_fart_interval {
                if Some(guy.id) == self.my_guy {
                    let mut effect = self.assets.sfx.fart_recharge.effect();
                    effect.set_volume(self.volume as f64 * 0.5);
                    effect.play();
                }
            }
            for _ in 0..farts {
                for _ in 0..self.config.farticle_count {
                    self.farticles.push(Farticle {
                        size: 1.0,
                        pos: butt,
                        vel: guy.vel
                            + vec2(
                                global_rng().gen_range(0.0..=self.config.farticle_additional_vel),
                                0.0,
                            )
                            .rotate(global_rng().gen_range(0.0..=2.0 * f32::PI)),
                        rot: global_rng().gen_range(0.0..2.0 * f32::PI),
                        w: global_rng().gen_range(-self.config.farticle_w..=self.config.farticle_w),
                        color: if in_water {
                            self.config.bubble_fart_color
                        } else if guy.touched_a_unicorn {
                            Hsva::new(global_rng().gen_range(0.0..1.0), 1.0, 1.0, 0.5).into()
                        } else {
                            self.config.fart_color
                        },
                        t: 1.0,
                    });
                }
                guy.vel += vec2(0.0, self.config.fart_strength).rotate(guy.rot);
                let sounds = if in_water {
                    &self.assets.sfx.bubble_fart
                } else if guy.touched_a_unicorn {
                    &self.assets.sfx.rainbow_fart
                } else {
                    &self.assets.sfx.fart
                };
                let mut effect = sounds.choose(&mut global_rng()).unwrap().effect();
                effect.set_volume(
                    (self.volume * (1.0 - (guy.pos - self.camera.center).len() / self.camera.fov))
                        .clamp(0.0, 1.0) as f64,
                );
                effect.play();
            }

            guy.pos += guy.vel * delta_time;
            guy.rot += guy.w * delta_time;

            struct Collision<'a> {
                penetration: f32,
                normal: Vec2<f32>,
                assets: &'a SurfaceAssets,
            }

            let mut collision_to_resolve = None;
            let level = if self.customization.postjam {
                &self.levels.1
            } else {
                &self.levels.0
            };
            let mut was_colliding_water = guy.colliding_water;
            guy.colliding_water = false;
            for surface in &level.surfaces {
                let v = surface.vector_from(guy.pos);
                let penetration = self.config.guy_radius - v.len();
                if penetration > EPS {
                    let assets = &self.assets.surfaces[&surface.type_name];

                    if surface.type_name == "water" {
                        guy.colliding_water = true;
                        if !was_colliding_water {
                            was_colliding_water = true;
                            if Vec2::dot(v, guy.vel).abs() > 0.5 {
                                let mut effect = self.assets.sfx.water_splash.effect();
                                effect.set_volume(
                                    (self.volume
                                        * 0.6
                                        * (1.0
                                            - (guy.pos - self.camera.center).len()
                                                / self.camera.fov))
                                        .clamp(0.0, 1.0) as f64,
                                );
                                effect.play();
                                for _ in 0..30 {
                                    self.farticles.push(Farticle {
                                        size: 0.6,
                                        pos: guy.pos
                                            + v
                                            + vec2(
                                                global_rng().gen_range(
                                                    -self.config.guy_radius
                                                        ..=self.config.guy_radius,
                                                ),
                                                0.0,
                                            ),
                                        vel: {
                                            let mut v =
                                                vec2(0.0, global_rng().gen_range(0.0..=1.0))
                                                    .rotate(
                                                        global_rng().gen_range(
                                                            -f32::PI / 4.0..=f32::PI / 4.0,
                                                        ),
                                                    );
                                            v.y *= 0.3;
                                            v * 2.0
                                        },
                                        rot: global_rng().gen_range(0.0..2.0 * f32::PI),
                                        w: global_rng().gen_range(
                                            -self.config.farticle_w..=self.config.farticle_w,
                                        ),
                                        color: self.config.bubble_fart_color,
                                        t: 0.5,
                                    });
                                }
                            }
                        }
                    }

                    if assets.params.non_collidable {
                        continue;
                    }
                    if Vec2::dot(v, guy.vel) > EPS {
                        let collision = Collision {
                            penetration,
                            normal: -v.normalize_or_zero(),
                            assets,
                        };
                        collision_to_resolve = std::cmp::max_by_key(
                            collision_to_resolve,
                            Some(collision),
                            |collision| {
                                r32(match collision {
                                    Some(collision) => collision.penetration,
                                    None => -1.0,
                                })
                            },
                        );
                    }
                }
            }
            if let Some(collision) = collision_to_resolve {
                guy.pos += collision.normal * collision.penetration;
                let normal_vel = Vec2::dot(guy.vel, collision.normal);
                let tangent = collision.normal.rotate_90();
                let tangent_vel = Vec2::dot(guy.vel, tangent) - guy.w * self.config.guy_radius;
                guy.vel -=
                    collision.normal * normal_vel * (1.0 + collision.assets.params.bounciness);
                let max_friction_impulse = normal_vel.abs() * collision.assets.params.friction;
                let friction_impulse = -tangent_vel.clamp_abs(max_friction_impulse);
                guy.vel += tangent * friction_impulse;
                guy.w -= friction_impulse / self.config.guy_radius;
                if let Some(sound) = &collision.assets.sound {
                    let volume = ((-0.5 - normal_vel) / 2.0).clamp(0.0, 1.0);
                    if volume > 0.0 && self.customization.postjam {
                        let mut effect = sound.effect();
                        effect.set_volume(
                            (self.volume
                                * volume
                                * (1.0 - (guy.pos - self.camera.center).len() / self.camera.fov))
                                .clamp(0.0, 1.0) as f64,
                        );
                        effect.play();
                    }
                }
            }
        }
    }

    #[track_caller]
    pub fn noise(&self, frequency: f32) -> f32 {
        let caller = std::panic::Location::caller();
        let phase = caller.line() as f64 * 1000.0 + caller.column() as f64;
        self.noise.get([(self.real_time * frequency) as f64, phase]) as f32
    }

    pub fn handle_event_editor(&mut self, event: &geng::Event) {
        if self.opt.editor
            && matches!(
                event,
                geng::Event::KeyDown {
                    key: geng::Key::Tab
                }
            )
        {
            if self.editor.is_none() {
                self.editor = Some(EditorState::new());
            } else {
                self.editor = None;
            }
        }
        if self.editor.is_none() {
            return;
        }
        let cursor_pos = self.snapped_cursor_position();
        let editor = self.editor.as_mut().unwrap();

        if !self.assets.surfaces.contains_key(&editor.selected_surface) {
            editor.selected_surface = self.assets.surfaces.keys().next().unwrap().clone();
        }
        if !self.assets.tiles.contains_key(&editor.selected_tile) {
            editor.selected_tile = self.assets.tiles.keys().next().unwrap().clone();
        }
        if !self.assets.objects.contains_key(&editor.selected_object) {
            editor.selected_object = self.assets.objects.keys().next().unwrap().clone();
        }

        match event {
            geng::Event::MouseDown {
                button: geng::MouseButton::Left,
                ..
            } => {
                if let Some(editor) = &mut self.editor {
                    editor.start_drag = Some(cursor_pos);
                }
            }
            geng::Event::MouseUp {
                button: geng::MouseButton::Left,
                ..
            } => {
                let p2 = cursor_pos;

                if let Some(p1) = editor.start_drag.take() {
                    if (p1 - p2).len() > self.config.snap_distance {
                        self.levels.1.surfaces.push(Surface {
                            p1,
                            p2,
                            type_name: editor.selected_surface.clone(),
                        });
                    }
                }
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some(index) = self.find_hovered_surface() {
                    self.levels.1.surfaces.remove(index);
                }
            }
            geng::Event::KeyUp { key } => match key {
                geng::Key::W => {
                    if let Some((index, start)) = editor.wind_drag.take() {
                        let to = self.camera.screen_to_world(
                            self.framebuffer_size,
                            self.geng.window().mouse_pos().map(|x| x as f32),
                        );
                        self.levels.1.tiles[index].flow = to - start;
                    }
                }
                _ => {}
            },
            geng::Event::KeyDown { key } => match key {
                geng::Key::W => {
                    if editor.wind_drag.is_none() {
                        self.editor.as_mut().unwrap().wind_drag =
                            self.find_hovered_tile().map(|index| {
                                (
                                    index,
                                    self.camera.screen_to_world(
                                        self.framebuffer_size,
                                        self.geng.window().mouse_pos().map(|x| x as f32),
                                    ),
                                )
                            });
                    }
                }
                geng::Key::F => {
                    editor.face_points.push(cursor_pos);
                    if editor.face_points.len() == 3 {
                        let mut vertices: [Vec2<f32>; 3] =
                            mem::take(&mut editor.face_points).try_into().unwrap();
                        if Vec2::skew(vertices[1] - vertices[0], vertices[2] - vertices[0]) < 0.0 {
                            vertices.reverse();
                        }
                        self.levels.1.tiles.push(Tile {
                            vertices,
                            flow: Vec2::ZERO,
                            type_name: editor.selected_tile.clone(),
                        });
                    }
                }
                geng::Key::D => {
                    if let Some(index) = self.find_hovered_tile() {
                        self.levels.1.tiles.remove(index);
                    }
                }
                geng::Key::C => {
                    editor.face_points.clear();
                }
                geng::Key::R => {
                    if !self.geng.window().is_key_pressed(geng::Key::LCtrl) {
                        if let Some(id) = self.my_guy.take() {
                            self.connection.send(ClientMessage::Despawn);
                            self.guys.remove(&id);
                        } else {
                            self.my_guy = Some(self.client_id);
                            self.guys
                                .insert(Guy::new(self.client_id, cursor_pos, false));
                        }
                    }
                }
                geng::Key::P => {
                    self.levels.1.spawn_point = self.camera.screen_to_world(
                        self.framebuffer_size,
                        self.geng.window().mouse_pos().map(|x| x as f32),
                    );
                }
                geng::Key::I => {
                    let level = if self.customization.postjam {
                        &mut self.levels.1
                    } else {
                        &mut self.levels.0
                    };
                    level.expected_path.push(self.camera.screen_to_world(
                        self.framebuffer_size,
                        self.geng.window().mouse_pos().map(|x| x as f32),
                    ));
                }
                geng::Key::O => {
                    let level = if self.customization.postjam {
                        &mut self.levels.1
                    } else {
                        &mut self.levels.0
                    };
                    level.objects.push(Object {
                        type_name: editor.selected_object.to_owned(),
                        pos: self.camera.screen_to_world(
                            self.framebuffer_size,
                            self.geng.window().mouse_pos().map(|x| x as f32),
                        ),
                    });
                }
                geng::Key::Backspace => {
                    let level = if self.customization.postjam {
                        &mut self.levels.1
                    } else {
                        &mut self.levels.0
                    };
                    level.expected_path.pop();
                }
                geng::Key::K => {
                    // self.level.finish_point = self.camera.screen_to_world(
                    //     self.framebuffer_size,
                    //     self.geng.window().mouse_pos().map(|x| x as f32),
                    // );
                }
                geng::Key::Z => {
                    let mut options: Vec<&String> = self.assets.surfaces.keys().collect();
                    options.sort();
                    let idx = options
                        .iter()
                        .position(|&s| s == &editor.selected_surface)
                        .unwrap_or(0);
                    editor.selected_surface = options[(idx + 1) % options.len()].clone();
                }
                geng::Key::X => {
                    let mut options: Vec<&String> = self.assets.tiles.keys().collect();
                    options.sort();
                    let idx = options
                        .iter()
                        .position(|&s| s == &editor.selected_tile)
                        .unwrap_or(0);
                    editor.selected_tile = options[(idx + 1) % options.len()].clone();
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn save_level(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.editor.is_some() {
            serde_json::to_writer_pretty(
                std::fs::File::create(static_path().join("new_level.json")).unwrap(),
                &self.levels.1,
            )
            .unwrap();
            info!("LVL SAVED");
        }
    }

    pub fn update_farticles(&mut self, delta_time: f32) {
        for farticle in &mut self.farticles {
            farticle.t -= delta_time;
            farticle.pos += farticle.vel * delta_time;
            farticle.rot += farticle.w * delta_time;

            let level = if self.customization.postjam {
                &self.levels.1
            } else {
                &self.levels.0
            };
            for surface in &level.surfaces {
                let v = surface.vector_from(farticle.pos);
                let penetration = self.config.farticle_size / 2.0 - v.len();
                if penetration > EPS && Vec2::dot(v, farticle.vel) > 0.0 {
                    let normal = -v.normalize_or_zero();
                    farticle.pos += normal * penetration;
                    farticle.vel -= normal * Vec2::dot(farticle.vel, normal);
                }
            }
        }
        self.farticles.retain(|farticle| farticle.t > 0.0);
    }

    pub fn draw_farticles(&self, framebuffer: &mut ugli::Framebuffer) {
        for farticle in &self.farticles {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(
                    &self.assets.farticle,
                    Rgba {
                        a: farticle.color.a * farticle.t,
                        ..farticle.color
                    },
                )
                .transform(Mat3::rotate(farticle.rot))
                .scale_uniform(self.config.farticle_size * farticle.size)
                .translate(farticle.pos),
            )
        }
    }

    pub fn handle_connection(&mut self) {
        let messages: Vec<ServerMessage> = self.connection.new_messages().collect();
        for message in messages {
            match message {
                ServerMessage::ForceReset => {
                    if self.my_guy.is_some() {
                        // COPYPASTA mmmmmmm
                        let new_guy = Guy::new(
                            self.client_id,
                            if self.customization.postjam {
                                self.levels.1.spawn_point
                            } else {
                                self.levels.0.spawn_point
                            },
                            true,
                        );
                        if self.my_guy.is_none() {
                            self.my_guy = Some(self.client_id);
                        }
                        self.guys.insert(new_guy);
                        self.simulation_time = 0.0;
                        self.connection.send(ClientMessage::Despawn);
                    }
                }
                ServerMessage::Pong => {
                    self.connection.send(ClientMessage::Ping);
                    if let Some(id) = self.my_guy {
                        let guy = self.guys.get(&id).unwrap();
                        self.connection
                            .send(ClientMessage::Update(self.simulation_time, guy.clone()));
                    }
                }
                ServerMessage::ClientId(_) => unreachable!(),
                ServerMessage::UpdateGuy(t, guy) => {
                    if !self.remote_simulation_times.contains_key(&guy.id) {
                        self.remote_simulation_times.insert(guy.id, t - 1.0);
                    }
                    self.remote_updates
                        .entry(guy.id)
                        .or_default()
                        .push_back((t, guy));
                }
                ServerMessage::Despawn(id) => {
                    self.guys.remove(&id);
                    self.remote_simulation_times.remove(&id);
                    if let Some(updates) = self.remote_updates.get_mut(&id) {
                        updates.clear();
                    }
                }
                ServerMessage::Emote(id, emote) => {
                    self.emotes.retain(|&(_, x, _)| x != id);
                    self.emotes.push((self.real_time, id, emote));
                }
            }
        }
    }

    fn update_remote(&mut self) {
        for (&id, updates) in &mut self.remote_updates {
            let current_simulation_time = match self.remote_simulation_times.get(&id) {
                Some(x) => *x,
                None => continue,
            };
            if let Some(update) = updates.back() {
                if (update.0 - current_simulation_time).abs() > 5.0 {
                    updates.clear();
                    self.remote_simulation_times.remove(&id);
                    self.guys.remove(&id);
                    continue;
                }
            }
            while let Some(update) = updates.front() {
                if (update.0 - current_simulation_time).abs() > 5.0 {
                    updates.clear();
                    self.remote_simulation_times.remove(&id);
                    self.guys.remove(&id);
                    break;
                }
                if update.0 <= current_simulation_time {
                    let update = updates.pop_front().unwrap().1;
                    self.guys.insert(update);
                } else {
                    break;
                }
            }
        }
    }

    pub fn respawn(&mut self) {
        // COPYPASTA MMMMM 🍝
        let new_guy = Guy::new(
            self.client_id,
            if self.customization.postjam {
                self.levels.1.spawn_point
            } else {
                self.levels.0.spawn_point
            },
            true,
        );
        if self.my_guy.is_none() {
            self.my_guy = Some(self.client_id);
        }
        self.guys.insert(new_guy);
        self.simulation_time = 0.0;
        self.connection.send(ClientMessage::Despawn);
    }

    pub fn draw_customizer(&mut self, framebuffer: &mut ugli::Framebuffer) {
        if !self.show_customizer {
            return;
        }
        let camera = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 10.0,
        };
        self.ui_controller
            .draw(framebuffer, &camera, self.buttons.clone());
        if self.customization.name.is_empty() {
            self.assets.font.draw(
                framebuffer,
                &camera,
                "type your name",
                vec2(0.0, 3.0),
                geng::TextAlign::CENTER,
                1.0,
                Rgba::new(0.5, 0.5, 1.0, 0.5),
            );
            self.assets.font.draw(
                framebuffer,
                &camera,
                "yes just type it",
                vec2(0.0, 2.0),
                geng::TextAlign::CENTER,
                1.0,
                Rgba::new(0.5, 0.5, 1.0, 0.5),
            );
        } else {
            self.assets.font.draw(
                framebuffer,
                &camera,
                &self.customization.name,
                vec2(0.0, 3.0),
                geng::TextAlign::CENTER,
                1.0,
                Rgba::new(0.5, 0.5, 1.0, 1.0),
            );
        }
    }

    fn handle_customizer_event(&mut self, event: &geng::Event) {
        if !self.show_customizer {
            return;
        }
        for msg in self.ui_controller.handle_event(event, self.buttons.clone()) {
            match msg {
                UiMessage::Play => {
                    self.show_customizer = false;
                }
                UiMessage::RandomizeSkin => {
                    self.customization.colors = Guy::new(-1, Vec2::ZERO, true).colors;
                }
                UiMessage::TogglePostJam => {
                    if self.customization.postjam {
                        self.customization.postjam = false;
                    } else {
                        self.customization.postjam = true;
                    }
                    self.buttons
                        .iter_mut()
                        .find(|button| button.text.starts_with("postjam"))
                        .unwrap()
                        .text = format!(
                        "postjam ({})",
                        if self.customization.postjam {
                            "on"
                        } else {
                            "off"
                        }
                    );

                    self.respawn();
                }
            }
        }
        match event {
            geng::Event::KeyDown { key } => {
                let s = format!("{:?}", key);
                if s.len() == 1 && self.customization.name.len() < 15 {
                    self.customization.name.push_str(&s);
                }
                if *key == geng::Key::Backspace {
                    self.customization.name.pop();
                }
            }
            _ => {}
        }
    }

    fn draw_leaderboard(&self, framebuffer: &mut ugli::Framebuffer) {
        if !self.show_leaderboard || !self.customization.postjam {
            return;
        }
        let mut guys: Vec<&Guy> = self.guys.iter().filter(|guy| guy.postjam).collect();
        guys.sort_by(|a, b| match (a.best_time, b.best_time) {
            (Some(a), Some(b)) => a.partial_cmp(&b).unwrap(),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a
                .best_progress
                .partial_cmp(&b.best_progress)
                .unwrap()
                .reverse(),
        });
        let mut camera = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 40.0,
        };
        camera.center.x += camera.fov * self.framebuffer_size.x / self.framebuffer_size.y / 2.0;
        for (place, guy) in guys.into_iter().enumerate() {
            let place = place + 1;
            let name = &guy.name;
            let progress = (guy.progress * 100.0).round() as i32;
            let mut text = format!("#{place}: {name} - {progress}% (");
            if let Some(time) = guy.best_time {
                let millis = (time * 1000.0).round() as i32;
                let seconds = millis / 1000;
                let millis = millis % 1000;
                let minutes = seconds / 60;
                let seconds = seconds % 60;
                let hours = minutes / 60;
                let minutes = minutes % 60;
                if hours != 0 {
                    text += &format!("{}:", hours);
                }
                if minutes != 0 {
                    text += &format!("{}:", minutes);
                }
                text += &format!("{}.{}", seconds, millis);
            } else {
                text += &format!("{}%", (guy.best_progress * 100.0).round() as i32);
            }
            text.push(')');
            self.geng.default_font().draw(
                framebuffer,
                &camera,
                &text,
                vec2(1.0, camera.fov / 2.0 - place as f32),
                geng::TextAlign::LEFT,
                1.0,
                Rgba::BLACK,
            );
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(self.config.background_color), None, None);

        self.draw_level_back(framebuffer);
        self.draw_guys(framebuffer);
        self.draw_level_front(framebuffer);
        self.draw_farticles(framebuffer);
        self.draw_level_editor(framebuffer);

        self.draw_customizer(framebuffer);

        self.draw_leaderboard(framebuffer);

        if !self.show_customizer {
            if let Some(id) = self.my_guy {
                let camera = geng::Camera2d {
                    center: Vec2::ZERO,
                    rotation: 0.0,
                    fov: 10.0,
                };
                let guy = self.guys.get_mut(&id).unwrap();
                if guy.finished {
                    self.assets.font.draw(
                        framebuffer,
                        &camera,
                        &"GG",
                        vec2(0.0, 3.0),
                        geng::TextAlign::CENTER,
                        1.5,
                        Rgba::BLACK,
                    );
                }
                let progress = {
                    let level = if self.customization.postjam {
                        &self.levels.1
                    } else {
                        &self.levels.0
                    };
                    let mut total_len = 0.0;
                    for window in level.expected_path.windows(2) {
                        let a = window[0];
                        let b = window[1];
                        total_len += (b - a).len();
                    }
                    let mut progress = 0.0;
                    let mut closest_point_distance = 1e9;
                    let mut prefix_len = 0.0;
                    for window in level.expected_path.windows(2) {
                        let a = window[0];
                        let b = window[1];
                        let v = Surface {
                            p1: a,
                            p2: b,
                            type_name: String::new(),
                        }
                        .vector_from(guy.pos);
                        if v.len() < closest_point_distance {
                            closest_point_distance = v.len();
                            progress = (prefix_len + (guy.pos + v - a).len()) / total_len;
                        }
                        prefix_len += (b - a).len();
                    }
                    progress
                };
                guy.progress = progress;
                self.best_progress = self.best_progress.max(progress);
                guy.best_progress = self.best_progress;
                if guy.finished && self.simulation_time < self.best_time.unwrap_or(1e9) {
                    self.best_time = Some(self.simulation_time);
                }
                guy.best_time = self.best_time;
                let mut time_text = String::new();
                let seconds = self.simulation_time.round() as i32;
                let minutes = seconds / 60;
                let seconds = seconds % 60;
                let hours = minutes / 60;
                let minutes = minutes % 60;
                if hours != 0 {
                    time_text += &format!("{} hours ", hours);
                }
                if minutes != 0 {
                    time_text += &format!("{} minutes ", minutes);
                }
                time_text += &format!("{} seconds", seconds);
                self.assets.font.draw(
                    framebuffer,
                    &camera,
                    &time_text,
                    vec2(0.0, -3.3),
                    geng::TextAlign::CENTER,
                    0.5,
                    Rgba::BLACK,
                );
                self.assets.font.draw(
                    framebuffer,
                    &camera,
                    &"progress",
                    vec2(0.0, -4.0),
                    geng::TextAlign::CENTER,
                    0.5,
                    Rgba::BLACK,
                );
                self.geng.draw_2d(
                    framebuffer,
                    &camera,
                    &draw_2d::Quad::new(
                        AABB::point(vec2(0.0, -4.5)).extend_symmetric(vec2(3.0, 0.1)),
                        Rgba::BLACK,
                    ),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &camera,
                    &draw_2d::Quad::new(
                        AABB::point(vec2(-3.0 + 6.0 * self.best_progress, -4.5))
                            .extend_uniform(0.3),
                        Rgba::new(0.0, 0.0, 0.0, 0.5),
                    ),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &camera,
                    &draw_2d::Quad::new(
                        AABB::point(vec2(-3.0 + 6.0 * progress, -4.5)).extend_uniform(0.3),
                        Rgba::BLACK,
                    ),
                );
            }
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        if self.my_guy.is_none() || !self.guys.get(&self.my_guy.unwrap()).unwrap().finished {
            self.simulation_time += delta_time;
        }
        for time in self.remote_simulation_times.values_mut() {
            *time += delta_time;
        }
        self.update_my_guy_input();
        self.update_guys(delta_time);
        self.update_farticles(delta_time);
    }

    fn update(&mut self, delta_time: f64) {
        // self.volume = self.assets.config.volume;
        if self.geng.window().is_key_pressed(geng::Key::PageUp) {
            self.volume += delta_time as f32 * 0.5;
        }
        if self.geng.window().is_key_pressed(geng::Key::PageDown) {
            self.volume -= delta_time as f32 * 0.5;
        }
        self.volume = self.volume.clamp(0.0, 1.0);
        if self.customization.postjam {
            self.new_music.set_volume(self.volume as f64);
            self.old_music.set_volume(0.0);
        } else {
            self.old_music.set_volume(self.volume as f64);
            self.new_music.set_volume(0.0);
        }
        self.emotes.retain(|&(t, ..)| t >= self.real_time - 1.0);
        let delta_time = delta_time as f32;
        self.real_time += delta_time;

        let mut target_center = self.camera.center;
        if let Some(id) = self.my_guy {
            let guy = self.guys.get(&id).unwrap();
            target_center = guy.pos;
            if self.show_customizer {
                target_center.x += 1.0;
            }
        } else if let Some(id) = self.follow {
            if let Some(guy) = self.guys.get(&id) {
                target_center = guy.pos;
            }
        }
        self.camera.center += (target_center - self.camera.center) * (delta_time * 5.0).min(1.0);

        if self.editor.is_none() {
            // let target_fov = if self.show_customizer { 2.0 } else { 6.0 };
            // self.camera.fov += (target_fov - self.camera.fov) * delta_time;
        }

        if let Some(editor) = &mut self.editor {
            editor.next_autosave -= delta_time;
            if editor.next_autosave < 0.0 {
                editor.next_autosave = 10.0;
                self.save_level();
            }
        }

        self.handle_connection();
        self.update_remote();

        if let Some(id) = self.my_guy {
            let guy = self.guys.get_mut(&id).unwrap();
            guy.name = self.customization.name.clone();
            guy.colors = self.customization.colors.clone();
            guy.postjam = self.customization.postjam;
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event_editor(&event);
        self.handle_customizer_event(&event);
        match event {
            geng::Event::MouseMove { position, .. }
                if self
                    .geng
                    .window()
                    .is_button_pressed(geng::MouseButton::Middle) =>
            {
                let old_pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, self.prev_mouse_pos.map(|x| x as f32));
                let new_pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                self.camera.center += old_pos - new_pos;
            }
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } if self.my_guy.is_none() && self.editor.is_none() => {
                let pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                if let Some(guy) = self
                    .guys
                    .iter()
                    .min_by_key(|guy| r32((guy.pos - pos).len()))
                {
                    if (guy.pos - pos).len() < self.assets.config.guy_radius {
                        self.follow = Some(guy.id);
                    }
                }
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                self.follow = None;
            }
            geng::Event::Wheel { delta } if self.opt.editor => {
                self.camera.fov = (self.camera.fov * 1.01f32.powf(-delta as f32)).clamp(1.0, 30.0);
            }
            geng::Event::KeyDown { key: geng::Key::S }
                if self.geng.window().is_key_pressed(geng::Key::LCtrl) =>
            {
                self.save_level();
            }
            geng::Event::KeyDown { key: geng::Key::R }
                if self.geng.window().is_key_pressed(geng::Key::LCtrl) =>
            {
                self.respawn();
            }
            geng::Event::KeyDown { key: geng::Key::H } if !self.show_customizer => {
                self.show_names = !self.show_names;
            }
            geng::Event::KeyDown { key: geng::Key::L } if !self.show_customizer => {
                if self.customization.postjam {
                    self.show_leaderboard = !self.show_leaderboard;
                }
            }
            geng::Event::KeyDown {
                key: geng::Key::Num1,
            } => self.connection.send(ClientMessage::Emote(0)),
            geng::Event::KeyDown {
                key: geng::Key::Num2,
            } => self.connection.send(ClientMessage::Emote(1)),
            geng::Event::KeyDown {
                key: geng::Key::Num3,
            } => self.connection.send(ClientMessage::Emote(2)),
            geng::Event::KeyDown {
                key: geng::Key::Num4,
            } => self.connection.send(ClientMessage::Emote(3)),
            _ => {}
        }
        self.prev_mouse_pos = self.geng.window().mouse_pos();
    }
}