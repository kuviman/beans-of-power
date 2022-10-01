use geng::prelude::*;

use noise::NoiseFn;

pub const EPS: f32 = 1e-9;

pub const CONTROLS_LEFT: [geng::Key; 2] = [geng::Key::A, geng::Key::Left];
pub const CONTROLS_RIGHT: [geng::Key; 2] = [geng::Key::D, geng::Key::Right];
pub const CONTROLS_FORCE_FART: [geng::Key; 3] = [geng::Key::W, geng::Key::Up, geng::Key::Space];

#[derive(geng::Assets, Deserialize, Clone, Debug)]
#[asset(json)]
pub struct Config {
    snap_distance: f32,
    guy_radius: f32,
    angular_acceleration: f32,
    gravity: f32,
    max_angular_speed: f32,
    fart_strength: f32,
    auto_fart_interval: f32,
    force_fart_interval: f32,
}

#[derive(Deref)]
pub struct Texture(#[deref] ugli::Texture);

impl std::borrow::Borrow<ugli::Texture> for Texture {
    fn borrow(&self) -> &ugli::Texture {
        &self.0
    }
}
impl std::borrow::Borrow<ugli::Texture> for &'_ Texture {
    fn borrow(&self) -> &ugli::Texture {
        &self.0
    }
}

