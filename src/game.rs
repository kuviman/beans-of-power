use super::*;

pub const CONTROLS_LEFT: [geng::Key; 2] = [geng::Key::A, geng::Key::Left];
pub const CONTROLS_RIGHT: [geng::Key; 2] = [geng::Key::D, geng::Key::Right];
pub const CONTROLS_FORCE_FART: [geng::Key; 3] = [geng::Key::W, geng::Key::Up, geng::Key::Space];

pub struct LongFartSfx {
    pub finish_time: Option<f32>,
    pub type_name: String,
    pub sfx: geng::SoundEffect,
}

pub struct Game {
    pub best_time: Option<f32>,
    pub emotes: Vec<(f32, Id, usize)>,
    pub best_progress: f32,
    pub framebuffer_size: vec2<f32>,
    pub prev_mouse_pos: vec2<f64>,
    pub geng: Geng,
    pub config: Rc<Config>,
    pub assets: AssetsHandle,
    pub camera: geng::Camera2d,
    pub level: Level,
    pub editor: Option<EditorState>,
    pub guys: Collection<Guy>,
    pub my_guy: Option<Id>,
    pub simulation_time: f32,
    pub remote_updates: HashMap<Id, Replay>,
    pub real_time: f32,
    pub noise: noise::OpenSimplex,
    pub opt: Opt,
    pub farticles: HashMap<String, Vec<Farticle>>,
    pub volume: f32,
    pub client_id: Id,
    pub connection: Option<Connection>,
    pub customization: CustomizationOptions,
    pub mute_music: bool,
    pub ui_controller: ui::Controller,
    pub buttons: Vec<ui::Button<UiMessage>>,
    pub show_customizer: bool,
    pub music: geng::SoundEffect,
    pub show_names: bool,
    pub show_leaderboard: bool,
    pub follow: Option<Id>,
    pub long_fart_sfx: HashMap<Id, LongFartSfx>,
    pub next_golden_glint: f32,
    pub time_scale: f32,
    pub quicksave: Option<Guy>,
    pub replays: Vec<Replay>,
    pub recording: Option<Replay>,
    pub video_editor: Option<video_editor::VideoEditor>,
}

impl Drop for Game {
    fn drop(&mut self) {
        if let Some(editor) = &mut self.editor {
            editor.save_level(&mut self.level);
        }
    }
}

impl Game {
    pub fn new(
        geng: &Geng,
        assets: &AssetsHandle,
        level: Level,
        opt: Opt,
        connection_info: Option<(Id, Connection)>,
    ) -> Self {
        let (client_id, connection) = match connection_info {
            Some((client_id, connection)) => (client_id, Some(connection)),
            None => (Id::LOCALHOST, None),
        };
        let mut result = Self {
            best_time: None,
            emotes: vec![],
            geng: geng.clone(),
            config: assets.get().config.clone(),
            assets: assets.clone(),
            camera: geng::Camera2d {
                center: level.spawn_point,
                rotation: 0.0,
                fov: assets.get().config.camera_fov,
            },
            framebuffer_size: vec2(1.0, 1.0),
            editor: if opt.editor {
                Some(EditorState::new(geng, assets))
            } else {
                None
            },
            level,
            guys: Collection::new(),
            my_guy: None,
            real_time: 0.0,
            noise: noise::OpenSimplex::new(0),
            prev_mouse_pos: vec2::ZERO,
            opt: opt.clone(),
            farticles: default(),
            volume: assets.get().config.volume,
            client_id,
            connection,
            simulation_time: 0.0,
            remote_updates: default(),
            customization: preferences::load("customization")
                .unwrap_or_else(CustomizationOptions::random),
            mute_music: false,
            best_progress: 0.0,
            ui_controller: ui::Controller::new(geng, assets),
            buttons: vec![
                ui::Button::new("PLAY", vec2(0.0, -3.0), 1.0, 0.5, UiMessage::Play),
                ui::Button::new(
                    "randomize",
                    vec2(2.0, 0.0),
                    0.7,
                    0.0,
                    UiMessage::RandomizeSkin,
                ),
            ],
            show_customizer: !opt.editor,
            music: {
                let mut effect = assets.get().sfx.new_music.play();
                effect.set_volume(0.0);
                effect
            },
            show_names: true,
            show_leaderboard: true,
            follow: None,
            long_fart_sfx: HashMap::new(),
            next_golden_glint: 0.0,
            quicksave: None,
            time_scale: 1.0,
            replays: if cfg!(target_arch = "wasm32") {
                vec![]
            } else {
                let path = run_dir().join("replays");
                if path.exists() {
                    futures::executor::block_on(replay::load_histories(path))
                        .unwrap()
                        .into_iter()
                        .map(Replay::from_history)
                        .collect()
                } else {
                    vec![]
                }
            },
            recording: None,
            video_editor: opt
                .video
                .as_ref()
                .map(|path| video_editor::VideoEditor::new(geng, path)),
        };
        if !opt.editor {
            result.my_guy = Some(client_id);
            result.guys.insert(Guy::new(
                client_id,
                result.level.spawn_point,
                true,
                &result.config,
            ));
        }
        result
    }

