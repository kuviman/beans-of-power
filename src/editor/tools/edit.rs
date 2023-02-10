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
    Copy { start: vec2<f32> },
    Grab { start: vec2<f32> },
    Scale { start: vec2<f32> },
    Rotate { start: vec2<f32> },
}

pub struct EditTool {
    geng: Geng,
    assets: Rc<Assets>,
    config: EditToolConfig,
    // selected_surfaces: HashSet<usize>,
    // selected_tiles: HashSet<usize>,
    selected_vertices: Vec<vec2<f32>>,
    state: State,
}

impl EditTool {
    fn find_selection_center(&self, level: &Level) -> vec2<f32> {
        let mut sum = vec2::ZERO;
        let mut count = 0;
        for &v in &self.selected_vertices {
            sum += v;
            count += 1;
        }
        // for &index in &self.selected_surfaces {
        //     sum += level.surfaces[index].p1;
        //     sum += level.surfaces[index].p2;
        //     count += 2;
        // }
        // for &index in &self.selected_tiles {
        //     for &p in &level.tiles[index].vertices {
        //         sum += p;
        //         count += 1;
        //     }
        // }
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
            State::Grab { start } | State::Copy { start } => {
                mat3::translate(cursor.snapped_world_pos - start)
            }
            State::Scale { start } => {
                mat3::translate(center)
                    * mat3::scale_uniform(
                        (cursor.snapped_world_pos - center).len() / (start - center).len(),
                    )
                    * mat3::translate(-center)
            }
            State::Rotate { start } => {
                mat3::translate(center)
                    * mat3::rotate(
                        (cursor.snapped_world_pos - center).arg() - (start - center).arg(),
                    )
                    * mat3::translate(-center)
            }
        }
    }
    fn is_selected(&self, p: vec2<f32>) -> bool {
        self.selected_vertices.contains(&p)
    }
    fn clear_selection(&mut self) {
        // self.selected_surfaces.clear();
        // self.selected_tiles.clear();
        self.selected_vertices.clear();
    }
}

