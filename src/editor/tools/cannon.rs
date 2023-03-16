use super::*;

pub struct CannonToolConfig {
    snap_distance: f32,
}

impl EditorToolConfig for CannonToolConfig {
    fn default(assets: &AssetsHandle) -> Self {
        Self {
            snap_distance: assets.get().config.snap_distance,
        }
    }
}

pub struct CannonTool {
    geng: Geng,
    assets: AssetsHandle,
    start_drag: Option<vec2<f32>>,
    config: CannonToolConfig,
}
impl CannonTool {
    fn find_hovered_cannon(&self, cursor: &Cursor, level: &Level) -> Option<usize> {
        level
            .cannons
            .iter()
            .enumerate()
            .filter(|(_index, cannon)| {
                (cannon.pos - cursor.world_pos).len() < self.config.snap_distance
            })
            .min_by_key(|(_index, cannon)| r32((cannon.pos - cursor.world_pos).len()))
            .map(|(index, _cannon)| index)
    }
}

impl EditorTool for CannonTool {
    type Config = CannonToolConfig;
    fn new(geng: &Geng, assets: &AssetsHandle, config: CannonToolConfig) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            config,
            start_drag: None,
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
        if let Some(start) = self.start_drag {
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Segment::new(
                    Segment(start, cursor.world_pos),
                    0.2,
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        } else if let Some(index) = self.find_hovered_cannon(cursor, level) {
            let cannon = &level.cannons[index];
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Quad::new(
                    Aabb2::point(cannon.pos).extend_uniform(0.5),
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
            } => self.start_drag = Some(cursor.world_pos),
            geng::Event::MouseUp {
                button: geng::MouseButton::Left,
                ..
            } => {
                if let Some(start) = self.start_drag.take() {
                    level.modify().cannons.push(Cannon {
                        pos: start,
                        rot: (cursor.world_pos - start).arg(),
                    });
                }
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some(index) = self.find_hovered_cannon(cursor, level) {
                    level.modify().cannons.remove(index);
                }
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Cannon";

    fn ui<'a>(&'a mut self, _cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        geng::ui::Void.fixed_size(vec2::ZERO).boxed()
    }
}
