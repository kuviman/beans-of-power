use super::*;

mod cannon;
mod draw;
mod object;
mod portal;
mod progress;
mod surface;
mod tile;

pub use cannon::*;
pub use object::*;
pub use portal::*;
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
    pub portals: Vec<Portal>,
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
            portals: vec![],
        }
    }
}

#[derive(Deref)]
pub struct Level {
    #[deref]
    info: LevelInfo,
    mesh: RefCell<Option<draw::LevelMesh>>,
    saved: bool,
}

impl Level {
    pub fn new(info: LevelInfo) -> Self {
        Self {
            info,
            mesh: RefCell::new(None),
            saved: true,
        }
    }
    pub fn info(&self) -> &LevelInfo {
        &self.info
    }
    pub fn modify(&mut self) -> &mut LevelInfo {
        *self.mesh.get_mut() = None;
        self.saved = false;
        &mut self.info
    }
    /// true if need to save now, next call will return false
    pub fn save(&mut self) -> bool {
        !mem::replace(&mut self.saved, true)
    }
}
