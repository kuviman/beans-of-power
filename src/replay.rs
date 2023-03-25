use super::*;

#[derive(Serialize, Deserialize, Clone)]
struct HistoryEntry {
    timestamp: f32,
    snapshot: Guy,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct History(VecDeque<HistoryEntry>);

impl History {
    pub fn push(&mut self, timestamp: f32, snapshot: Guy) {
        self.0.push_back(HistoryEntry {
            timestamp,
            snapshot,
        });
    }
}

pub struct Replay {
    pub history: History,
    next_index: usize,
    current_time: f32,
    // current_state: Guy,
}

impl Replay {
    pub fn from_history(history: History) -> Self {
        Self {
            current_time: history.0.front().unwrap().timestamp,
            next_index: 0,
            history,
        }
    }
    pub fn new(timestamp: f32, snapshot: Guy) -> Self {
        let current_state = snapshot.clone();
        let mut history = VecDeque::new();
        history.push_back(HistoryEntry {
            timestamp,
            snapshot,
        });
        Self {
            history: History(history),
            // current_state,
            current_time: timestamp,
            next_index: 0,
        }
    }
    pub fn push(&mut self, timestamp: f32, snapshot: Guy) {
        self.history.push(timestamp, snapshot);
    }
    pub fn time_left(&self) -> f32 {
        self.history.0.back().unwrap().timestamp - self.current_time
    }
    pub fn reset(&mut self) {
        self.next_index = 0;
        self.current_time = self.history.0.front().unwrap().timestamp;
    }
    pub fn update(&mut self, delta_time: f32) -> Option<Guy> {
        // Check for desync
        if self
            .history
            .0
            .get(self.next_index)
            .map_or(false, |entry| entry.timestamp < self.current_time)
        {
            self.current_time = self.history.0.back().unwrap().timestamp;
            self.next_index = self.history.0.len() - 1;
        }

        self.current_time += delta_time;
        let mut result = None;
        while let Some(entry) = self.history.0.get(self.next_index) {
            if entry.timestamp > self.current_time {
                break;
            }
            self.next_index += 1;
            result = Some(&entry.snapshot);
        }
        result.cloned()
    }
    pub fn trim_beginning(&mut self) {
        while self.next_index > 0 {
            self.next_index -= 1;
            self.history.0.pop_front();
        }
    }
}
