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
pub struct LevelLayer {
    pub name: String,
    pub gameplay: bool,
    pub surfaces: Vec<Surface>,
    pub tiles: Vec<Tile>,
    pub objects: Vec<Object>,
    #[serde(default = "default_parallax")]
    pub parallax: vec2<f32>,
}

fn default_parallax() -> vec2<f32> {
    vec2(1.0, 1.0)
}

#[derive(Serialize, Deserialize)]
pub struct LevelInfo {
    pub spawn_point: vec2<f32>,
    pub finish_point: vec2<f32>,
    pub expected_path: Vec<vec2<f32>>,
    pub layers: Vec<LevelLayer>,
    pub cannons: Vec<Cannon>,
    pub portals: Vec<Portal>,
}

impl LevelInfo {
    pub fn empty() -> Self {
        Self {
            spawn_point: vec2::ZERO,
            finish_point: vec2::ZERO,
            expected_path: vec![],
            layers: vec![],
            cannons: vec![],
            portals: vec![],
        }
    }

    pub fn gameplay_surfaces(&self) -> impl Iterator<Item = &Surface> {
        self.layers
            .iter()
            .filter(|layer| layer.gameplay)
            .flat_map(|layer| &layer.surfaces)
    }

    pub fn gameplay_tiles(&self) -> impl Iterator<Item = &Tile> {
        self.layers
            .iter()
            .filter(|layer| layer.gameplay)
            .flat_map(|layer| &layer.tiles)
    }

    pub fn gameplay_objects(&self) -> impl Iterator<Item = &Object> {
        self.layers
            .iter()
            .filter(|layer| layer.gameplay)
            .flat_map(|layer| &layer.objects)
    }

    pub fn all_surfaces(&self) -> impl Iterator<Item = &Surface> {
        self.layers.iter().flat_map(|layer| &layer.surfaces)
    }

    pub fn all_tiles(&self) -> impl Iterator<Item = &Tile> {
        self.layers.iter().flat_map(|layer| &layer.tiles)
    }

    pub fn all_objects(&self) -> impl Iterator<Item = &Object> {
        self.layers.iter().flat_map(|layer| &layer.objects)
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