impl EditorTool for EditTool {
    type Config = EditToolConfig;
    fn new(geng: &Geng, assets: &Rc<Assets>, config: EditToolConfig) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            config,
            // selected_surfaces: default(),
            // selected_tiles: default(),
            selected_vertices: default(),
            state: State::Idle,
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
        let transform = self.transform(level, cursor);
        let transform = |p: vec2<f32>| -> vec2<f32> {
            if self.is_selected(p) {
                (transform * p.extend(1.0)).into_2d()
            } else {
                p
            }
        };
        for (index, surface) in level.layers[selected_layer].surfaces.iter().enumerate() {
            if [surface.p1, surface.p2]
                .into_iter()
                .any(|p| self.is_selected(p))
            {
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
        for (index, tile) in level.layers[selected_layer].tiles.iter().enumerate() {
            if tile.vertices.into_iter().any(|p| self.is_selected(p)) {
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
        for &p in &self.selected_vertices {
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Quad::new(
                    Aabb2::point(transform(p)).extend_uniform(0.2),
                    Rgba::new(1.0, 1.0, 1.0, 0.5),
                ),
            );
        }
        if let State::DragSelection { start } = self.state {
            self.geng.draw_2d(
                framebuffer,
                camera,
                &draw_2d::Quad::new(
                    Aabb2::from_corners(start, cursor.snapped_world_pos),
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
            geng::Event::MouseDown { button, .. } => match self.state {
                State::Idle => {
                    if *button == geng::MouseButton::Left {
                        self.state = State::DragSelection {
                            start: cursor.snapped_world_pos,
                        };
                    }
                }
                State::DragSelection { start } => {}
                State::Grab { .. } | State::Scale { .. } | State::Rotate { .. } => {
                    if *button == geng::MouseButton::Left {
                        let matrix = self.transform(level, cursor);
                        let transform = |p: &mut vec2<f32>| {
                            if self.is_selected(*p) {
                                *p = (matrix * p.extend(1.0)).into_2d();
                            }
                        };
                        let level = level.modify();
                        for surface in &mut level.layers[selected_layer].surfaces {
                            transform(&mut surface.p1);
                            transform(&mut surface.p2);
                        }
                        for tile in &mut level.layers[selected_layer].tiles {
                            for p in &mut tile.vertices {
                                transform(p);
                            }
                        }
                        for p in &mut self.selected_vertices {
                            *p = (matrix * p.extend(1.0)).into_2d();
                        }
                    }
                    self.state = State::Idle;
                }
                State::Copy { .. } => {
                    if *button == geng::MouseButton::Left {
                        let matrix = self.transform(level, cursor);
                        let transform = |p: &mut vec2<f32>| {
                            *p = (matrix * p.extend(1.0)).into_2d();
                        };
                        let level = level.modify();
                        let mut new_surfaces = Vec::new();
                        for surface in &mut level.layers[selected_layer].surfaces {
                            if self.is_selected(surface.p1) && self.is_selected(surface.p2) {
                                let mut new_surface = surface.clone();
                                transform(&mut new_surface.p1);
                                transform(&mut new_surface.p2);
                                new_surfaces.push(new_surface);
                            }
                        }
                        level.layers[selected_layer].surfaces.extend(new_surfaces);
                        let mut new_tiles = Vec::new();
                        for tile in &mut level.layers[selected_layer].tiles {
                            if tile.vertices.iter().all(|&p| self.is_selected(p)) {
                                let mut new_tile = tile.clone();
                                for p in &mut new_tile.vertices {
                                    transform(p);
                                }
                                new_tiles.push(new_tile);
                            }
                        }
                        level.layers[selected_layer].tiles.extend(new_tiles);
                        for p in &mut self.selected_vertices {
                            *p = (matrix * p.extend(1.0)).into_2d();
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
                    let aabb = Aabb2::from_corners(start, cursor.snapped_world_pos);

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
                    for p in itertools::chain![
                        level.layers[selected_layer]
                            .surfaces
                            .iter()
                            .flat_map(|surface| [surface.p1, surface.p2]),
                        level.layers[selected_layer]
                            .tiles
                            .iter()
                            .flat_map(|tile| tile.vertices)
                    ] {
                        if aabb.contains(p) && !self.is_selected(p) {
                            self.selected_vertices.push(p);
                        }
                    }
                    // for (index, surface) in level.surfaces.iter().enumerate() {
                    //     if collide_convex_polygons(&selection_polygon, &[surface.p1, surface.p2])
                    //         .penetration
                    //         > 0.0
                    //     {
                    //         self.selected_surfaces.insert(index);
                    //     }
                    // }
                    // for (index, tile) in level.tiles.iter().enumerate() {
                    //     if collide_convex_polygons(&selection_polygon, &tile.vertices).penetration
                    //         > 0.0
                    //     {
                    //         self.selected_tiles.insert(index);
                    //     }
                    // }
                }
            }
            geng::Event::KeyDown { key: geng::Key::G } => {
                self.state = State::Grab {
                    start: cursor.snapped_world_pos,
                };
            }
            geng::Event::KeyDown { key: geng::Key::C }
                if self.geng.window().is_key_pressed(geng::Key::LCtrl) =>
            {
                self.state = State::Copy {
                    start: cursor.snapped_world_pos,
                };
            }
            geng::Event::KeyDown { key: geng::Key::S } => {
                self.state = State::Scale {
                    start: cursor.snapped_world_pos,
                };
            }
            geng::Event::KeyDown { key: geng::Key::R } => {
                self.state = State::Rotate {
                    start: cursor.snapped_world_pos,
                };
            }
            geng::Event::KeyDown {
                key: geng::Key::Delete,
            } => {
                if let State::Idle = self.state {
                    let level = level.modify();
                    level.layers[selected_layer].surfaces.retain(|surface| {
                        ![surface.p1, surface.p2]
                            .iter()
                            .any(|&p| self.is_selected(p))
                    });
                    level.layers[selected_layer]
                        .tiles
                        .retain(|tile| !tile.vertices.iter().any(|&p| self.is_selected(p)));
                }
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
