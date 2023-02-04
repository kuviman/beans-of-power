use std::cmp::max_by_key;

use super::*;

pub struct EditToolConfig {}

impl EditorToolConfig for EditToolConfig {
    fn default(assets: &Assets) -> Self {
        Self {}
    }
}

enum State {
    Idle,
    DragSelection { start: vec2<f32> },
    Grab { start: vec2<f32> },
    Scale { start: vec2<f32> },
    Rotate { start: vec2<f32> },
}

pub struct EditTool {
    geng: Geng,
    assets: Rc<Assets>,
    config: EditToolConfig,
    selected_surfaces: HashSet<usize>,
    selected_tiles: HashSet<usize>,
    state: State,
}

impl EditTool {
    fn find_selection_center(&self, level: &Level) -> vec2<f32> {
        let mut sum = vec2::ZERO;
        let mut count = 0;
        for &index in &self.selected_surfaces {
            sum += level.surfaces[index].p1;
            sum += level.surfaces[index].p2;
            count += 2;
        }
        for &index in &self.selected_tiles {
            for &p in &level.tiles[index].vertices {
                sum += p;
                count += 1;
            }
        }
        if count == 0 {
            return vec2::ZERO;
        }
        sum / count as f32
    }
    fn transform(&self, level: &Level, cursor: &Cursor) -> mat3<f32> {
        let center = self.find_selection_center(level);
        match self.state {
            State::Idle => mat3::identity(),
            State::DragSelection { .. } => mat3::identity(),
            State::Grab { start } => mat3::translate(cursor.world_pos - start),
            State::Scale { start } => {
                mat3::translate(center)
                    * mat3::scale_uniform(
                        (cursor.world_pos - center).len() / (start - center).len(),
                    )
                    * mat3::translate(-center)
            }
            State::Rotate { start } => {
                mat3::translate(center)
                    * mat3::rotate((cursor.world_pos - center).arg() - (start - center).arg())
                    * mat3::translate(-center)
            }
        }
    }
    fn clear_selection(&mut self) {
        self.selected_surfaces.clear();
        self.selected_tiles.clear();
    }
}

impl EditorTool for EditTool {
    type Config = EditToolConfig;
    fn new(geng: &Geng, assets: &Rc<Assets>, config: EditToolConfig) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            config,
            selected_surfaces: default(),
            selected_tiles: default(),
            state: State::Idle,
        }
    }
    fn draw(
        &self,
        cursor: &Cursor,
        level: &Level,
        camera: &geng::Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let transform = self.transform(level, cursor);
        let transform = |p: vec2<f32>| -> vec2<f32> { (transform * p.extend(1.0)).into_2d() };
        for (index, surface) in level.surfaces.iter().enumerate() {
            if self.selected_surfaces.contains(&index) {
                self.geng.draw_2d(
                    framebuffer,
                    camera,
                    &draw_2d::Segment::new(
                        Segment(transform(surface.p1), transform(surface.p2)),
                        0.2,
                        Rgba::new(1.0, 1.0, 1.0, 0.5),
                    ),
                );
            }
        }
        for (index, tile) in level.tiles.iter().enumerate() {
            if self.selected_tiles.contains(&index) {
                self.geng.draw_2d(
                    framebuffer,
                    camera,
                    &draw_2d::Polygon::new(
                        tile.vertices.iter().copied().map(transform).collect(),
                        Rgba::new(1.0, 1.0, 1.0, 0.5),
                    ),
                );
            }
        }
        if let State::DragSelection { start } = self.state {
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Quad::new(
                    Aabb2::from_corners(start, cursor.world_pos),
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        }
    }
    fn handle_event(&mut self, cursor: &Cursor, event: &geng::Event, level: &mut Level) {
        match event {
            geng::Event::MouseDown { button, .. } => match self.state {
                State::Idle => {
                    if *button == geng::MouseButton::Left {
                        self.state = State::DragSelection {
                            start: cursor.world_pos,
                        };
                    }
                }
                State::DragSelection { start } => {}
                State::Grab { .. } | State::Scale { .. } | State::Rotate { .. } => {
                    if *button == geng::MouseButton::Left {
                        let transform = self.transform(level, cursor);
                        let transform = |p: &mut vec2<f32>| {
                            *p = (transform * p.extend(1.0)).into_2d();
                        };
                        let level = level.modify();
                        for &index in &self.selected_surfaces {
                            transform(&mut level.surfaces[index].p1);
                            transform(&mut level.surfaces[index].p2);
                        }
                        for &index in &self.selected_tiles {
                            for p in &mut level.tiles[index].vertices {
                                transform(p);
                            }
                        }
                    }
                    self.state = State::Idle;
                }
            },
            geng::Event::MouseUp {
                button: geng::MouseButton::Left,
                ..
            } => {
                if let State::DragSelection { start } = self.state {
                    self.state = State::Idle;
                    if !self.geng.window().is_key_pressed(geng::Key::LShift) {
                        self.clear_selection();
                    }
                    let aabb = Aabb2::from_corners(start, cursor.world_pos);

                    struct Collision {
                        normal: vec2<f32>,
                        penetration: f32,
                    }

                    fn collide_convex_polygons(a: &[vec2<f32>], b: &[vec2<f32>]) -> Collision {
                        fn f(a: &[vec2<f32>], b: &[vec2<f32>]) -> Collision {
                            (0..a.len())
                                .map(|i| {
                                    let side = [a[i], a[(i + 1) % a.len()]];
                                    let n = (side[0] - side[1]).normalize_or_zero().rotate_90();
                                    let penetration = b
                                        .iter()
                                        .map(|&p| vec2::dot(side[0] - p, n))
                                        .max_by_key(|&x| r32(x))
                                        .unwrap();
                                    Collision {
                                        normal: n,
                                        penetration,
                                    }
                                })
                                .min_by_key(|collision| r32(collision.penetration))
                                .unwrap()
                        }
                        let from_a = f(a, b);
                        let mut from_b = f(b, a);
                        from_b.normal = -from_b.normal;
                        std::cmp::min_by_key(from_a, from_b, |collision| r32(collision.penetration))
                    }

                    let selection_polygon = aabb.corners();
                    for (index, surface) in level.surfaces.iter().enumerate() {
                        if collide_convex_polygons(&selection_polygon, &[surface.p1, surface.p2])
                            .penetration
                            > 0.0
                        {
                            self.selected_surfaces.insert(index);
                        }
                    }
                    for (index, tile) in level.tiles.iter().enumerate() {
                        if collide_convex_polygons(&selection_polygon, &tile.vertices).penetration
                            > 0.0
                        {
                            self.selected_tiles.insert(index);
                        }
                    }
                }
            }
            geng::Event::KeyDown { key: geng::Key::G } => {
                self.state = State::Grab {
                    start: cursor.world_pos,
                };
            }
            geng::Event::KeyDown { key: geng::Key::S } => {
                self.state = State::Scale {
                    start: cursor.world_pos,
                };
            }
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.state = State::Rotate {
                    start: cursor.world_pos,
                };
            }
            _ => {}
        }
    }

    const NAME: &'static str = "Edit";

    fn ui<'a>(&'a mut self, _cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        geng::ui::Void.fixed_size(vec2::ZERO).boxed()
    }
}