impl geng::LoadAsset for Texture {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let texture = <ugli::Texture as geng::LoadAsset>::load(geng, path);
        async move {
            let mut texture = texture.await?;
            texture.set_filter(ugli::Filter::Nearest);
            Ok(Texture(texture))
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("png");
}

#[derive(Deserialize)]
pub struct SurfaceParams {
    pub bounciness: f32,
    pub friction: f32,
}

pub struct SurfaceAssets {
    pub name: String,
    pub params: SurfaceParams,
    pub front_texture: Texture,
    pub back_texture: Texture,
}

#[derive(geng::Assets)]
pub struct GuyAssets {
    pub body: Texture,
    pub cheeks: Texture,
    pub eyes: Texture,
}

fn load_surface_assets(
    geng: &Geng,
    path: &std::path::Path,
) -> geng::AssetFuture<Vec<SurfaceAssets>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("config.json")).await?;
        let config: std::collections::BTreeMap<String, SurfaceParams> =
            serde_json::from_str(&json).unwrap();
        future::join_all(config.into_iter().map(|(name, params)| {
            let geng = geng.clone();
            let path = path.clone();
            async move {
                let mut back_texture = <Texture as geng::LoadAsset>::load(
                    &geng,
                    &path.join(format!("{}_back.png", name)),
                )
                .await?;
                back_texture.0.set_wrap_mode(ugli::WrapMode::Repeat);
                let mut front_texture = <Texture as geng::LoadAsset>::load(
                    &geng,
                    &path.join(format!("{}_front.png", name)),
                )
                .await?;
                front_texture.0.set_wrap_mode(ugli::WrapMode::Repeat);
                Ok(SurfaceAssets {
                    name,
                    params,
                    front_texture,
                    back_texture,
                })
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<SurfaceAssets>, anyhow::Error>>()
    }
    .boxed_local()
}

pub struct BackgroundAssets {
    pub name: String,
    pub texture: Texture,
}

fn load_background_assets(
    geng: &Geng,
    path: &std::path::Path,
) -> geng::AssetFuture<Vec<BackgroundAssets>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("_list.json")).await?;
        let list: Vec<String> = serde_json::from_str(&json).unwrap();
        future::join_all(list.into_iter().map(|name| {
            let geng = geng.clone();
            let path = path.clone();
            async move {
                let mut texture =
                    <Texture as geng::LoadAsset>::load(&geng, &path.join(format!("{}.png", name)))
                        .await?;
                texture.0.set_wrap_mode(ugli::WrapMode::Repeat);
                Ok(BackgroundAssets { name, texture })
            }
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<BackgroundAssets>, anyhow::Error>>()
    }
    .boxed_local()
}

#[derive(geng::Assets)]
pub struct Assets {
    pub config: Config,
    pub guy: GuyAssets,
    #[asset(load_with = "load_surface_assets(&geng, &base_path.join(\"surfaces\"))")]
    pub surfaces: Vec<SurfaceAssets>,
    #[asset(load_with = "load_background_assets(&geng, &base_path.join(\"background\"))")]
    pub background: Vec<BackgroundAssets>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Surface {
    pub p1: Vec2<f32>,
    pub p2: Vec2<f32>,
    pub type_index: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BackgroundTile {
    pub vertices: [Vec2<f32>; 3],
    pub type_index: usize,
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
    pub background_tiles: Vec<BackgroundTile>,
}

impl Level {
    pub fn empty() -> Self {
        Self {
            surfaces: vec![],
            background_tiles: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Input {
    pub roll_direction: f32, // -1 to +1
    pub force_fart: bool,
}

pub type Id = i32;

#[derive(Serialize, Deserialize, Clone, Debug, HasId)]
pub struct Guy {
    pub id: Id,
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub rot: f32,
    pub w: f32,
    pub input: Input,
    pub auto_fart_timer: f32,
    pub force_fart_timer: f32,
}

impl Guy {
    pub fn new(id: Id, pos: Vec2<f32>) -> Self {
        Self {
            id,
            pos,
            vel: Vec2::ZERO,
            rot: 0.0,
            w: 0.0,
            input: Input::default(),
            auto_fart_timer: 0.0,
            force_fart_timer: 0.0,
        }
    }
}

struct EditorState {
    start_drag: Option<Vec2<f32>>,
    face_points: Vec<Vec2<f32>>,
}

struct Game {
    framebuffer_size: Vec2<f32>,
    geng: Geng,
    config: Config,
    assets: Rc<Assets>,
    camera: geng::Camera2d,
    level: Level,
    editor: EditorState,
    guys: Collection<Guy>,
    my_guy: Option<Id>,
    real_time: f32,
    noise: noise::OpenSimplex,
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
                fov: 5.0,
            },
            framebuffer_size: vec2(1.0, 1.0),
            editor: EditorState {
                start_drag: None,
                face_points: vec![],
            },
            level: Level::empty(),
            guys: Collection::new(),
            my_guy: None,
            real_time: 0.0,
            noise: noise::OpenSimplex::new(),
        }
    }

    pub fn snapped_cursor_position(&self) -> Vec2<f32> {
        self.snap_position(self.camera.screen_to_world(
            self.framebuffer_size,
            self.geng.window().mouse_pos().map(|x| x as f32),
        ))
    }

    pub fn snap_position(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let closest_point = self
            .level
            .surfaces
            .iter()
            .flat_map(|surface| [surface.p1, surface.p2])
            .filter(|&p| (pos - p).len() < self.config.snap_distance)
            .min_by_key(|&p| r32((pos - p).len()));
        closest_point.unwrap_or(pos)
    }

    pub fn draw_guys(&self, framebuffer: &mut ugli::Framebuffer) {
        for guy in &self.guys {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit(&self.assets.guy.body)
                    .scale_uniform(self.config.guy_radius)
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
            );
            let autofart_progress = guy.auto_fart_timer / self.config.auto_fart_interval;
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit(&self.assets.guy.eyes)
                    .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * autofart_progress)
                    .scale_uniform(self.config.guy_radius * (0.8 + 0.6 * autofart_progress))
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
            );
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(
                    &self.assets.guy.cheeks,
                    Rgba::new(1.0, 1.0, 1.0, (0.5 + 1.0 * autofart_progress).min(1.0)),
                )
                .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * autofart_progress)
                .scale_uniform(self.config.guy_radius * (0.8 + 0.7 * autofart_progress))
                .transform(Mat3::rotate(guy.rot))
                .translate(guy.pos),
            );
        }
    }

    pub fn draw_level_impl(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: impl Fn(&SurfaceAssets) -> &Texture,
    ) {
        for surface in &self.level.surfaces {
            let assets = &self.assets.surfaces[surface.type_index];
            let texture = texture(assets);
            let normal = (surface.p2 - surface.p1).normalize().rotate_90();
            let len = (surface.p2 - surface.p1).len();
            let height = texture.size().y as f32 / texture.size().x as f32;
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedPolygon::new(
                    vec![
                        draw_2d::TexturedVertex {
                            a_pos: surface.p1,
                            a_color: Rgba::WHITE,
                            a_vt: vec2(0.0, 0.0),
                        },
                        draw_2d::TexturedVertex {
                            a_pos: surface.p2,
                            a_color: Rgba::WHITE,
                            a_vt: vec2(len, 0.0),
                        },
                        draw_2d::TexturedVertex {
                            a_pos: surface.p2 + normal * height,
                            a_color: Rgba::WHITE,
                            a_vt: vec2(len, 1.0),
                        },
                        draw_2d::TexturedVertex {
                            a_pos: surface.p1 + normal * height,
                            a_color: Rgba::WHITE,
                            a_vt: vec2(0.0, 1.0),
                        },
                    ],
                    texture,
                ),
            );
        }
    }

    pub fn draw_level_back(&self, framebuffer: &mut ugli::Framebuffer) {
        for tile in &self.level.background_tiles {
            let assets = &self.assets.background[tile.type_index];
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedPolygon::new(
                    tile.vertices
                        .into_iter()
                        .map(|v| draw_2d::TexturedVertex {
                            a_pos: v,
                            a_color: Rgba::WHITE,
                            a_vt: v,
                        })
                        .collect(),
                    &assets.texture,
                ),
            );
        }
        self.draw_level_impl(framebuffer, |assets| &assets.back_texture);
    }

    pub fn draw_level_front(&self, framebuffer: &mut ugli::Framebuffer) {
        self.draw_level_impl(framebuffer, |assets| &assets.front_texture);
    }

    pub fn find_hovered_surface(&self) -> Option<usize> {
        let cursor = self.camera.screen_to_world(
            self.framebuffer_size,
            self.geng.window().mouse_pos().map(|x| x as f32),
        );
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
        if let Some(p1) = self.editor.start_drag {
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
        for &p in &self.editor.face_points {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(p).extend_uniform(0.1),
                    Rgba::new(0.0, 1.0, 0.0, 0.5),
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

    pub fn update_my_guy_input(&mut self) {
        let my_guy = match self.my_guy.map(|id| self.guys.get_mut(&id).unwrap()) {
            Some(guy) => guy,
            None => return,
        };
        let new_input = Input {
            roll_direction: {
                let mut direction = 0.0;
                if CONTROLS_LEFT
                    .iter()
                    .any(|&key| self.geng.window().is_key_pressed(key))
                {
                    direction += 1.0;
                }
                if CONTROLS_RIGHT
                    .iter()
                    .any(|&key| self.geng.window().is_key_pressed(key))
                {
                    direction -= 1.0;
                }
                direction
            },
            force_fart: CONTROLS_FORCE_FART
                .iter()
                .any(|&key| self.geng.window().is_key_pressed(key)),
        };
        my_guy.input = new_input;
    }

    pub fn update_guys(&mut self, delta_time: f32) {
        for guy in &mut self.guys {
            guy.w += guy.input.roll_direction.clamp(-1.0, 1.0)
                * self.config.angular_acceleration
                * delta_time;
            // guy.w = guy.w.clamp_abs(self.config.max_angular_speed);
            guy.vel.y -= self.config.gravity * delta_time;

            let mut farts = 0;
            guy.auto_fart_timer += delta_time;
            if guy.auto_fart_timer >= self.config.auto_fart_interval {
                guy.auto_fart_timer = 0.0;
                farts += 1;
            }
            guy.force_fart_timer += delta_time;
            if guy.force_fart_timer >= self.config.force_fart_interval && guy.input.force_fart {
                farts += 1;
                guy.force_fart_timer = 0.0;
            }
            for _ in 0..farts {
                guy.vel += vec2(0.0, self.config.fart_strength).rotate(guy.rot);
            }

            guy.pos += guy.vel * delta_time;
            guy.rot += guy.w * delta_time;

            struct Collision {
                penetration: f32,
                normal: Vec2<f32>,
                surface_type: usize,
            }

            let mut collision_to_resolve = None;
            for surface in &self.level.surfaces {
                let v = surface.vector_from(guy.pos);
                let penetration = self.config.guy_radius - v.len();
                if penetration > EPS && Vec2::dot(v, guy.vel) > 0.0 {
                    let collision = Collision {
                        penetration,
                        normal: -v.normalize_or_zero(),
                        surface_type: surface.type_index,
                    };
                    collision_to_resolve =
                        std::cmp::max_by_key(collision_to_resolve, Some(collision), |collision| {
                            r32(match collision {
                                Some(collision) => collision.penetration,
                                None => -1.0,
                            })
                        });
                }
            }
            if let Some(collision) = collision_to_resolve {
                let surface_params = &self.assets.surfaces[collision.surface_type].params;
                guy.pos += collision.normal * collision.penetration;
                let normal_vel = Vec2::dot(guy.vel, collision.normal);
                let tangent = collision.normal.rotate_90();
                let tangent_vel = Vec2::dot(guy.vel, tangent) - guy.w * self.config.guy_radius;
                guy.vel -= collision.normal * normal_vel * (1.0 + surface_params.bounciness);
                let max_friction_impulse = normal_vel.abs() * surface_params.friction;
                let friction_impulse = -tangent_vel.clamp_abs(max_friction_impulse);
                guy.vel += tangent * friction_impulse;
                guy.w -= friction_impulse / self.config.guy_radius;
            }
        }
    }

    #[track_caller]
    pub fn noise(&self, frequency: f32) -> f32 {
        let caller = std::panic::Location::caller();
        let phase = caller.line() as f64 * 1000.0 + caller.column() as f64;
        self.noise.get([(self.real_time * frequency) as f64, phase]) as f32
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::new(0.8, 0.8, 1.0, 1.0)), None, None);

        self.draw_level_back(framebuffer);
        self.draw_guys(framebuffer);
        self.draw_level_front(framebuffer);
        self.draw_level_editor(framebuffer);
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.update_my_guy_input();
        self.update_guys(delta_time);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.real_time += delta_time;
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } => {
                self.editor.start_drag = Some(
                    self.snap_position(
                        self.camera
                            .screen_to_world(self.framebuffer_size, position.map(|x| x as f32)),
                    ),
                );
            }
            geng::Event::MouseUp {
                position,
                button: geng::MouseButton::Left,
            } => {
                let p2 = self.snap_position(
                    self.camera
                        .screen_to_world(self.framebuffer_size, position.map(|x| x as f32)),
                );
                if let Some(p1) = self.editor.start_drag.take() {
                    if (p1 - p2).len() > self.config.snap_distance {
                        self.level.surfaces.push(Surface {
                            p1,
                            p2,
                            type_index: 0,
                        });
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
            geng::Event::KeyDown { key } => match key {
                geng::Key::F => {
                    self.editor.face_points.push(self.snapped_cursor_position());
                    if self.editor.face_points.len() == 3 {
                        self.level.background_tiles.push(BackgroundTile {
                            vertices: mem::take(&mut self.editor.face_points).try_into().unwrap(),
                            type_index: 0,
                        });
                    }
                }
                geng::Key::C => {
                    self.editor.face_points.clear();
                }
                geng::Key::R => {
                    if let Some(id) = self.my_guy.take() {
                        self.guys.remove(&id);
                    } else {
                        let id = -1;
                        self.my_guy = Some(id);
                        self.guys.insert(Guy::new(
                            id,
                            self.camera.screen_to_world(
                                self.framebuffer_size,
                                self.geng.window().cursor_position().map(|x| x as f32),
                            ),
                        ));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = Geng::new_with(geng::ContextOptions {
        title: "LD51".to_owned(),
        fixed_delta_time: 1.0 / 200.0,
        ..default()
    });
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
