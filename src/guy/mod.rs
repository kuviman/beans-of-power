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

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GuyRenderLayerMode {
    Body,
    ForceFart,
    Idle,
    Growl,
    Cheeks,
}

impl std::str::FromStr for GuyRenderLayerMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "body" => Self::Body,
            "force-fart" => Self::ForceFart,
            "idle" => Self::Idle,
            "growl" => Self::Growl,
            "cheeks" => Self::Cheeks,
            _ => anyhow::bail!("{s:?} is unknown mode"),
        })
    }
}

#[derive(Clone)]
pub struct GuyRenderLayerParams {
    color: Option<String>,
    scale: f32,
    shake: f32,
    origin: vec2<f32>,
    go_left: f32,
    go_right: f32,
    mode: GuyRenderLayerMode,
}

pub struct GuyRenderLayer {
    texture: ugli::Texture,
    params: GuyRenderLayerParams,
}

pub struct GuyRenderAssets {
    layers: Vec<GuyRenderLayer>,
}

impl geng::LoadAsset for GuyRenderAssets {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            use resvg::usvg::NodeExt;
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
            let svg_size = vec2(svg.size.width(), svg.size.height()).map(|x| x as f32);
            let svg_scale = vec2(
                svg.size.width() / svg.view_box.rect.width(),
                svg.size.height() / svg.view_box.rect.height(),
            )
            .map(|x| x as f32);

            let mut params_stack = vec![GuyRenderLayerParams {
                color: None,
                scale: 1.0,
                shake: 1.0,
                origin: vec2::ZERO,
                go_left: 0.0,
                go_right: 0.0,
                mode: GuyRenderLayerMode::Body,
            }];

            let mut layers = Vec::new();
            let mut t = 0;
            let mut ts = vec![0];
            for edge in svg.root.traverse() {
                let svg_node = match &edge {
                    rctree::NodeEdge::Start(node) => node,
                    rctree::NodeEdge::End(node) => node,
                };
                let Some(xml_node) = xml_nodes.get(&*svg_node.id()) else { continue };
                if !matches!(*svg_node.borrow(), resvg::usvg::NodeKind::Group(_)) {
                    continue;
                }
                match edge {
                    rctree::NodeEdge::Start(_) => {
                        let mut params = params_stack.last().unwrap().clone();
                        if let Some(color) = xml_node.attribute("color") {
                            params.color = Some(color.to_owned());
                        }
                        params.origin = {
                            let inkscape_transform_center = vec2(
                                xml_node
                                    .attribute((inkscape, "transform-center-x"))
                                    .map_or(0.0, |v| v.parse().unwrap()),
                                xml_node
                                    .attribute((inkscape, "transform-center-y"))
                                    .map_or(0.0, |v| v.parse().unwrap()),
                            );
                            let inkscape_transform_center =
                                inkscape_transform_center.map(|x| x as f32) * svg_scale;
                            if let Some(bbox) = svg_node.calculate_bbox() {
                                let bbox_center = vec2(
                                    bbox.x() as f32 + bbox.width() as f32 / 2.0,
                                    bbox.y() as f32 + bbox.height() as f32 / 2.0,
                                ) * svg_scale;
                                let bbox_center = vec2(bbox_center.x, svg_size.y - bbox_center.y); // Because svg vs inkscape coordinate system
                                (bbox_center + inkscape_transform_center) / svg_size * 2.0
                                    - vec2(1.0, 1.0)
                            } else {
                                vec2::ZERO
                            }
                        };
                        params.shake *= xml_node
                            .attribute("shake")
                            .map_or(1.0, |v| v.parse().expect("Failed to parse shake attr"));
                        params.scale *= xml_node
                            .attribute("scale")
                            .map_or(1.0, |v| v.parse().expect("Failed to parse scale attr"));
                        params.go_left += xml_node
                            .attribute("go-left")
                            .map_or(0.0, |v| v.parse().expect("Failed to parse go-left attr"));
                        params.go_right += xml_node
                            .attribute("go-right")
                            .map_or(0.0, |v| v.parse().expect("Failed to parse go-right attr"));
                        if let Some(mode) = xml_node.attribute("mode") {
                            params.mode = mode.parse().expect("Failed to parse mode");
                        }
                        params_stack.push(params);
                        t += 1;
                        ts.push(t);
                        info!("IN {t} {:?}", xml_node.attribute((inkscape, "label")));
                    }
                    rctree::NodeEdge::End(_) => {
                        info!("OUT {t} {:?}", xml_node.attribute((inkscape, "label")));
                        if ts.pop() == Some(t) {
                            info!("DRAW {:?}", xml_node.attribute((inkscape, "label")));
                            let texture = svg::render(&geng, &svg, Some(svg_node));
                            layers.push(GuyRenderLayer {
                                texture,
                                params: params_stack.last().unwrap().clone(),
                            });
                        }
                        params_stack.pop();
                        t += 1;
                    }
                }
            }
            Ok(Self { layers })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("svg");
}

#[derive(geng::Assets)]
pub struct GuyAssets {
    pub guy: GuyRenderAssets,
}
