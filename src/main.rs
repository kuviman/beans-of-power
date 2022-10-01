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
    max_angular_speed: f32, // TODO: maybe?
    fart_strength: f32,
    auto_fart_interval: f32,
    force_fart_interval: f32,
    fart_color: Rgba<f32>,
    farticle_w: f32,
    farticle_size: f32,
    farticle_count: usize,
    farticle_additional_vel: f32,
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
    pub front: bool,
    pub back: bool,
}

pub struct SurfaceAssets {
    pub name: String,
    pub params: SurfaceParams,
    pub front_texture: Option<Texture>,
    pub back_texture: Option<Texture>,
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
) -> geng::AssetFuture<HashMap<String, SurfaceAssets>> {
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
                let load = |file| {
                    let geng = geng.clone();
                    let path = path.clone();
                    async move {
                        let mut texture =
                            <Texture as geng::LoadAsset>::load(&geng, &path.join(file)).await?;
                        texture.0.set_wrap_mode(ugli::WrapMode::Repeat);
                        Ok::<_, anyhow::Error>(texture)
                    }
                };
                let mut back_texture = if params.back {
                    Some(load(format!("{}_back.png", name)).await?)
                } else {
                    None
                };
                let mut front_texture = if params.front {
                    Some(load(format!("{}_front.png", name)).await?)
                } else {
                    None
                };
                Ok((
                    name.clone(),
                    SurfaceAssets {
                        name,
                        params,
                        front_texture,
                        back_texture,
                    },
                ))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<_, anyhow::Error>>()
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
) -> geng::AssetFuture<HashMap<String, BackgroundAssets>> {
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
                Ok((name.clone(), BackgroundAssets { name, texture }))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<_, anyhow::Error>>()
    }
    .boxed_local()
}

