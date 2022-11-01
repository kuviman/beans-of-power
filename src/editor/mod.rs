use super::*;

mod tools;

use tools::*;

pub struct Cursor {
    pub screen_pos: Vec2<f32>,
    pub world_pos: Vec2<f32>,
    pub snapped_world_pos: Vec2<f32>,
}

pub struct EditorState {
    geng: Geng,
    cursor: Cursor,
    next_autosave: f32,
    selected_object: String,
    tool: Box<dyn DynEditorTool>,
}

impl EditorState {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            cursor: Cursor {
                screen_pos: Vec2::ZERO,
                world_pos: Vec2::ZERO,
                snapped_world_pos: Vec2::ZERO,
            },
            next_autosave: 0.0,
            selected_object: "".to_owned(),
            // tool: Box::new(SurfaceTool::new(
            //     geng,
            //     assets,
            //     SurfaceToolConfig::default(assets),
            // )),
            tool: Box::new(TileTool::new(geng, assets, TileToolConfig::default(assets))),
        }
    }
    pub fn update(&mut self, levels: &mut Levels, delta_time: f32) {
        self.next_autosave -= delta_time;
        if self.next_autosave < 0.0 {
            self.next_autosave = 10.0;
            self.save_level(levels);
        }
    }

    pub fn save_level(&self, levels: &Levels) {
        #[cfg(not(target_arch = "wasm32"))]
        serde_json::to_writer_pretty(
            std::fs::File::create(static_path().join("new_level.json")).unwrap(),
            &levels.postjam,
        )
        .unwrap();
        info!("LVL SAVED");
    }
}

impl Game {
    pub fn snapped_cursor_position(&self, level: &Level) -> Vec2<f32> {
        self.snap_position(
            level,
            self.camera.screen_to_world(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            ),
        )
    }

    pub fn snap_position(&self, level: &Level, pos: Vec2<f32>) -> Vec2<f32> {
        let closest_point = itertools::chain![
            level
                .surfaces
                .iter()
                .flat_map(|surface| [surface.p1, surface.p2]),
            level.tiles.iter().flat_map(|tile| tile.vertices)
        ]
        .filter(|&p| (pos - p).len() < self.config.snap_distance)
        .min_by_key(|&p| r32((pos - p).len()));
        closest_point.unwrap_or(pos)
    }

    pub fn draw_level_editor(&self, framebuffer: &mut ugli::Framebuffer) {
        let level = &self.levels.postjam;
        if let Some(editor) = &self.editor {
            editor
                .tool
                .draw(&editor.cursor, level, &self.camera, framebuffer);
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(self.snapped_cursor_position(level)).extend_uniform(0.1),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );

            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(level.spawn_point).extend_uniform(0.1),
                    Rgba::new(1.0, 0.8, 0.8, 0.5),
                ),
            );
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(level.finish_point).extend_uniform(0.1),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );

            for (i, &p) in level.expected_path.iter().enumerate() {
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
        }
    }

    pub fn handle_event_editor(&mut self, event: &geng::Event) {
        macro_rules! level_mut {
            () => {{
                self.levels.postjam.mesh.take();
                &mut self.levels.postjam
            }};
        }
        if self.opt.editor
            && matches!(
                event,
                geng::Event::KeyDown {
                    key: geng::Key::Tab
                }
            )
        {
            if self.editor.is_none() {
                self.editor = Some(EditorState::new(&self.geng, &self.assets));
            } else {
                self.editor = None;
            }
        }
        if self.editor.is_none() {
            return;
        }
        let cursor_pos = self.snapped_cursor_position(&self.levels.postjam);
        let editor = self.editor.as_mut().unwrap();
        editor.cursor = Cursor {
            screen_pos: self.geng.window().mouse_pos().map(|x| x as f32),
            world_pos: self.camera.screen_to_world(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            ),
            snapped_world_pos: cursor_pos,
        };

        if !self.assets.objects.contains_key(&editor.selected_object) {
            editor.selected_object = self.assets.objects.keys().next().unwrap().clone();
        }

        editor
            .tool
            .handle_event(&editor.cursor, event, level_mut!());

        match event {
            geng::Event::KeyDown { key } => match key {
                geng::Key::R => {
                    if !self.geng.window().is_key_pressed(geng::Key::LCtrl) {
                        if let Some(id) = self.my_guy.take() {
                            if let Some(con) = &mut self.connection {
                                con.send(ClientMessage::Despawn);
                            }
                            self.guys.remove(&id);
                        } else {
                            self.my_guy = Some(self.client_id);
                            self.guys
                                .insert(Guy::new(self.client_id, cursor_pos, false));
                        }
                    }
                }
                geng::Key::P => {
                    level_mut!().spawn_point = self.camera.screen_to_world(
                        self.framebuffer_size,
                        self.geng.window().mouse_pos().map(|x| x as f32),
                    );
                }
                geng::Key::I => {
                    level_mut!().expected_path.push(self.camera.screen_to_world(
                        self.framebuffer_size,
                        self.geng.window().mouse_pos().map(|x| x as f32),
                    ));
                }
                geng::Key::O => {
                    level_mut!().objects.push(Object {
                        type_name: editor.selected_object.to_owned(),
                        pos: self.camera.screen_to_world(
                            self.framebuffer_size,
                            self.geng.window().mouse_pos().map(|x| x as f32),
                        ),
                    });
                }
                geng::Key::Backspace => {
                    level_mut!().expected_path.pop();
                }
                geng::Key::K => {
                    level_mut!().finish_point = self.camera.screen_to_world(
                        self.framebuffer_size,
                        self.geng.window().mouse_pos().map(|x| x as f32),
                    );
                }
                geng::Key::S if self.geng.window().is_key_pressed(geng::Key::LCtrl) => {
                    editor.save_level(&self.levels);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
