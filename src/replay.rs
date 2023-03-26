use super::*;

#[derive(Serialize, Deserialize, Clone)]
struct HistoryEntry {
    timestamp: f32,
    input: Input,
    snapshot: PhysicsState,
}

impl HistoryEntry {
    fn new(timestamp: f32, guy: &Guy) -> Self {
        Self {
            timestamp,
            input: guy.input.clone(),
            snapshot: guy.state.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct History {
    customization: CustomizationOptions,
    log: VecDeque<HistoryEntry>,
}

impl History {
    pub fn new(timestamp: f32, guy: &Guy) -> Self {
        let mut log = VecDeque::new();
        log.push_back(HistoryEntry::new(timestamp, guy));
        Self {
            customization: guy.customization.clone(),
            log,
        }
    }
    pub fn push(&mut self, timestamp: f32, guy: &Guy) {
        self.customization = guy.customization.clone();
        self.log.push_back(HistoryEntry {
            timestamp,
            input: guy.input.clone(),
            snapshot: guy.state.clone(),
        });
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        let data: Versioned = self.clone().into();
        bincode::serialize_into(writer, &data)?;
        Ok(())
    }
}

mod v0 {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Ball {
        pub radius: f32,
        pub pos: vec2<f32>,
        pub vel: vec2<f32>,
        pub rot: f32,
        pub w: f32,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct FartState {
        pub long_farting: bool,
        pub fart_pressure: f32,
    }

    impl Default for FartState {
        fn default() -> Self {
            Self {
                long_farting: false,
                fart_pressure: 0.0,
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug, HasId)]
    pub struct Guy {
        pub id: Id,
        pub customization: CustomizationOptions,
        pub ball: Ball,
        pub fart_state: FartState,
        pub input: Input,
        pub animation: GuyAnimationState,
        pub progress: Progress,

        pub fart_type: String,
        pub snow_layer: f32,
        pub cannon_timer: Option<CannonTimer>,
        pub stick_force: vec2<f32>,
        pub bubble_timer: Option<f32>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct HistoryEntry {
        pub timestamp: f32,
        pub snapshot: Guy,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct History(pub VecDeque<HistoryEntry>);
}

#[derive(Serialize, Deserialize)]
pub enum Versioned {
    V0(v0::History),
    V1(History),
}

impl From<History> for Versioned {
    fn from(value: History) -> Self {
        Self::V1(value)
    }
}

impl From<Versioned> for History {
    fn from(value: Versioned) -> Self {
        match value {
            Versioned::V0(history) => Self {
                customization: history.0[0].snapshot.customization.clone(),
                log: history
                    .0
                    .into_iter()
                    .map(|entry| HistoryEntry {
                        timestamp: entry.timestamp,
                        input: entry.snapshot.input,
                        snapshot: PhysicsState {
                            radius: entry.snapshot.ball.radius,
                            pos: entry.snapshot.ball.pos,
                            vel: entry.snapshot.ball.vel,
                            rot: entry.snapshot.ball.rot,
                            w: entry.snapshot.ball.w,
                            fart_type: entry.snapshot.fart_type,
                            long_farting: entry.snapshot.fart_state.long_farting,
                            fart_pressure: entry.snapshot.fart_state.fart_pressure,
                            snow_layer: entry.snapshot.snow_layer,
                            cannon_timer: entry.snapshot.cannon_timer,
                            stick_force: entry.snapshot.stick_force,
                            bubble_timer: entry.snapshot.bubble_timer,
                        },
                    })
                    .collect(),
            },
            Versioned::V1(value) => value,
        }
    }
}

pub fn save(path: impl AsRef<std::path::Path>, histories: &[&History]) -> anyhow::Result<()> {
    let path = path.as_ref();
    std::fs::create_dir_all(path)?;
    for (index, &history) in histories.iter().enumerate() {
        history.save(path.join(format!("{index}.bincode")))?;
    }
    {
        let file = std::fs::File::create(path.join("number.txt"))?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(histories.len().to_string().as_bytes())?;
    }
    Ok(())
}

pub async fn load_histories(path: impl AsRef<std::path::Path>) -> anyhow::Result<Vec<History>> {
    let path = path.as_ref();
    let number = file::load_string(path.join("number.txt"))
        .await
        .context("Failed to load number")?
        .parse()?;
    future::try_join_all((0..number).map(|index| async move {
        let bytes = file::load_bytes(path.join(format!("{index}.bincode")))
            .await
            .context(format!("Failed to load {index}"))?;
        let history: Versioned =
            bincode::deserialize(&bytes).context("Failed to deserialize history")?;
        Ok(history.into())
    }))
    .await
}

pub struct Replay {
    pub history: History,
    next_index: usize,
    current_time: f32,
}

impl Replay {
    pub fn from_history(history: History) -> Self {
        Self {
            current_time: history.log.front().unwrap().timestamp,
            next_index: 0,
            history,
        }
    }
    pub fn new(timestamp: f32, guy: &Guy) -> Self {
        Self {
            history: History::new(timestamp, guy),
            current_time: timestamp,
            next_index: 0,
        }
    }
    pub fn push(&mut self, timestamp: f32, guy: &Guy) {
        self.history.push(timestamp, guy);
    }
    pub fn time_left(&self) -> f32 {
        self.history.log.back().unwrap().timestamp - self.current_time
    }
    pub fn reset(&mut self) {
        self.next_index = 0;
        self.current_time = self.history.log.front().unwrap().timestamp;
    }
    pub fn customization(&self) -> &CustomizationOptions {
        &self.history.customization
    }
    pub fn update(&mut self, delta_time: f32) -> Option<(Input, PhysicsState)> {
        // Check for desync
        if self
            .history
            .log
            .get(self.next_index)
            .map_or(false, |entry| entry.timestamp < self.current_time)
        {
            self.current_time = self.history.log.back().unwrap().timestamp;
            self.next_index = self.history.log.len() - 1;
        }

        self.current_time += delta_time;
        let mut result = None;
        while let Some(entry) = self.history.log.get(self.next_index) {
            if entry.timestamp > self.current_time {
                break;
            }
            self.next_index += 1;
            result = Some(entry);
        }
        result.map(|entry| (entry.input.clone(), entry.snapshot.clone()))
    }
    pub fn trim_beginning(&mut self) {
        while self.next_index > 0 {
            self.next_index -= 1;
            self.history.log.pop_front();
        }
    }
}