#[derive(geng::Assets)]
pub struct SfxAssets {
    #[asset(range = "1..=3", path = "fart/*.wav")]
    pub fart: Vec<geng::Sound>,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub config: Config,
    pub sfx: SfxAssets,
    pub guy: GuyAssets,
    #[asset(load_with = "load_surface_assets(&geng, &base_path.join(\"surfaces\"))")]
    pub surfaces: HashMap<String, SurfaceAssets>,
    #[asset(load_with = "load_background_assets(&geng, &base_path.join(\"background\"))")]
    pub background: HashMap<String, BackgroundAssets>,
    pub farticle: Texture,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Surface {
    pub p1: Vec2<f32>,
    pub p2: Vec2<f32>,
    pub type_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackgroundTile {
    pub vertices: [Vec2<f32>; 3],
    pub type_name: String,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Level {
    pub spawn_point: Vec2<f32>,
    pub surfaces: Vec<Surface>,
    pub background_tiles: Vec<BackgroundTile>,
}

impl Level {
    pub fn empty() -> Self {
        Self {
            spawn_point: Vec2::ZERO,
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
    next_autosave: f32,
    start_drag: Option<Vec2<f32>>,
    face_points: Vec<Vec2<f32>>,
    selected_surface: String,
    selected_background: String,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            next_autosave: 0.0,
            start_drag: None,
            face_points: vec![],
            selected_surface: "".to_owned(),
            selected_background: "".to_owned(),
        }
    }
}

pub struct Farticle {
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub color: Rgba<f32>,
    pub rot: f32,
    pub w: f32,
    pub t: f32,
}

struct Game {
    framebuffer_size: Vec2<f32>,
    prev_mouse_pos: Vec2<f64>,
    geng: Geng,
    config: Config,
    assets: Rc<Assets>,
    camera: geng::Camera2d,
    level: Level,
    editor: Option<EditorState>,
    guys: Collection<Guy>,
    my_guy: Option<Id>,
    real_time: f32,
    noise: noise::OpenSimplex,
    opt: Opt,
    farticles: Vec<Farticle>,
    volume: f32,
}

impl Drop for Game {
    fn drop(&mut self) {
        self.save_level();
    }
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, level: Level, opt: Opt) -> Self {
        let mut result = Self {
            geng: geng.clone(),
            config: assets.config.clone(),
            assets: assets.clone(),
            camera: geng::Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: 5.0,
            },
            framebuffer_size: vec2(1.0, 1.0),
            editor: if opt.editor {
                Some(EditorState::new())
            } else {
                None
            },
            level,
            guys: Collection::new(),
            my_guy: None,
            real_time: 0.0,
            noise: noise::OpenSimplex::new(),
            prev_mouse_pos: Vec2::ZERO,
            opt: opt.clone(),
            farticles: default(),
            volume: 0.5,
        };
        if !opt.editor {
            let id = -1;
            result.my_guy = Some(id);
            result.guys.insert(Guy::new(id, result.level.spawn_point));
        }
        result
    }

    pub fn snapped_cursor_position(&self) -> Vec2<f32> {
        self.snap_position(self.camera.screen_to_world(
            self.framebuffer_size,
            self.geng.window().mouse_pos().map(|x| x as f32),
        ))
    }

    pub fn snap_position(&self, pos: Vec2<f32>) -> Vec2<f32> {
        let closest_point = itertools::chain![
            self.level
                .surfaces
                .iter()
                .flat_map(|surface| [surface.p1, surface.p2]),
            self.level
                .background_tiles
                .iter()
                .flat_map(|tile| tile.vertices)
        ]
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
        texture: impl Fn(&SurfaceAssets) -> Option<&Texture>,
    ) {
        for surface in &self.level.surfaces {
            let assets = &self.assets.surfaces[&surface.type_name];
            let texture = match texture(assets) {
                Some(texture) => texture,
                None => continue,
            };
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
        self.draw_level_impl(framebuffer, |assets| assets.back_texture.as_ref());
    }

    pub fn draw_level_front(&self, framebuffer: &mut ugli::Framebuffer) {
        for tile in &self.level.background_tiles {
            let assets = &self.assets.background[&tile.type_name];
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
        self.draw_level_impl(framebuffer, |assets| assets.front_texture.as_ref());
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

    pub fn find_hovered_background(&self) -> Option<usize> {
        let p = self.camera.screen_to_world(
            self.framebuffer_size,
            self.geng.window().mouse_pos().map(|x| x as f32),
        );
        'tile_loop: for (index, tile) in self.level.background_tiles.iter().enumerate() {
            for i in 0..3 {
                let p1 = tile.vertices[i];
                let p2 = tile.vertices[(i + 1) % 3];
                if Vec2::skew(p2 - p1, p - p1) < 0.0 {
                    continue 'tile_loop;
                }
            }
            return Some(index);
        }
        None
    }

    pub fn draw_level_editor(&self, framebuffer: &mut ugli::Framebuffer) {
        if let Some(editor) = &self.editor {
            if let Some(p1) = editor.start_drag {
                let p2 = self.snapped_cursor_position();
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Segment::new(
                        Segment::new(p1, p2),
                        0.1,
                        Rgba::new(1.0, 1.0, 1.0, 0.5),
                    ),
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
            if let Some(index) = self.find_hovered_background() {
                let tile = &self.level.background_tiles[index];
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Polygon::new(tile.vertices.into(), Rgba::new(0.0, 0.0, 1.0, 0.5)),
                );
            }
            for &p in &editor.face_points {
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

            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::new(
                    AABB::point(self.level.spawn_point).extend_uniform(0.1),
                    Rgba::new(1.0, 0.8, 0.8, 0.5),
                ),
            );
        }
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
                for _ in 0..self.config.farticle_count {
                    self.farticles.push(Farticle {
                        pos: guy.pos,
                        vel: guy.vel
                            + vec2(
                                global_rng().gen_range(0.0..=self.config.farticle_additional_vel),
                                0.0,
                            )
                            .rotate(global_rng().gen_range(0.0..=2.0 * f32::PI)),
                        rot: global_rng().gen_range(0.0..2.0 * f32::PI),
                        w: global_rng().gen_range(-self.config.farticle_w..=self.config.farticle_w),
                        color: self.config.fart_color,
                        t: 1.0,
                    });
                }
                guy.vel += vec2(0.0, self.config.fart_strength).rotate(guy.rot);
                let mut effect = self
                    .assets
                    .sfx
                    .fart
                    .choose(&mut global_rng())
                    .unwrap()
                    .effect();
                effect.set_volume(
                    (self.volume * (1.0 - (guy.pos - self.camera.center).len() / self.camera.fov))
                        .clamp(0.0, 1.0) as f64,
                );
                effect.play();
            }

            guy.pos += guy.vel * delta_time;
            guy.rot += guy.w * delta_time;

            struct Collision<'a> {
                penetration: f32,
                normal: Vec2<f32>,
                surface_params: &'a SurfaceParams,
            }

            let mut collision_to_resolve = None;
            for surface in &self.level.surfaces {
                let v = surface.vector_from(guy.pos);
                let penetration = self.config.guy_radius - v.len();
                if penetration > EPS && Vec2::dot(v, guy.vel) > 0.0 {
                    let collision = Collision {
                        penetration,
                        normal: -v.normalize_or_zero(),
                        surface_params: &self.assets.surfaces[&surface.type_name].params,
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
                guy.pos += collision.normal * collision.penetration;
                let normal_vel = Vec2::dot(guy.vel, collision.normal);
                let tangent = collision.normal.rotate_90();
                let tangent_vel = Vec2::dot(guy.vel, tangent) - guy.w * self.config.guy_radius;
                guy.vel -=
                    collision.normal * normal_vel * (1.0 + collision.surface_params.bounciness);
                let max_friction_impulse = normal_vel.abs() * collision.surface_params.friction;
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

    pub fn handle_event_editor(&mut self, event: &geng::Event) {
        if self.opt.editor
            && matches!(
                event,
                geng::Event::KeyDown {
                    key: geng::Key::Tab
                }
            )
        {
            if self.editor.is_none() {
                self.editor = Some(EditorState::new());
            } else {
                self.editor = None;
            }
        }
        if self.editor.is_none() {
            return;
        }
        let cursor_pos = self.snapped_cursor_position();
        let editor = self.editor.as_mut().unwrap();

        if !self.assets.surfaces.contains_key(&editor.selected_surface) {
            editor.selected_surface = self.assets.surfaces.keys().next().unwrap().clone();
        }
        if !self
            .assets
            .background
            .contains_key(&editor.selected_background)
        {
            editor.selected_background = self.assets.background.keys().next().unwrap().clone();
        }

        match event {
            geng::Event::MouseDown {
                button: geng::MouseButton::Left,
                ..
            } => {
                if let Some(editor) = &mut self.editor {
                    editor.start_drag = Some(cursor_pos);
                }
            }
            geng::Event::MouseUp {
                button: geng::MouseButton::Left,
                ..
            } => {
                let p2 = cursor_pos;

                if let Some(p1) = editor.start_drag.take() {
                    if (p1 - p2).len() > self.config.snap_distance {
                        self.level.surfaces.push(Surface {
                            p1,
                            p2,
                            type_name: editor.selected_surface.clone(),
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
                    editor.face_points.push(cursor_pos);
                    if editor.face_points.len() == 3 {
                        let mut vertices: [Vec2<f32>; 3] =
                            mem::take(&mut editor.face_points).try_into().unwrap();
                        if Vec2::skew(vertices[1] - vertices[0], vertices[2] - vertices[0]) < 0.0 {
                            vertices.reverse();
                        }
                        self.level.background_tiles.push(BackgroundTile {
                            vertices,
                            type_name: editor.selected_background.clone(),
                        });
                    }
                }
                geng::Key::D => {
                    if let Some(index) = self.find_hovered_background() {
                        self.level.background_tiles.remove(index);
                    }
                }
                geng::Key::C => {
                    editor.face_points.clear();
                }
                geng::Key::R => {
                    if self.geng.window().is_key_pressed(geng::Key::LCtrl) {
                        if self.my_guy.is_none() {
                            let id = -1;
                            self.my_guy = Some(id);
                            self.guys.insert(Guy::new(id, cursor_pos));
                        }
                        self.guys.get_mut(&self.my_guy.unwrap()).unwrap().pos =
                            self.level.spawn_point;
                    } else {
                        if let Some(id) = self.my_guy.take() {
                            self.guys.remove(&id);
                        } else {
                            let id = -1;
                            self.my_guy = Some(id);
                            self.guys.insert(Guy::new(id, cursor_pos));
                        }
                    }
                }
                geng::Key::P => {
                    self.level.spawn_point = self.camera.screen_to_world(
                        self.framebuffer_size,
                        self.geng.window().mouse_pos().map(|x| x as f32),
                    );
                }
                geng::Key::Z => {
                    let mut options: Vec<&String> = self.assets.surfaces.keys().collect();
                    options.sort();
                    let idx = options
                        .iter()
                        .position(|&s| s == &editor.selected_surface)
                        .unwrap_or(0);
                    editor.selected_surface = options[(idx + 1) % options.len()].clone();
                }
                geng::Key::X => {
                    let mut options: Vec<&String> = self.assets.background.keys().collect();
                    options.sort();
                    let idx = options
                        .iter()
                        .position(|&s| s == &editor.selected_background)
                        .unwrap_or(0);
                    editor.selected_background = options[(idx + 1) % options.len()].clone();
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn save_level(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.editor.is_some() {
            serde_json::to_writer_pretty(
                std::fs::File::create(static_path().join("level.json")).unwrap(),
                &self.level,
            )
            .unwrap();
            info!("LVL SAVED");
        }
    }

    pub fn update_farticles(&mut self, delta_time: f32) {
        for farticle in &mut self.farticles {
            farticle.t -= delta_time;
            farticle.pos += farticle.vel * delta_time;
            farticle.rot += farticle.w * delta_time;

            for surface in &self.level.surfaces {
                let v = surface.vector_from(farticle.pos);
                let penetration = self.config.farticle_size / 2.0 - v.len();
                if penetration > EPS && Vec2::dot(v, farticle.vel) > 0.0 {
                    let normal = -v.normalize_or_zero();
                    farticle.pos += normal * penetration;
                    farticle.vel -= normal * Vec2::dot(farticle.vel, normal);
                }
            }
        }
        self.farticles.retain(|farticle| farticle.t > 0.0);
    }

    pub fn draw_farticles(&self, framebuffer: &mut ugli::Framebuffer) {
        for farticle in &self.farticles {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(
                    &self.assets.farticle,
                    Rgba {
                        a: farticle.color.a * farticle.t,
                        ..farticle.color
                    },
                )
                .transform(Mat3::rotate(farticle.rot))
                .scale_uniform(self.config.farticle_size)
                .translate(farticle.pos),
            )
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::new(0.8, 0.8, 1.0, 1.0)), None, None);

        self.draw_level_back(framebuffer);
        self.draw_guys(framebuffer);
        self.draw_level_front(framebuffer);
        self.draw_farticles(framebuffer);
        self.draw_level_editor(framebuffer);
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.update_my_guy_input();
        self.update_guys(delta_time);
        self.update_farticles(delta_time);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.real_time += delta_time;

        if let Some(id) = self.my_guy {
            let guy = self.guys.get(&id).unwrap();
            self.camera.center += (guy.pos - self.camera.center) * (delta_time * 5.0).min(1.0);
        }

        if let Some(editor) = &mut self.editor {
            editor.next_autosave -= delta_time;
            if editor.next_autosave < 0.0 {
                editor.next_autosave = 10.0;
                self.save_level();
            }
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event_editor(&event);
        match event {
            geng::Event::MouseMove { position, .. }
                if self
                    .geng
                    .window()
                    .is_button_pressed(geng::MouseButton::Middle) =>
            {
                let old_pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, self.prev_mouse_pos.map(|x| x as f32));
                let new_pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                self.camera.center += old_pos - new_pos;
            }
            geng::Event::Wheel { delta } if self.opt.editor => {
                self.camera.fov = (self.camera.fov * 1.01f32.powf(-delta as f32)).clamp(1.0, 30.0);
            }
            geng::Event::KeyDown { key: geng::Key::S }
                if self.geng.window().is_key_pressed(geng::Key::LCtrl) =>
            {
                self.save_level();
            }
            _ => {}
        }
        self.prev_mouse_pos = self.geng.window().mouse_pos();
    }
}

#[derive(clap::Parser, Clone)]
pub struct Opt {
    #[clap(long)]
    pub editor: bool,
    #[clap(long)]
    pub server: Option<String>,
    #[clap(long)]
    pub connect: Option<String>,
}

fn main() {
    geng::setup_panic_handler();
    let mut opt: Opt = program_args::parse();

    if opt.connect.is_none() && opt.server.is_none() {
        if cfg!(target_arch = "wasm32") {
            opt.connect = Some(
                option_env!("CONNECT")
                    .unwrap_or("ws://127.0.0.1:1155")
                    // .expect("Set CONNECT compile time env var")
                    .to_owned(),
            );
        } else {
            opt.server = Some("127.0.0.1:1155".to_owned());
            opt.connect = Some("ws://127.0.0.1:1155".to_owned());
        }
    }

    let level_path = static_path().join("level.json");
    logger::init().unwrap();
    let geng = Geng::new_with(geng::ContextOptions {
        title: "LD51 - Stercore Dare".to_owned(),
        fixed_delta_time: 1.0 / 200.0,
        vsync: false,
        ..default()
    });
    let state = geng::LoadingScreen::new(
        &geng,
        geng::EmptyLoadingScreen,
        future::join(
            <Assets as geng::LoadAsset>::load(&geng, &static_path()),
            <String as geng::LoadAsset>::load(&geng, &level_path),
        ),
        {
            let geng = geng.clone();
            move |(assets, level)| {
                let assets = assets.expect("Failed to load assets");
                let level = match level {
                    Ok(json) => serde_json::from_str(&json).unwrap(),
                    Err(_) => Level::empty(),
                };
                Game::new(&geng, &Rc::new(assets), level, opt)
            }
        },
    );
    geng::run(&geng, state);
}
