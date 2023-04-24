use super::*;

#[derive(Deserialize)]
struct Config {
    music: std::path::PathBuf,
}

#[derive(Serialize, Deserialize)]
enum CameraInfo {
    Static(geng::Camera2d),
    Follow(Id),
}

#[derive(Serialize, Deserialize)]
struct SegmentConfig {
    start_time: f32,
    camera_info: CameraInfo,
}

struct Segment {
    config: SegmentConfig,
    replays: Vec<History>,
}

struct Save {
    segments: Vec<Segment>,
}

impl Save {
    fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
        std::fs::create_dir_all(path)?;
        for (index, segment) in self.segments.iter().enumerate() {
            let path = path.join("segments").join(index.to_string());
            replay::save(&path, &segment.replays.iter().collect::<Vec<_>>())?;
            serde_json::to_writer_pretty(
                std::fs::File::create(path.join("config.json"))?,
                &segment.config,
            )?;
        }
        Ok(())
    }
    fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self {
                segments: vec![Segment {
                    config: SegmentConfig {
                        start_time: 0.0,
                        camera_info: CameraInfo::Static(geng::Camera2d {
                            center: vec2::ZERO,
                            rotation: 0.0,
                            fov: 10.0,
                        }),
                    },
                    replays: vec![],
                }],
            });
        }
        let mut segments = std::collections::BTreeMap::new();
        for entry in std::fs::read_dir(path.join("segments"))? {
            let entry = entry?;
            let index: usize = entry.file_name().to_str().unwrap().parse()?;
            let replays = futures::executor::block_on(replay::load_histories(entry.path()))?;
            let config =
                futures::executor::block_on(file::load_json(entry.path().join("config.json")))?;
            segments.insert(index, Segment { config, replays });
        }
        Ok(Self {
            segments: segments.into_values().collect(),
        })
    }
    fn load2(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        match std::fs::File::open(path.as_ref().with_extension("json")) {
            Ok(file) => {
                #[derive(Deserialize)]
                struct SegmentData {
                    #[serde(flatten)]
                    config: SegmentConfig,
                    replays: Vec<replay::Versioned>,
                }
                #[derive(Deserialize)]
                struct Data {
                    segments: Vec<SegmentData>,
                }
                let data: Data = serde_json::from_reader(std::io::BufReader::new(file)).unwrap();
                Ok(Self {
                    segments: data
                        .segments
                        .into_iter()
                        .map(|segment| Segment {
                            config: segment.config,
                            replays: segment.replays.into_iter().map(|v| v.into()).collect(),
                        })
                        .collect(),
                })
            }
            _ => todo!(),
        }
    }
}

pub struct VideoEditor {
    path: std::path::PathBuf,
    music: geng::Sound,
    music_effect: Option<geng::SoundEffect>,
    time: f32,
    current_segment: usize,
    playthrough: bool,
    save: Save,
}

impl Drop for VideoEditor {
    fn drop(&mut self) {
        self.save.save(self.path.join("save")).unwrap();
    }
}

impl VideoEditor {
    pub fn new(geng: &Geng, path: impl AsRef<std::path::Path>) -> Self {
        use futures::executor::block_on;
        let path = path.as_ref();
        let config: Config = block_on(file::load_json(path.join("config.json"))).unwrap();
        let music: geng::Sound =
            block_on(geng.asset_manager().load(path.join(&config.music))).unwrap();
        Self {
            path: path.to_owned(),
            music,
            save: Save::load(path.join("save")).unwrap(),
            music_effect: None,
            time: 0.0,
            current_segment: 0,
            playthrough: false,
        }
    }
}

