use super::*;

pub struct System {
    geng: Geng,
    pub volume: f32,
    pub pos: vec2<f32>,
    pub fov: f32,
    pub speed: f32,
}

impl System {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            pos: vec2::ZERO,
            fov: 1.0,
            speed: 1.0,
            volume: 1.0,
        }
    }
    pub fn play(&self, sound: &geng::Sound, volume: f32, pos: vec2<f32>) {
        let mut effect = sound.effect();
        // TODO check formula
        effect.set_volume(
            (self.volume * volume * (1.0 - (pos - self.pos).len() / self.fov)).clamp(0.0, 1.0)
                as f64,
        );
        effect.set_speed(self.speed as f64); // TODO may change over time
        effect.play();
    }

    pub fn sync_with_camera(&mut self, camera: &Camera2d) {
        self.pos = camera.center;
        self.fov = camera.fov;
    }
}
