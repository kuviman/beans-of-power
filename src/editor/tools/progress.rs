use super::*;

pub struct ProgressToolConfig {
    snap_distance: f32,
}

impl EditorToolConfig for ProgressToolConfig {
    fn default(assets: &Assets) -> Self {
        Self {
            snap_distance: assets.config.snap_distance,
        }
    }
}

pub struct ProgressTool {
    geng: Geng,
    assets: Rc<Assets>,
    config: ProgressToolConfig,
}

impl ProgressTool {
    fn find_hovered_point(&self, cursor: &Cursor, level: &Level) -> Option<usize> {
        level
            .expected_path
            .iter()
            .enumerate()
            .filter(|(_index, &pos)| (pos - cursor.world_pos).len() < self.config.snap_distance)
            .min_by_key(|(_index, &pos)| r32((pos - cursor.world_pos).len()))
            .map(|(index, _pos)| index)
    }
}

impl EditorTool for ProgressTool {
    type Config = ProgressToolConfig;
    fn new(geng: &Geng, assets: &Rc<Assets>, config: ProgressToolConfig) -> Self {
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
        for (i, &p) in level.expected_path.iter().enumerate() {
            self.assets.font.draw(
                framebuffer,
                camera,
                &(i + 1).to_string(),
                p,
                geng::TextAlign::CENTER,
                0.1,
                Rgba::new(0.0, 0.0, 0.0, 0.5),
            );
        }
        for seg in level.expected_path.windows(2) {
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Segment::new(
                    Segment::new(seg[0], seg[1]),
                    0.1,
                    Rgba::new(0.0, 0.0, 0.0, 0.25),
                ),
            );
        }
        if let Some(index) = self.find_hovered_point(cursor, level) {
            let point = level.expected_path[index];
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Quad::new(
                    AABB::point(point).extend_uniform(0.2),
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
                level.modify().expected_path.push(cursor.world_pos);
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some(index) = self.find_hovered_point(cursor, level) {
                    level.modify().expected_path.remove(index);
                }
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Progress";

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        Box::new(geng::ui::column![
            "left click changes spawn",
            "right click changes finish",
        ])
    }
}
