use super::*;

impl Game {
    pub fn update_remote(&mut self) {
        for (&id, updates) in &mut self.remote_updates {
            let current_simulation_time = match self.remote_simulation_times.get(&id) {
                Some(x) => *x,
                None => continue,
            };
            if let Some(update) = updates.back() {
                if (update.0 - current_simulation_time).abs() > 5.0 {
                    updates.clear();
                    self.remote_simulation_times.remove(&id);
                    self.guys.remove(&id);
                    continue;
                }
            }
            while let Some(update) = updates.front() {
                if (update.0 - current_simulation_time).abs() > 5.0 {
                    updates.clear();
                    self.remote_simulation_times.remove(&id);
                    self.guys.remove(&id);
                    break;
                }
                if update.0 <= current_simulation_time {
                    let update = updates.pop_front().unwrap().1;
                    self.guys.insert(update);
                } else {
                    break;
                }
            }
        }
    }
}
