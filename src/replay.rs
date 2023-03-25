use super::*;

struct HistoryEntry {
    timestamp: f32,
    snapshot: Guy,
}

pub struct Replay {
    history: VecDeque<HistoryEntry>,
    current_history_index: usize,
    current_time: f32,
    // current_state: Guy,
}

impl Replay {
    pub fn new(timestamp: f32, snapshot: Guy) -> Self {
        let current_state = snapshot.clone();
        let mut history = VecDeque::new();
        history.push_back(HistoryEntry {
            timestamp,
            snapshot,
        });
        Self {
            history,
            // current_state,
            current_time: timestamp,
            current_history_index: 0,
        }
    }
    pub fn push(&mut self, timestamp: f32, snapshot: Guy) {
        self.history.push_back(HistoryEntry {
            timestamp,
            snapshot,
        });
    }
    pub fn time_left(&self) -> f32 {
        self.history.back().unwrap().timestamp - self.current_time
    }
    pub fn update(&mut self, delta_time: f32) -> Option<Guy> {
        // Check for desync
        if self.history.back().unwrap().timestamp < self.current_time
            && self.current_history_index != self.history.len() - 1
        {
            self.current_time = self.history.back().unwrap().timestamp;
            self.current_history_index = self.history.len() - 1;
        }

        self.current_time += delta_time;
        let mut result = None;
        while self.current_history_index + 1 < self.history.len()
            && self.history[self.current_history_index + 1].timestamp <= self.current_time
        {
            self.current_history_index += 1;
            result = Some(self.history[self.current_history_index].snapshot.clone());
        }
        result
    }
    pub fn trim_beginning(&mut self) {
        while self.current_history_index > 0 {
            self.current_history_index -= 1;
            self.history.pop_front();
        }
    }
}
