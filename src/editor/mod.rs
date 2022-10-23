use super::*;

pub struct EditorState {
    pub next_autosave: f32,
    pub start_drag: Option<Vec2<f32>>,
    pub face_points: Vec<Vec2<f32>>,
    pub selected_surface: String,
    pub selected_tile: String,
    pub wind_drag: Option<(usize, Vec2<f32>)>,
    pub selected_object: String,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            next_autosave: 0.0,
            start_drag: None,
            face_points: vec![],
            selected_surface: "".to_owned(),
            selected_tile: "".to_owned(),
            selected_object: "".to_owned(),
            wind_drag: None,
        }
    }
}

impl Game {
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
}
