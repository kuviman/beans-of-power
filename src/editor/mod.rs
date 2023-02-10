use super::*;

mod tools;

use tools::*;

pub struct Cursor {
    pub screen_pos: vec2<f32>,
    pub world_pos: vec2<f32>,
    pub snapped_world_pos: vec2<f32>,
}

pub struct EditorState {
    geng: Geng,
    cursor: Cursor,
    next_autosave: f32,
    available_tools: Vec<Box<dyn ToolConstructor>>,
    selected_tool_index: usize,
    selected_layer: usize,
    tool: Box<dyn DynEditorTool>,
}

impl EditorState {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        let available_tools = vec![
            tool_constructor::<EditTool>(geng, assets),
            tool_constructor::<SurfaceTool>(geng, assets),
            tool_constructor::<TileTool>(geng, assets),
            tool_constructor::<ObjectTool>(geng, assets),
            tool_constructor::<EndpointTool>(geng, assets),
            tool_constructor::<ProgressTool>(geng, assets),
            tool_constructor::<CannonTool>(geng, assets),
            tool_constructor::<PortalTool>(geng, assets),
        ];
        let selected_tool_index = 0;
        Self {
            geng: geng.clone(),
            cursor: Cursor {
                screen_pos: vec2::ZERO,
                world_pos: vec2::ZERO,
                snapped_world_pos: vec2::ZERO,
            },
            next_autosave: 0.0,
            selected_tool_index,
            selected_layer: 0,
            tool: available_tools[selected_tool_index].create(),
            available_tools,
        }
    }
    pub fn update(&mut self, level: &mut Level, delta_time: f32) {
        self.next_autosave -= delta_time;
        if self.next_autosave < 0.0 {
            self.next_autosave = 10.0;
            self.save_level(level);
        }
    }

    pub fn save_level(&self, level: &mut Level) {
        if level.save() {
            #[cfg(not(target_arch = "wasm32"))]
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(
                    std::fs::File::create(run_dir().join("assets").join("level.json")).unwrap(),
                ),
                level.info(),
            )
            .unwrap();
            info!("LVL SAVED");
        }
    }
}

impl Game {
    pub fn snapped_cursor_position(&self, level: &Level) -> vec2<f32> {
        let Some(editor) = &self.editor else { return vec2::ZERO; };
        let camera = geng::Camera2d {
            center: self.camera.center * level.layers[editor.selected_layer].parallax,
            ..self.camera
        };
        self.snap_position(
            level,
            camera.screen_to_world(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            ),
        )
    }

    pub fn snap_position(&self, level: &Level, pos: vec2<f32>) -> vec2<f32> {
        let closest_point = itertools::chain![
            level
                .all_surfaces()
                .flat_map(|surface| [surface.p1, surface.p2]),
            level.all_tiles().flat_map(|tile| tile.vertices)
        ]
        .filter(|&p| (pos - p).len() < self.config.snap_distance)
        .min_by_key(|&p| r32((pos - p).len()));
        closest_point.unwrap_or(pos)
    }

    pub fn draw_level_editor(&self, framebuffer: &mut ugli::Framebuffer) {
        if let Some(editor) = &self.editor {
            let camera = geng::Camera2d {
                center: self.camera.center * self.level.layers[editor.selected_layer].parallax,
                ..self.camera
            };
            editor.tool.draw(
                &editor.cursor,
                &self.level,
                editor.selected_layer,
                &camera,
                framebuffer,
            );
            self.geng.draw_2d(
                framebuffer,
                &camera,
                &draw_2d::Quad::new(
                    Aabb2::point(self.snapped_cursor_position(&self.level)).extend_uniform(0.1),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        }
    }

    pub fn handle_event_editor(&mut self, event: &geng::Event) {
        if self.editor.is_none() {
            return;
        }
        let cursor_pos = self.snapped_cursor_position(&self.level);
        let editor = self.editor.as_mut().unwrap();
        editor.cursor = Cursor {
            screen_pos: self.geng.window().mouse_pos().map(|x| x as f32),
            world_pos: self.camera.screen_to_world(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            ),
            snapped_world_pos: cursor_pos,
        };

        editor.tool.handle_event(
            &editor.cursor,
            event,
            &mut self.level,
            editor.selected_layer,
        );

        match event {
            geng::Event::KeyDown { key } => match key {
                geng::Key::Tab => {
                    editor.selected_tool_index =
                        (editor.selected_tool_index + 1) % editor.available_tools.len();
                    editor.tool = editor.available_tools[editor.selected_tool_index].create();
                }
                geng::Key::Q => {
                    if !self.geng.window().is_key_pressed(geng::Key::LCtrl) {
                        if let Some(id) = self.my_guy.take() {
                            if let Some(con) = &mut self.connection {
                                con.send(ClientMessage::Despawn);
                            }
                            self.guys.remove(&id);
                        } else {
                            self.my_guy = Some(self.client_id);
                            self.guys.insert(Guy::new(
                                self.client_id,
                                cursor_pos,
                                false,
                                &self.config,
                            ));
                        }
                    }
                }
                geng::Key::S if self.geng.window().is_key_pressed(geng::Key::LCtrl) => {
                    editor.save_level(&mut self.level);
                }
                _ => {}
            },
            _ => {}
        }
    }
    pub fn editor_ui<'a>(
        &'a mut self,
        cx: &'a geng::ui::Controller,
    ) -> Box<dyn geng::ui::Widget + 'a> {
        let editor = self.editor.as_mut().unwrap();
        use geng::ui::*;
        let tool_selection = {
            let mut tools: Vec<Box<dyn Widget>> = vec![];
            for (index, constructor) in editor.available_tools.iter().enumerate() {
                let button = Button::new(cx, constructor.name());
                if button.was_clicked() {
                    editor.selected_tool_index = index;
                    editor.tool = constructor.create();
                }
                let mut widget: Box<dyn Widget> = Box::new(button.uniform_padding(8.0).center());
                if index == editor.selected_tool_index {
                    widget = Box::new(widget.background_color(Rgba::new(0.5, 0.5, 1.0, 0.5)));
                }
                tools.push(widget);
            }
            column(tools)
        };
        let tool_config = editor.tool.ui(cx);
        (
            tool_selection.align(vec2(0.0, 1.0)),
            tool_config.align(vec2(0.0, 1.0)),
        )
            .row()
            .uniform_padding(16.0)
            .background_color(Rgba::new(0.0, 0.0, 0.0, 0.8))
            .align(vec2(0.0, 1.0))
            .boxed()
    }
}