    fn draw_progress(&mut self, framebuffer: &mut ugli::Framebuffer) {
        if self.show_customizer {
            return;
        }
        if let Some(id) = self.my_guy {
            let camera = geng::Camera2d {
                center: vec2::ZERO,
                rotation: 0.0,
                fov: 10.0,
            };
            let guy = self.guys.get_mut(&id).unwrap();
            let text_color = if guy.progress.finished {
                Rgba::WHITE
            } else {
                Rgba::BLACK
            };
            if guy.progress.finished {
                self.assets.get().font.draw(
                    framebuffer,
                    &camera,
                    &"GG",
                    vec2(0.0, 3.0),
                    geng::TextAlign::CENTER,
                    1.5,
                    text_color,
                );
            }
            let progress = self
                .level
                .progress_at(guy.state.pos)
                .unwrap_or(guy.progress.current);
            guy.progress.current = progress;
            self.best_progress = self.best_progress.max(progress);
            guy.progress.best = self.best_progress;
            if guy.progress.finished && self.simulation_time < self.best_time.unwrap_or(1e9) {
                self.best_time = Some(self.simulation_time);
            }
            guy.progress.best_time = self.best_time;
            let mut time_text = String::new();
            let seconds = self.simulation_time.round() as i32;
            let minutes = seconds / 60;
            let seconds = seconds % 60;
            let hours = minutes / 60;
            let minutes = minutes % 60;
            if hours != 0 {
                time_text += &format!("{} hours ", hours);
            }
            if minutes != 0 {
                time_text += &format!("{} minutes ", minutes);
            }
            time_text += &format!("{} seconds", seconds);
            self.assets.get().font.draw(
                framebuffer,
                &camera,
                &time_text,
                vec2(0.0, -3.3),
                geng::TextAlign::CENTER,
                0.5,
                text_color,
            );
            if !guy.progress.finished {
                self.assets.get().font.draw(
                    framebuffer,
                    &camera,
                    &"progress",
                    vec2(0.0, -4.0),
                    geng::TextAlign::CENTER,
                    0.5,
                    text_color,
                );
                self.geng.draw_2d(
                    framebuffer,
                    &camera,
                    &draw_2d::Quad::new(
                        Aabb2::point(vec2(0.0, -4.5)).extend_symmetric(vec2(3.0, 0.1)),
                        text_color,
                    ),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &camera,
                    &draw_2d::Quad::new(
                        Aabb2::point(vec2(-3.0 + 6.0 * self.best_progress, -4.5))
                            .extend_uniform(0.3),
                        Rgba::new(0.0, 0.0, 0.0, 0.5),
                    ),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &camera,
                    &draw_2d::Quad::new(
                        Aabb2::point(vec2(-3.0 + 6.0 * progress, -4.5)).extend_uniform(0.3),
                        text_color,
                    ),
                );
            }
        }
    }
    fn save_replays(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            replay::save(
                run_dir().join("replays"),
                &self
                    .replays
                    .iter()
                    .map(|replay| &replay.history)
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        let finished = self
            .my_guy
            .and_then(|id| self.guys.get(&id))
            .map(|guy| guy.progress.finished)
            .unwrap_or(false);
        ugli::clear(
            framebuffer,
            Some(if finished {
                Rgba::BLACK
            } else {
                self.config.background_color
            }),
            None,
            None,
        );

        for (index, layer) in self.level.layers.iter().enumerate() {
            if !finished {
                self.draw_layer_back(&self.level, index, framebuffer);
            }
            if layer.name == "main" {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&self.assets.get().closed_outhouse)
                        .translate(self.level.spawn_point),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&self.assets.get().golden_toilet)
                        .translate(self.level.finish_point),
                );
                self.draw_guys(framebuffer);
                self.draw_farticles(framebuffer);
            }
            if !finished {
                self.draw_layer_front(&self.level, index, framebuffer);
            }
        }
        self.draw_level_editor(framebuffer);
        self.draw_customizer(framebuffer);
        self.draw_leaderboard(framebuffer);
        self.draw_progress(framebuffer);

        if self.recording.is_some() {
            self.geng.default_font().draw(
                framebuffer,
                &geng::PixelPerfectCamera,
                "RECORDING",
                vec2::ZERO,
                geng::TextAlign::LEFT,
                64.0,
                Rgba::RED,
            );
        }
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32 * self.time_scale;
        if self.my_guy.is_none()
            || !self
                .guys
                .get(&self.my_guy.unwrap())
                .unwrap()
                .progress
                .finished
        {
            self.simulation_time += delta_time;
        }
        self.update_my_guy_input();
        self.update_guys(delta_time);
        self.update_farticles(delta_time);
        self.update_remote(delta_time);
        self.update_replays(delta_time);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        // self.volume = self.assets.config.volume;
        if self.geng.window().is_key_pressed(geng::Key::PageUp) {
            self.volume += delta_time * 0.5;
        }
        if self.geng.window().is_key_pressed(geng::Key::PageDown) {
            self.volume -= delta_time * 0.5;
        }
        self.volume = self.volume.clamp(0.0, 1.0);
        if self.mute_music {
            self.music.set_volume(0.0);
        } else {
            self.music.set_volume(self.volume as f64);
        }

        self.emotes.retain(|&(t, ..)| t >= self.real_time - 1.0);

        self.real_time += delta_time;

        let mut target_center = self.camera.center;
        if let Some(id) = self.my_guy {
            let guy = self.guys.get(&id).unwrap();
            target_center = guy.state.pos;
            if self.show_customizer {
                target_center.x += 1.0;
            }
        } else if let Some(id) = self.follow {
            if let Some(guy) = self.guys.get(&id) {
                target_center = guy.state.pos;
            }
        }
        self.camera.center += (target_center - self.camera.center) * (delta_time * 5.0).min(1.0);

        if self.editor.is_none() {
            // let target_fov = if self.show_customizer { 2.0 } else { 6.0 };
            // self.camera.fov += (target_fov - self.camera.fov) * delta_time;
        }

        if let Some(editor) = &mut self.editor {
            editor.update(&mut self.level, delta_time);
        }

        self.handle_connection();

        if let Some(id) = self.my_guy {
            let guy = self.guys.get_mut(&id).unwrap();
            guy.customization.name = self.customization.name.clone();
            guy.customization.colors = self.customization.colors.clone();
        }

        self.next_golden_glint -= delta_time;
        if self.next_golden_glint < 0.0 {
            let fart_type = "glint".to_owned();
            let assets = self.assets.get();
            let fart_assets = &assets.farts[&fart_type];
            self.next_golden_glint = 1.0 / fart_assets.config.farticle_count as f32;
            self.farticles.entry(fart_type).or_default().push(Farticle {
                size: fart_assets.config.farticle_size,
                pos: thread_rng().gen_circle(self.level.finish_point, 1.0),
                vel: thread_rng()
                    .gen_circle(vec2::ZERO, fart_assets.config.farticle_additional_vel),
                rot: thread_rng().gen_range(0.0..2.0 * f32::PI),
                w: thread_rng()
                    .gen_range(-fart_assets.config.farticle_w..=fart_assets.config.farticle_w),
                colors: fart_assets.config.colors.get(),
                t: 1.0,
            });
        }

        self.update_video_editor(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        self.handle_event_editor(&event);
        self.handle_customizer_event(&event);
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
            geng::Event::MouseDown {
                position,
                button: geng::MouseButton::Left,
            } if self.my_guy.is_none() && self.editor.is_none() => {
                let pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size, position.map(|x| x as f32));
                if let Some(guy) = self
                    .guys
                    .iter()
                    .min_by_key(|guy| r32((guy.state.pos - pos).len()))
                {
                    if (guy.state.pos - pos).len() < guy.radius() {
                        self.follow = Some(guy.id);
                    }
                }
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Right,
                ..
            } => {
                self.follow = None;
            }
            geng::Event::Wheel { delta } if self.opt.editor => {
                self.camera.fov = (self.camera.fov * 1.01f32.powf(-delta as f32)).clamp(1.0, 200.0);
            }
            geng::Event::KeyDown { key: geng::Key::R }
                if self.geng.window().is_key_pressed(geng::Key::LCtrl) =>
            {
                self.respawn_my_guy();
            }
            geng::Event::KeyDown { key: geng::Key::M } if !self.show_customizer => {
                self.mute_music = !self.mute_music;
            }
            geng::Event::KeyDown { key: geng::Key::H } if !self.show_customizer => {
                self.show_names = !self.show_names;
            }
            geng::Event::KeyDown { key: geng::Key::L } if !self.show_customizer => {
                self.show_leaderboard = !self.show_leaderboard;
            }
            geng::Event::KeyDown {
                key: geng::Key::Num1,
            } => {
                if let Some(con) = &mut self.connection {
                    con.send(ClientMessage::Emote(0));
                }
            }
            geng::Event::KeyDown {
                key: geng::Key::Num2,
            } => {
                if let Some(con) = &mut self.connection {
                    con.send(ClientMessage::Emote(1));
                }
            }
            geng::Event::KeyDown {
                key: geng::Key::Num3,
            } => {
                if let Some(con) = &mut self.connection {
                    con.send(ClientMessage::Emote(2));
                }
            }
            geng::Event::KeyDown {
                key: geng::Key::Num4,
            } => {
                if let Some(con) = &mut self.connection {
                    con.send(ClientMessage::Emote(3));
                }
            }
            geng::Event::KeyDown {
                key: geng::Key::Tab,
            } if self.opt.editor => {
                if self.editor.take().is_none() {
                    self.editor = Some(EditorState::new(&self.geng, &self.assets));
                }
            }
            geng::Event::KeyDown { key: geng::Key::I } => {
                self.camera.fov = self.assets.get().config.camera_fov;
            }
            geng::Event::KeyDown { key: geng::Key::F5 } if self.opt.editor => {
                self.quicksave = self.my_guy.and_then(|id| self.guys.get(&id)).cloned();
            }
            geng::Event::KeyDown { key: geng::Key::F7 } if self.opt.editor => {
                if let Some(save) = &self.quicksave {
                    let save = save.clone();
                    self.respawn_my_guy();
                    *self.guys.get_mut(&self.my_guy.unwrap()).unwrap() = save;
                }
            }
            geng::Event::KeyDown { key: geng::Key::Z } if self.opt.editor => {
                self.time_scale = 1.0;
            }
            geng::Event::KeyDown { key: geng::Key::X } if self.opt.editor => {
                self.time_scale = 0.5;
            }
            geng::Event::KeyDown { key: geng::Key::C } if self.opt.editor => {
                self.time_scale = 0.25;
            }
            geng::Event::KeyDown { key: geng::Key::Q } if self.opt.editor => {
                if self.geng.window().is_key_pressed(geng::Key::LCtrl) {
                    if let Some(mut recording) = self.recording.take() {
                        if let Some(guy) = self.my_guy.and_then(|id| self.guys.get(&id)) {
                            recording.push(self.simulation_time, guy);
                        }
                        self.replays.push(recording);
                        self.save_replays();
                    } else if let Some(guy) = self.my_guy.and_then(|id| self.guys.get(&id)) {
                        self.recording = Some(Replay::new(self.simulation_time, guy));
                    }
                }
            }
            _ => {}
        }
        self.prev_mouse_pos = self.geng.window().mouse_pos();
    }
    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        let mut result = geng::ui::Void.boxed();
        if self.editor.is_some() {
            result = stack![result, self.editor_ui(cx)].boxed();
        } else if self.video_editor.is_some() {
            result = stack![result, self.video_editor_ui(cx)].boxed();
        }
        result
    }
}
