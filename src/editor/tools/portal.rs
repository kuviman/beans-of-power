use super::*;

pub struct PortalToolConfig {
    snap_distance: f32,
}

impl EditorToolConfig for PortalToolConfig {
    fn default(assets: &AssetsHandle) -> Self {
        Self {
            snap_distance: assets.get().config.snap_distance,
        }
    }
}

pub struct PortalTool {
    geng: Geng,
    assets: AssetsHandle,
    start_drag: Option<usize>,
    config: PortalToolConfig,
}
impl PortalTool {
    fn find_hovered_portal(&self, cursor: &Cursor, level: &Level) -> Option<usize> {
        level
            .portals
            .iter()
            .enumerate()
            .filter(|(_index, portal)| {
                (portal.pos - cursor.world_pos).len() < self.config.snap_distance
            })
            .min_by_key(|(_index, portal)| r32((portal.pos - cursor.world_pos).len()))
            .map(|(index, _portal)| index)
    }
}

impl EditorTool for PortalTool {
    type Config = PortalToolConfig;
    fn new(geng: &Geng, assets: &AssetsHandle, config: PortalToolConfig) -> Self {
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
                    Segment(level.portals[start].pos, cursor.world_pos),
                    0.2,
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        } else if let Some(index) = self.find_hovered_portal(cursor, level) {
            let portal = &level.portals[index];
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Quad::new(
                    Aabb2::point(portal.pos).extend_uniform(0.5),
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
            } => {
                if let Some(index) = self.find_hovered_portal(cursor, level) {
                    self.start_drag = Some(index);
                } else {
                    self.start_drag = Some(level.portals.len());
                    level.modify().portals.push(Portal {
                        pos: cursor.world_pos,
                        dest: None,
                        color: random_hue(),
                    });
                }
            }
            geng::Event::MouseUp {
                button: geng::MouseButton::Left,
                ..
            } => {
                if let Some(start) = self.start_drag.take() {
                    if let Some(index) = self.find_hovered_portal(cursor, level) {
                        if index != start {
                            level.modify().portals[start].dest = Some(index);
                        }
                    }
                }
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some(index) = self.find_hovered_portal(cursor, level) {
                    let level = level.modify();
                    level.portals.remove(index);
                    for portal in &mut level.portals {
                        if let Some(dest) = &mut portal.dest {
                            match (*dest).cmp(&index) {
                                std::cmp::Ordering::Less => {}
                                std::cmp::Ordering::Equal => portal.dest = None,
                                std::cmp::Ordering::Greater => *dest -= 1,
                            }
                        }
                    }
                }
            }
            geng::Event::KeyDown { key: geng::Key::C } => {
                if let Some(index) = self.find_hovered_portal(cursor, level) {
                    level.modify().portals[index].color = random_hue();
                }
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Portal";

    fn ui<'a>(&'a mut self, _cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        geng::ui::Void.fixed_size(vec2::ZERO).boxed()
    }
}
