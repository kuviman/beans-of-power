use super::*;

pub struct ProgressToolConfig {
    snap_distance: f32,
}

impl EditorToolConfig for ProgressToolConfig {
    fn default(assets: &AssetsHandle) -> Self {
        Self {
            snap_distance: assets.get().config.snap_distance,
        }
    }
}

pub struct ProgressTool {
    geng: Geng,
    assets: AssetsHandle,
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
    fn new(geng: &Geng, assets: &AssetsHandle, config: ProgressToolConfig) -> Self {
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
            self.assets.get().font.draw(
                framebuffer,
                camera,
                &(i + 1).to_string(),
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(p) * mat3::scale_uniform(0.1),
                Rgba::new(0.0, 0.0, 0.0, 0.5),
            );
        }
        for path in &level.expected_path {
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Chain::new(
                    CardinalSpline::new(path.clone(), 0.5).chain(5),
                    level.max_progress_distance * 2.0,
                    Rgba::new(0.0, 0.0, 0.0, 0.25),
                    5,
                ),
            );
        }
        if let Some((i, j)) = self.find_hovered_point(cursor, level) {
            let point = level.expected_path[i][j];
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(
                    Aabb2::point(point).extend_uniform(0.2),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        }
        if let Some(progress) = level.progress_at(cursor.world_pos) {
            self.geng.draw2d().draw2d(
                framebuffer,
                &geng::Camera2d {
                    center: vec2::ZERO,
                    rotation: 0.0,
                    fov: 15.0,
                },
                &draw2d::Text::unit(
                    &**self.geng.default_font(),
                    format!("{}%", (progress * 100.0) as i32),
                    Rgba::BLACK,
                )
                .scale_uniform(0.2)
                .translate(vec2(0.0, -6.0)),
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
