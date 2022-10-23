use super::*;

mod draw;
mod object;
mod surface;
mod tile;

pub use object::*;
pub use surface::*;
pub use tile::*;

pub struct Levels {
    pub jam: Level,
    pub postjam: Level,
}

impl Levels {
    pub fn get(&self, postjam: bool) -> &Level {
        if postjam {
            &self.postjam
        } else {
            &self.jam
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub spawn_point: Vec2<f32>,
    pub finish_point: Vec2<f32>,
    pub surfaces: Vec<Surface>,
    pub tiles: Vec<Tile>,
    pub expected_path: Vec<Vec2<f32>>,
    pub objects: Vec<Object>,
    #[serde(skip)]
    pub mesh: RefCell<Option<draw::LevelMesh>>,
}

impl Level {
    pub fn empty() -> Self {
        Self {
            spawn_point: Vec2::ZERO,
            finish_point: Vec2::ZERO,
            surfaces: vec![],
            tiles: vec![],
            expected_path: vec![],
            objects: vec![],
            mesh: default(),
        }
    }
}
