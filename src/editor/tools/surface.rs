use super::*;

pub struct SurfaceToolConfig {
    snap_distance: f32,
    selected_type: String,
}

impl EditorToolConfig for SurfaceToolConfig {
    fn default(assets: &Assets) -> Self {
        Self {
            snap_distance: assets.config.snap_distance,
            selected_type: assets.surfaces.keys().min().unwrap().clone(),
        }
    }
}

pub struct SurfaceTool {
    geng: Geng,
    assets: Rc<Assets>,
    start_drag: Option<vec2<f32>>,
    wind_drag: Option<(usize, vec2<f32>)>,
    config: SurfaceToolConfig,
}
impl SurfaceTool {
    fn find_hovered_surface(&self, cursor: &Cursor, level: &Level) -> Option<usize> {
        level
            .surfaces
            .iter()
            .enumerate()
            .filter(|(_index, surface)| {
                surface.vector_from(cursor.world_pos).len() < self.config.snap_distance
            })
            .min_by_key(|(_index, surface)| r32(surface.vector_from(cursor.world_pos).len()))
            .map(|(index, _surface)| index)
    }
}

impl EditorTool for SurfaceTool {
    type Config = SurfaceToolConfig;
    fn new(geng: &Geng, assets: &Rc<Assets>, config: SurfaceToolConfig) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            start_drag: None,
            wind_drag: None,
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
        if let Some(p1) = self.start_drag {
            let p2 = cursor.snapped_world_pos;
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Segment::new(Segment(p1, p2), 0.1, Rgba::new(1.0, 1.0, 1.0, 0.5)),
            );
        } else if let Some(index) = self.find_hovered_surface(cursor, level) {
            let surface = &level.surfaces[index];
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Segment::new(
                    Segment(surface.p1, surface.p2),
                    0.2,
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        }
        if let Some((_, start)) = self.wind_drag {
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Segment::new(
                    Segment(start, cursor.world_pos),
                    0.2,
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
            } => {
                self.start_drag = Some(cursor.snapped_world_pos);
            }
            geng::Event::MouseUp {
                button: geng::MouseButton::Left,
                ..
            } => {
                let p2 = cursor.snapped_world_pos;
                if let Some(p1) = self.start_drag.take() {
                    if (p1 - p2).len() > self.config.snap_distance {
                        level.modify().surfaces.push(Surface {
                            p1,
                            p2,
                            flow: 0.0,
                            type_name: self.config.selected_type.clone(),
                        });
                    }
                }
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some(index) = self.find_hovered_surface(cursor, level) {
                    level.modify().surfaces.remove(index);
                }
            }

            geng::Event::KeyDown { key: geng::Key::W } => {
                if self.wind_drag.is_none() {
                    self.wind_drag = self
                        .find_hovered_surface(cursor, level)
                        .map(|index| (index, cursor.world_pos));
                }
            }
            geng::Event::KeyUp { key: geng::Key::W } => {
                if let Some((index, start)) = self.wind_drag.take() {
                    let level = level.modify();
                    let surface = &mut level.surfaces[index];
                    surface.flow = vec2::dot(
                        cursor.world_pos - start,
                        (surface.p2 - surface.p1).normalize_or_zero(),
                    );
                }
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Surface";

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;

        let mut options: Vec<&String> = self.assets.surfaces.keys().collect();
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
