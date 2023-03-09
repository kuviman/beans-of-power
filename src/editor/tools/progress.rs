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
    fn find_hovered_point(&self, cursor: &Cursor, level: &Level) -> Option<(usize, usize)> {
        level
            .expected_path
            .iter()
            .enumerate()
            .flat_map(|(i, path)| path.iter().enumerate().map(move |(j, p)| ((i, j), p)))
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
        selected_layer: usize,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        for (i, &p) in level.expected_path.iter().flatten().enumerate() {
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
        for path in &level.expected_path {
            for seg in path.windows(2) {
                self.geng.draw_2d(
                    framebuffer,
                    camera,
                    &draw_2d::Segment::new(
                        Segment(seg[0], seg[1]),
                        0.1,
                        Rgba::new(0.0, 0.0, 0.0, 0.25),
                    ),
                );
            }
        }
        if let Some((i, j)) = self.find_hovered_point(cursor, level) {
            let point = level.expected_path[i][j];
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Quad::new(
                    Aabb2::point(point).extend_uniform(0.2),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        }
        self.geng.draw_2d(
            framebuffer,
            &geng::Camera2d {
                center: vec2::ZERO,
                rotation: 0.0,
                fov: 15.0,
            },
            &draw_2d::Text::unit(
                &**self.geng.default_font(),
                format!("{}%", (level.progress_at(cursor.world_pos) * 100.0) as i32),
                Rgba::BLACK,
            )
            .scale_uniform(0.2)
            .translate(vec2(0.0, -6.0)),
        )
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
                let level = level.modify();
                if level.expected_path.is_empty()
                    || self.geng.window().is_key_pressed(geng::Key::LShift)
                {
                    level.expected_path.push(vec![]);
                }
                level
                    .expected_path
                    .last_mut()
                    .unwrap()
                    .push(cursor.world_pos);
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some((i, j)) = self.find_hovered_point(cursor, level) {
                    let level = level.modify();
                    level.expected_path[i].remove(j);
                    if level.expected_path[i].is_empty() {
                        level.expected_path.remove(i);
                    }
                }
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Progress";

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        Box::new(geng::ui::column![
            "left click changes spawn",
            "right click changes finish",
        ])
    }
}
