use super::*;

impl Game {
    pub fn update_remote(&mut self, delta_time: f32) {
        let mut to_remove = Vec::new();
        for (&id, replay) in &mut self.remote_updates {
            if let Some(new_snapshot) = replay.update(delta_time) {
                self.guys.insert(new_snapshot);
            }
            // TODO speedup replay instead?
            if replay.time_left() > 5.0 {
                to_remove.push(id);
            }
            replay.trim_beginning();
        }
        for id in to_remove {
            self.guys.remove(&id);
            self.remote_updates.remove(&id);
        }
    }
}
