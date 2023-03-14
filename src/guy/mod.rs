use super::*;

mod draw;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Input {
    pub roll_direction: f32, // -1 to +1
    pub force_fart: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomizationOptions {
    pub name: String,
    pub colors: GuyColors,
}

impl CustomizationOptions {
    pub fn random() -> Self {
        Self {
            name: "".to_owned(),
            colors: GuyColors::random(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Progress {
    pub finished: bool,
    pub current: f32,
    pub best: f32,
    pub best_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ball {
    pub radius: f32,
    pub pos: vec2<f32>,
    pub vel: vec2<f32>,
    pub rot: f32,
    pub w: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FartState {
    pub long_farting: bool,
    pub fart_pressure: f32,
}

impl Default for FartState {
    fn default() -> Self {
        Self {
            long_farting: false,
            fart_pressure: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuyAnimationState {
    pub growl_progress: Option<f32>,
    pub next_farticle_time: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, HasId)]
pub struct Guy {
    pub id: Id,
    pub customization: CustomizationOptions,
    pub ball: Ball,
    pub fart_state: FartState,
    pub input: Input,
    pub animation: GuyAnimationState,
    pub progress: Progress,

    pub fart_type: String,
    pub snow_layer: f32,
    pub cannon_timer: Option<CannonTimer>,
    pub stick_force: vec2<f32>,
    pub bubble_timer: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CannonTimer {
    pub cannon_index: usize,
    pub time: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuyColors {
    pub top: Rgba<f32>,
    pub bottom: Rgba<f32>,
    pub hair: Rgba<f32>,
    pub skin: Rgba<f32>,
}

impl GuyColors {
    pub fn random() -> Self {
        Self {
            top: random_hue(),
            bottom: random_hue(),
            hair: random_hue(),
            skin: {
                let tone = thread_rng().gen_range(0.5..1.0);
                Rgba::new(tone, tone, tone, 1.0)
            },
        }
    }
}

impl Guy {
    pub fn new(id: Id, pos: vec2<f32>, rng: bool, config: &Config) -> Self {
        Self {
            id,
            customization: CustomizationOptions::random(),
            ball: Ball {
                radius: config.guy_radius,
                pos: pos
                    + if rng {
                        vec2(thread_rng().gen_range(-1.0..=1.0), 0.0)
                    } else {
                        vec2::ZERO
                    },
                vel: vec2::ZERO,
                rot: if rng {
                    thread_rng().gen_range(-1.0..=1.0)
                } else {
                    0.0
                },
                w: 0.0,
            },
            snow_layer: 0.0,
            fart_state: default(),
            input: Input::default(),
            progress: Progress {
                finished: false,
                current: 0.0,
                best: 0.0,
                best_time: None,
            },
            animation: GuyAnimationState {
                growl_progress: None,
                next_farticle_time: 0.0,
            },

            fart_type: "normal".to_owned(), // TODO
            cannon_timer: None,
            stick_force: vec2::ZERO,
            bubble_timer: None,
        }
    }

    pub fn radius(&self) -> f32 {
        self.ball.radius + self.snow_layer
    }

    pub fn mass(&self, config: &Config) -> f32 {
        1.0 + self.snow_layer * config.snow_density
    }
}

#[derive(geng::Assets)]
pub struct CustomGuyAssets {
    pub body: Texture,
    pub eyes: Texture,
    pub closed_eyes: Texture,
    pub cheeks: Texture,
}

fn load_custom_guy_assets(
    geng: &Geng,
    path: &std::path::Path,
) -> geng::AssetFuture<HashMap<String, CustomGuyAssets>> {
    let geng = geng.clone();
    let path = path.to_owned();
    async move {
        let json = <String as geng::LoadAsset>::load(&geng, &path.join("_list.json")).await?;
        let list: Vec<String> = serde_json::from_str(&json).unwrap();
        future::join_all(list.into_iter().map(|name| {
            let geng = geng.clone();
            let path = path.clone();
            async move {
                let assets = geng::LoadAsset::load(&geng, &path.join(&name)).await?;
                Ok((name.to_uppercase(), assets))
            }
        }))
        .await
        .into_iter()
        .collect::<Result<_, anyhow::Error>>()
    }
    .boxed_local()
}

pub struct GuyRenderLayer {
    texture: ugli::Texture,
    color: String,
}

pub struct GuyRenderAssets {
    open_eyes: Vec<GuyRenderLayer>,
    closed_eyes: Vec<GuyRenderLayer>,
    cheeks: Vec<GuyRenderLayer>,
    body: Vec<GuyRenderLayer>,
    growl: Vec<GuyRenderLayer>,
}

impl geng::LoadAsset for GuyRenderAssets {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let raw_xml = file::load_string(path).await?;
            let raw_xml = raw_xml.replace("display:none", "");
            let xml = roxmltree::Document::parse(&raw_xml)?;
            let svg = resvg::usvg::Tree::from_xmltree(&xml, &resvg::usvg::Options::default())?;
            let xml_nodes: HashMap<&str, roxmltree::Node> = xml
                .descendants()
                .filter_map(|node| node.attribute("id").map(|id| (id, node)))
                .collect();
            let inkscape = xml
                .descendants()
                .flat_map(|node| node.namespaces())
                .find(|namespace| namespace.name() == Some("inkscape"))
                .expect("No inkscape")
                .uri();
            let make_layers = |id: &str| -> Vec<GuyRenderLayer> {
                let mut layers = Vec::new();
                for svg_node in svg
                    .node_by_id(id)
                    .unwrap_or_else(|| panic!("{id} not found"))
                    .descendants()
                {
                    let _svg_node = svg_node.borrow();
                    let id = _svg_node.id();
                    if !id.is_empty() {
                        let xml_node = xml_nodes.get(id).unwrap();
                        if let Some(color) = xml_node.attribute("color") {
                            let texture = svg::render(&geng, &svg, Some(&svg_node));
                            layers.push(GuyRenderLayer {
                                texture,
                                color: color.to_owned(),
                            });
                        }
                    }
                }
                layers
            };
            Ok(Self {
                open_eyes: make_layers("open-eyes"),
                closed_eyes: make_layers("closed-eyes"),
                cheeks: make_layers("cheeks"),
                body: make_layers("body"),
                growl: make_layers("growl"),
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("svg");
}

#[derive(geng::Assets)]
pub struct GuyAssets {
    pub guy: GuyRenderAssets,
}
