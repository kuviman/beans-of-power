use super::*;

pub struct ObjectToolConfig {
    snap_distance: f32,
    selected_type: String,
}

impl EditorToolConfig for ObjectToolConfig {
    fn default(assets: &Assets) -> Self {
        Self {
            snap_distance: assets.config.snap_distance,
            selected_type: assets.objects.keys().min().unwrap().clone(),
        }
    }
}

pub struct ObjectTool {
    geng: Geng,
    assets: Rc<Assets>,
    config: ObjectToolConfig,
}
impl ObjectTool {
    fn find_hovered_object(&self, cursor: &Cursor, level: &Level) -> Option<usize> {
        level
            .objects
            .iter()
            .enumerate()
            .filter(|(_index, object)| {
                (object.pos - cursor.world_pos).len() < self.config.snap_distance
            })
            .min_by_key(|(_index, object)| r32((object.pos - cursor.world_pos).len()))
            .map(|(index, _object)| index)
    }
}

impl EditorTool for ObjectTool {
    type Config = ObjectToolConfig;
    fn new(geng: &Geng, assets: &Rc<Assets>, config: ObjectToolConfig) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            config,
        }
    }
    fn draw(
        &self,
        cursor: &Cursor,
        level: &Level,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if let Some(index) = self.find_hovered_object(cursor, level) {
            let object = &level.objects[index];
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Quad::new(
                    Aabb2::point(object.pos).extend_uniform(0.5),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        }
    }
    fn handle_event(&mut self, cursor: &Cursor, event: &geng::Event, level: &mut Level) {
        match event {
            geng::Event::MouseDown {
                button: geng::MouseButton::Left,
                ..
            } => level.modify().objects.push(Object {
                type_name: self.config.selected_type.clone(),
                pos: cursor.world_pos,
            }),
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some(index) = self.find_hovered_object(cursor, level) {
                    level.modify().objects.remove(index);
                }
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Object";

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;

        let mut options: Vec<&String> = self.assets.objects.keys().collect();
        options.sort();
        let options = column(
            options
                .into_iter()
                .map(|name| {
                    let button = Button::new(cx, name);
                    if button.was_clicked() {
                        self.config.selected_type = name.clone();
                    }
                    let mut widget: Box<dyn Widget> =
                        Box::new(button.uniform_padding(8.0).align(vec2(0.0, 0.0)));
                    if *name == self.config.selected_type {
                        widget = Box::new(widget.background_color(Rgba::new(0.5, 0.5, 1.0, 0.5)))
                    }
                    widget
                })
                .collect(),
        );
        options.boxed()
    }
}