impl VideoEditor {
    fn stop(game: &mut Game) {
        let editor = game.video_editor.as_mut().unwrap();
        if let Some(mut rec) = game.recording.take() {
            if let Some(guy) = game.my_guy.and_then(|id| game.guys.get(&id)) {
                rec.push(game.simulation_time, guy);
            }
            editor.save.segments[editor.current_segment]
                .replays
                .push(rec.history);
        }
    }
    fn restart_segment(game: &mut Game, segment_index: usize) {
        let editor = game.video_editor.as_mut().unwrap();
        let segment = &editor.save.segments[segment_index];
        game.guys.retain(|guy| Some(guy.id) == game.my_guy);
        editor.time = segment.config.start_time;
        game.replays = segment
            .replays
            .iter()
            .cloned()
            .map(Replay::from_history)
            .collect();
        if editor.playthrough {
            match &segment.config.camera_info {
                CameraInfo::Static(camera) => {
                    game.camera = camera.clone();
                    game.follow = None;
                }
                &CameraInfo::Follow(id) => {
                    game.follow = Some(id);
                }
            }
        }
        if true && segment_index == 0 {
            game.music.stop();
            game.music = editor.music.effect();
            game.music
                .play_from(time::Duration::from_secs_f64(editor.time as f64));
        }
    }
}

impl Game {
    pub fn update_video_editor(&mut self, delta_time: f32) {
        let game = self;
        if let Some(editor) = &mut game.video_editor {
            let segment_before = editor
                .save
                .segments
                .iter()
                .rposition(|segment| segment.config.start_time <= editor.time);
            editor.time += delta_time;
            let segment_after = editor
                .save
                .segments
                .iter()
                .rposition(|segment| segment.config.start_time <= editor.time);
            if segment_before != segment_after {
                if let Some(index) = segment_after {
                    if editor.playthrough {
                        VideoEditor::stop(game);
                        VideoEditor::restart_segment(game, index);
                    } else {
                        let index = editor.current_segment;
                        VideoEditor::stop(game);
                        VideoEditor::restart_segment(game, index);
                    }
                }
            }
        }
    }
    pub fn video_editor_ui<'a>(
        &'a mut self,
        cx: &'a geng::ui::Controller,
    ) -> Box<dyn geng::ui::Widget + 'a> {
        use geng::ui::*;
        let game = self;
        let Some(editor) = &mut game.video_editor else {
            return geng::ui::Void.boxed();
        };
        if true && editor.playthrough {
            return Void.boxed();
        }
        let current_segment_text = editor.current_segment.to_string();
        let start_recording = Button::new(cx, "rec");
        let next_segment = Button::new(cx, "next");
        if next_segment.was_clicked() {
            editor.current_segment =
                (editor.current_segment + 1).min(editor.save.segments.len() - 1);
        }
        let prev_segment = Button::new(cx, "prev");
        if prev_segment.was_clicked() {
            editor.current_segment = editor.current_segment.max(1) - 1;
        }
        let new_segment = Button::new(cx, "new");
        if new_segment.was_clicked() {
            editor.save.segments.push(Segment {
                config: SegmentConfig {
                    start_time: editor.time,
                    camera_info: CameraInfo::Static(game.camera.clone()),
                },
                replays: vec![],
            })
        }
        let cam = Button::new(cx, "cam");
        if cam.was_clicked() {
            editor.save.segments[editor.current_segment]
                .config
                .camera_info = match game.follow {
                Some(id) => CameraInfo::Follow(id),
                None => CameraInfo::Static(game.camera.clone()),
            };
        }
        let undo = Button::new(cx, "undo");
        if undo.was_clicked() {
            editor.save.segments[editor.current_segment].replays.pop();
        }
        let playthrough = Button::new(cx, &format!("playthrough: {:?}", editor.playthrough));
        if playthrough.was_clicked() {
            editor.playthrough = !editor.playthrough;
        }
        let play = Button::new(cx, "play");
        if start_recording.was_clicked() {
            if let Some(guy) = game.my_guy.and_then(|id| game.guys.get(&id)) {
                game.recording = Some(Replay::new(game.simulation_time, guy));
                let segment = editor.current_segment;
                VideoEditor::restart_segment(game, segment);
            }
        } else if play.was_clicked() {
            if true {
                editor.playthrough = true;
            }
            let segment = editor.current_segment;
            VideoEditor::stop(game);
            VideoEditor::restart_segment(game, segment);
        }
        geng::ui::column![
            start_recording,
            next_segment,
            prev_segment,
            new_segment,
            undo,
            cam,
            play,
            current_segment_text,
            playthrough,
        ]
        .align(vec2(0.0, 1.0))
        .boxed()
    }
}
