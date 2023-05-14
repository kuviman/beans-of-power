use super::*;

impl Game {
    fn update_replay(id: Id, replay: &mut Replay, delta_time: f32, guys: &mut Collection<Guy>) {
        if let Some((input, snapshot)) = replay.update(delta_time) {
            match guys.get_mut(&id) {
                Some(guy) => {
                    guy.input = input;
                    guy.state = snapshot;
                    guy.customization = replay.customization().clone();
                }
                None => {
                    let guy = Guy {
                        id,
                        customization: replay.customization().clone(),
                        input,
                        state: snapshot,
                        animation: default(),
                        progress: default(),
                        paused: false,
                    };
                    guys.insert(guy);
                }
            }
        }
    }
    pub fn update_remote(&mut self, delta_time: f32) {
        let mut to_remove = Vec::new();
        for (&id, replay) in &mut self.remote_updates {
            Self::update_replay(id, replay, delta_time, &mut self.guys);
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

    pub fn update_replays(&mut self, delta_time: f32) {
        for (i, replay) in self.replays.iter_mut().enumerate() {
            Self::update_replay(Id::replay(i), replay, delta_time, &mut self.guys);
            if replay.time_left() < 0.0 {
                replay.reset();
            }
        }
    }
}
