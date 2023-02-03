use super::*;

mod cannon;
mod draw;
mod object;
mod progress;
mod surface;
mod tile;

pub use cannon::*;
pub use object::*;
pub use surface::*;
pub use tile::*;

#[derive(Serialize, Deserialize)]
pub struct LevelInfo {
    pub spawn_point: vec2<f32>,
    pub finish_point: vec2<f32>,
    pub surfaces: Vec<Surface>,
    pub tiles: Vec<Tile>,
    pub expected_path: Vec<vec2<f32>>,
    pub objects: Vec<Object>,
    pub cannons: Vec<Cannon>,
}

impl LevelInfo {
    pub fn empty() -> Self {
        Self {
            spawn_point: vec2::ZERO,
            finish_point: vec2::ZERO,
            surfaces: vec![],
            tiles: vec![],
            expected_path: vec![],
            objects: vec![],
            cannons: vec![],
        }
    }
}

#[derive(Deref)]
pub struct Level {
    #[deref]
    info: LevelInfo,
    mesh: RefCell<Option<draw::LevelMesh>>,
}

impl Level {
    pub fn new(info: LevelInfo) -> Self {
        Self {
            info,
            mesh: RefCell::new(None),
        }
    }
    pub fn info(&self) -> &LevelInfo {
        &self.info
    }
    pub fn modify(&mut self) -> &mut LevelInfo {
        *self.mesh.get_mut() = None;
        &mut self.info
    }
}
