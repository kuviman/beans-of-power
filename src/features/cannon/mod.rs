use super::*;

pub mod editor;

#[derive(geng::asset::Load, Deserialize, Clone, Debug)]
#[load(serde = "json")] // TODO toml
pub struct Config {
    pub strength: f32,
    pub activate_distance: f32,
    pub shoot_time: f32,
    pub particle_speed: f32,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub farticles: Rc<FartAssets>,
    pub body: Texture,
    pub base: Texture,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cannon {
    pub pos: vec2<f32>,
    pub rot: Angle<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelInfo {
    pub cannons: Vec<Cannon>,
}

impl Default for LevelInfo {
    fn default() -> Self {
        Self { cannons: vec![] }
    }
}

impl LevelInfo {
    pub fn draw(
        &self,
        geng: &Geng,
        assets: &assets::Assets,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera2d,
    ) {
        for cannon in &self.cannons {
            let mut scale = vec2(1.0, 1.0);
            if cannon.rot > Angle::from_radians(f32::PI / 2.0)
                || cannon.rot < Angle::from_radians(-f32::PI / 2.0)
            {
                scale.x = -scale.x;
            }
            geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::TexturedQuad::unit(&assets.cannon.body)
                    .rotate(cannon.rot)
                    .translate(cannon.pos),
            );
            geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::TexturedQuad::unit(&assets.cannon.base)
                    .scale(scale)
                    .translate(cannon.pos),
            );
        }
    }
}

pub fn update_guy(
    guy: &mut guy::PhysicsState,
    delta_time: f32,
    level: &level::LevelInfo,
    config: &assets::Config,
    assets: &assets::Assets, // TODO: optional
    sound: &sound::System,
    farticles: &mut farticle::System,
) -> std::ops::ControlFlow<()> {
    let assets = &assets.cannon;
    let config = &config.cannon;

    // This is where we do the cannon mechanics aha
    if guy.cannon_timer.is_none() {
        for (index, cannon) in level.cannon.cannons.iter().enumerate() {
            if (guy.pos - cannon.pos).len() < config.activate_distance {
                guy.long_farting = false;
                guy.fart_pressure = 0.0;
                guy.cannon_timer = Some(CannonTimer {
                    cannon_index: index,
                    time: config.shoot_time,
                });
            }
        }
    }
    if let Some(timer) = &mut guy.cannon_timer {
        let cannon = &level.cannon.cannons[timer.cannon_index];
        guy.pos = cannon.pos;
        guy.rot = cannon.rot - Angle::from_radians(f32::PI / 2.0);
        timer.time -= delta_time;
        if timer.time < 0.0 {
            guy.cannon_timer = None;
            let dir = vec2(1.0, 0.0).rotate(cannon.rot);

            // TODO wtf does 1.01 mean?
            // Probably to stop getting triggered by the same cannon after being fired
            guy.pos += dir * config.activate_distance * 1.01;

            guy.vel = dir * config.strength;
            guy.w = Angle::ZERO;

            sound.play(
                assets.farticles.sfx.choose(&mut thread_rng()).unwrap(),
                1.0,
                guy.pos,
            );
            farticles.spawn(&assets.farticles, guy.pos, dir * config.particle_speed);
        }
        return std::ops::ControlFlow::Break(());
    }
    std::ops::ControlFlow::Continue(())
}
