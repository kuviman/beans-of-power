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
    #[serde(default)]
    pub reveal_radius: f32,
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

impl Default for LevelInfo {
    fn default() -> Self {
        Self {
            spawn_point: vec2::ZERO,
            finish_point: vec2::ZERO,
            expected_path: vec![],
            layers: vec![LevelLayer {
                name: "main".to_owned(),
                gameplay: true,
                surfaces: vec![],
                tiles: vec![],
                objects: vec![],
                parallax: default_parallax(),
                reveal_radius: 0.0,
            }],
            cannons: vec![],
            portals: vec![],
        }
    }
}

impl LevelInfo {
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
    path: std::path::PathBuf,
    #[deref]
    info: LevelInfo,
    mesh: RefCell<Option<draw::LevelMesh>>,
    saved: bool,
}

impl Level {
    pub async fn load(path: impl AsRef<std::path::Path>, create_if_not_exist: bool) -> Self {
        let path = path.as_ref();
        let mut saved = true;
        let info: LevelInfo = match file::load_json(path).await {
            Ok(info) => info,
            Err(e) => {
                if !path.exists() && create_if_not_exist {
                    let info: LevelInfo = default();
                    saved = false;
                    info
                } else {
                    panic!("{e}");
                }
            }
        };
        Self {
            path: path.to_owned(),
            info,
            mesh: RefCell::new(None),
            saved,
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
    pub fn save(&mut self) {
        if !mem::replace(&mut self.saved, true) {
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(std::fs::File::create(&self.path).unwrap()),
                self.info(),
            )
            .unwrap();
        }
    }
}
