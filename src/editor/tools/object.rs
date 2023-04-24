use super::*;

pub struct ObjectToolConfig {
    snap_distance: f32,
    selected_type: String,
}

impl EditorToolConfig for ObjectToolConfig {
    fn default(assets: &AssetsHandle) -> Self {
        Self {
            snap_distance: assets.get().config.snap_distance,
            selected_type: assets.get().objects.keys().min().unwrap().to_owned(),
        }
    }
}

pub struct ObjectTool {
    geng: Geng,
    assets: AssetsHandle,
    config: ObjectToolConfig,
}
impl ObjectTool {
    fn find_hovered_object(
        &self,
        cursor: &Cursor,
        level: &Level,
        selected_layer: usize,
    ) -> Option<usize> {
        level.layers[selected_layer]
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
    fn new(geng: &Geng, assets: &AssetsHandle, config: ObjectToolConfig) -> Self {
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
        selected_layer: usize,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if let Some(index) = self.find_hovered_object(cursor, level, selected_layer) {
            let object = &level.layers[selected_layer].objects[index];
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(
                    Aabb2::point(object.pos).extend_uniform(0.5),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        }
    }
    fn handle_event(
        &mut self,
        cursor: &Cursor,
        event: &geng::Event,
        level: &mut Level,
        selected_layer: usize,
    ) {
        match event {
            geng::Event::MouseDown {
                button: geng::MouseButton::Left,
                ..
            } => level.modify().layers[selected_layer].objects.push(Object {
                type_name: self.config.selected_type.clone(),
                pos: cursor.world_pos,
            }),
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some(index) = self.find_hovered_object(cursor, level, selected_layer) {
                    level.modify().layers[selected_layer].objects.remove(index);
                }
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Object";

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;

        let assets = self.assets.get();
        let mut options: Vec<&str> = assets.objects.keys().collect();
        options.sort();
        let options = column(
            options
                .into_iter()
                .map(|name| {
                    let button = Button::new(cx, name);
                    if button.was_clicked() {
                        self.config.selected_type = name.to_owned();
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
