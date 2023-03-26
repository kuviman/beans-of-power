use super::*;

mod draw;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Input {
    pub roll_left: f32,
    pub roll_right: f32,
    pub force_fart: bool,
}

impl Input {
    pub fn roll_direction(&self) -> f32 {
        self.roll_left - self.roll_right
    }
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

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Progress {
    pub finished: bool,
    pub current: f32,
    pub best: f32,
    pub best_time: Option<f32>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct GuyAnimationState {
    pub growl_progress: Option<f32>,
    pub next_farticle_time: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhysicsState {
    pub radius: f32,
    pub pos: vec2<f32>,
    pub vel: vec2<f32>,
    pub rot: f32,
    pub w: f32,
    pub fart_type: String,
    pub long_farting: bool,
    pub fart_pressure: f32,
    pub snow_layer: f32,
    pub cannon_timer: Option<CannonTimer>,
    pub stick_force: vec2<f32>,
    pub bubble_timer: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, HasId)]
pub struct Guy {
    pub id: Id,
    pub customization: CustomizationOptions,
    pub input: Input,
    pub state: PhysicsState,
    pub animation: GuyAnimationState,
    pub progress: Progress,
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
            state: PhysicsState {
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
                snow_layer: 0.0,
                fart_type: "normal".to_owned(), // TODO
                cannon_timer: None,
                stick_force: vec2::ZERO,
                bubble_timer: None,
                long_farting: false,
                fart_pressure: 0.0,
            },
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
        }
    }

    pub fn radius(&self) -> f32 {
        self.state.radius + self.state.snow_layer
    }

    pub fn mass(&self, config: &Config) -> f32 {
        1.0 + self.state.snow_layer * config.snow_density
    }
}

#[derive(geng::Assets)]
pub struct CustomGuyAssets {
    pub body: Texture,
    pub eyes: Texture,
    pub closed_eyes: Texture,
    pub cheeks: Texture,
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
    scale_from: f32,
    scale_to: f32,
    shake: f32,
    origin: vec2<f32>,
    go_left: f32,
    go_right: f32,
    fadein: f32,
    shake_phase: Option<f32>,
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
                fadein: 0.0,
                scale_from: 1.0,
                scale_to: 1.0,
                shake: 0.0,
                origin: vec2::ZERO,
                go_left: 0.0,
                go_right: 0.0,
                shake_phase: None,
                mode: GuyRenderLayerMode::Body,
            }];

            let mut layers = Vec::new();
            let mut t = 0;
            let mut ts = vec![0];
            let mut ignore = 0;
            let mut ignore_stack = vec![];
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
                        if xml_node.attribute("update-origin") == Some("true") {
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
                                    let bbox_center =
                                        vec2(bbox_center.x, svg_size.y - bbox_center.y); // Because svg vs inkscape coordinate system
                                    (bbox_center + inkscape_transform_center) / svg_size * 2.0
                                        - vec2(1.0, 1.0)
                                } else {
                                    vec2::ZERO
                                }
                            };
                        }
                        if let Some(fadein) = xml_node.attribute("fadein") {
                            params.fadein = fadein.parse().expect("Failed to parse fadein attr");
                        }
                        if let Some(shake) = xml_node.attribute("shake") {
                            params.shake = shake.parse().expect("Failed to parse shake attr");
                        }
                        if let Some(scale_to) = xml_node.attribute("scale-to") {
                            params.scale_to =
                                scale_to.parse().expect("Failed to parse scale-to attr");
                        }
                        if let Some(scale_from) = xml_node.attribute("scale-from") {
                            params.scale_from =
                                scale_from.parse().expect("Failed to parse scale-from attr");
                        }
                        if let Some(go_left) = xml_node.attribute("go-left") {
                            params.go_left = go_left.parse().expect("Failed to parse go-left attr");
                        }
                        if let Some(go_right) = xml_node.attribute("go-right") {
                            params.go_right =
                                go_right.parse().expect("Failed to parse go-right attr");
                        }
                        if let Some(mode) = xml_node.attribute("mode") {
                            params.mode = mode.parse().expect("Failed to parse mode");
                        }
                        if let Some(shake_phase) = xml_node.attribute("shake-phase") {
                            params.shake_phase =
                                Some(shake_phase.parse().expect("Failed to parse shake-phase"));
                        }
                        params_stack.push(params);
                        if ignore == 0 {
                            t += 1;
                            ts.push(t);
                        }
                        let ignore_value = match xml_node
                            .attribute("render-full")
                            .map(|v| v.parse().unwrap())
                        {
                            Some(true) => 1,
                            _ => 0,
                        };
                        ignore_stack.push(ignore_value);
                        ignore += ignore_value;
                    }
                    rctree::NodeEdge::End(_) => {
                        let ignore_value = ignore_stack.pop().unwrap();
                        ignore -= ignore_value;
                        if ignore == 0 {
                            if ts.pop() == Some(t) {
                                let texture = svg::render(&geng, &svg, Some(svg_node));
                                layers.push(GuyRenderLayer {
                                    texture,
                                    params: params_stack.last().unwrap().clone(),
                                });
                            }
                            t += 1;
                        }
                        params_stack.pop();
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
    pub regular: GuyRenderAssets,
    pub custom: Listed<GuyRenderAssets>,
}
