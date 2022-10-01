use geng::prelude::*;

#[derive(geng::Assets, Deserialize, Clone, Debug)]
#[asset(json)]
pub struct Config {
    snap_distance: f32,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub config: Config,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Surface {
    pub p1: Vec2<f32>,
    pub p2: Vec2<f32>,
}

impl Surface {
    pub fn vector_from(&self, point: Vec2<f32>) -> Vec2<f32> {
        if Vec2::dot(point - self.p1, self.p2 - self.p1) < 0.0 {
            return self.p1 - point;
        }
        if Vec2::dot(point - self.p2, self.p1 - self.p2) < 0.0 {
            return self.p2 - point;
        }
        let n = (self.p2 - self.p1).rotate_90();
        // dot(point + n * t - p1, n) = 0
        // dot(point - p1, n) + dot(n, n) * t = 0
        let t = Vec2::dot(self.p1 - point, n) / Vec2::dot(n, n);
        n * t
    }
}

#[derive(geng::Assets, Deserialize, Clone, Debug)]
#[asset(json)]
pub struct Level {
    pub surfaces: Vec<Surface>,
}

impl Level {
    pub fn empty() -> Self {
        Self { surfaces: vec![] }
    }
}

struct Game {
    framebuffer_size: Vec2<f32>,
    geng: Geng,
    config: Config,
    assets: Rc<Assets>,
    camera: geng::Camera2d,
    start_drag: Option<Vec2<f32>>,
    level: Level,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            config: assets.config.clone(),
            assets: assets.clone(),
            camera: geng::Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: 10.0,
            },
            framebuffer_size: vec2(1.0, 1.0),
            start_drag: None,
            level: Level::empty(),
        }
    }

    pub fn snapped_cursor_position(&self) -> Vec2<f32> {
        self.snap_position(self.geng.window().mouse_pos().map(|x| x as f32))
    }

    pub fn snap_position(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let pos = self.camera.screen_to_world(self.framebuffer_size, pos);
        let closest_point = self
            .level
            .surfaces
            .iter()
            .flat_map(|surface| [surface.p1, surface.p2])
            .filter(|&p| (pos - p).len() < self.config.snap_distance)
            .min_by_key(|&p| r32((pos - p).len()));
        closest_point.unwrap_or(pos)
    }

    pub fn draw_level(&self, framebuffer: &mut ugli::Framebuffer) {
        for surface in &self.level.surfaces {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Segment::new(Segment::new(surface.p1, surface.p2), 0.1, Rgba::WHITE),
            );
        }
    }

    pub fn find_hovered_surface(&self) -> Option<usize> {
        let cursor = self.geng.window().mouse_pos().map(|x| x as f32);
        self.level
            .surfaces
            .iter()
            .enumerate()
            .filter(|(_index, surface)| {
                surface.vector_from(cursor).len() < self.config.snap_distance
            })
            .min_by_key(|(_index, surface)| r32(surface.vector_from(cursor).len()))
            .map(|(index, _surface)| index)
    }

    pub fn draw_level_editor(&self, framebuffer: &mut ugli::Framebuffer) {
        if let Some(p1) = self.start_drag {
            let p2 = self.snapped_cursor_position();
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Segment::new(Segment::new(p1, p2), 0.1, Rgba::new(1.0, 1.0, 1.0, 0.5)),
            );
        }
        if let Some(index) = self.find_hovered_surface() {
            let surface = &self.level.surfaces[index];
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Segment::new(
                    Segment::new(surface.p1, surface.p2),
                    0.2,
                    Rgba::new(1.0, 0.0, 0.0, 0.5),
                ),
            );
        }
        self.geng.draw_2d(
            framebuffer,
            &self.camera,
            &draw_2d::Quad::new(
                AABB::point(self.snapped_cursor_position()).extend_uniform(0.1),
                Rgba::new(1.0, 0.0, 0.0, 0.5),
            ),
        );
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);

        self.draw_level(framebuffer);
        self.draw_level_editor(framebuffer);
    }

    fn update(&mut self, delta_time: f64) {
        #![allow(unused_variables)]
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } => {
                self.start_drag = Some(self.snap_position(position.map(|x| x as f32)));
            }
            geng::Event::MouseUp {
                position,
                button: geng::MouseButton::Left,
            } => {
                let p2 = self.snap_position(position.map(|x| x as f32));
                if let Some(p1) = self.start_drag.take() {
                    if (p1 - p2).len() > self.config.snap_distance {
                        self.level.surfaces.push(Surface { p1, p2 });
                    }
                }
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                if let Some(index) = self.find_hovered_surface() {
                    self.level.surfaces.remove(index);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = Geng::new("LD51");
    let state = geng::LoadingScreen::new(
        &geng,
        geng::EmptyLoadingScreen,
        <Assets as geng::LoadAsset>::load(&geng, &static_path()),
        {
            let geng = geng.clone();
            move |assets| {
                let assets = assets.expect("Failed to load assets");
                Game::new(&geng, &Rc::new(assets))
            }
        },
    );
    geng::run(&geng, state);
}
