use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Input {
    pub roll_direction: f32, // -1 to +1
    pub force_fart: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, HasId)]
pub struct Guy {
    pub name: String,
    pub colliding_water: bool,
    pub id: Id,
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub rot: f32,
    pub w: f32,
    pub input: Input,
    pub auto_fart_timer: f32,
    pub force_fart_timer: f32,
    pub finished: bool,
    pub colors: GuyColors,
    pub postjam: bool,
    pub progress: f32,
    pub best_progress: f32,
    pub best_time: Option<f32>,
    pub touched_a_unicorn: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuyColors {
    pub top: Rgba<f32>,
    pub bottom: Rgba<f32>,
    pub hair: Rgba<f32>,
    pub skin: Rgba<f32>,
}

impl Guy {
    pub fn new(id: Id, pos: Vec2<f32>, rng: bool) -> Self {
        let random_hue = || {
            let hue = global_rng().gen_range(0.0..1.0);
            Hsva::new(hue, 1.0, 1.0, 1.0).into()
        };
        Self {
            colliding_water: false,
            name: "".to_owned(),
            id,
            pos: pos
                + if rng {
                    vec2(global_rng().gen_range(-1.0..=1.0), 0.0)
                } else {
                    Vec2::ZERO
                },
            vel: Vec2::ZERO,
            rot: if rng {
                global_rng().gen_range(-1.0..=1.0)
            } else {
                0.0
            },
            w: 0.0,
            input: Input::default(),
            auto_fart_timer: 0.0,
            force_fart_timer: 0.0,
            finished: false,
            colors: GuyColors {
                top: random_hue(),
                bottom: random_hue(),
                hair: random_hue(),
                skin: {
                    let tone = global_rng().gen_range(0.5..1.0);
                    Rgba::new(tone, tone, tone, 1.0)
                },
            },
            postjam: false,
            progress: 0.0,
            best_progress: 0.0,
            best_time: None,
            touched_a_unicorn: false,
        }
    }
}

#[derive(geng::Assets)]
pub struct CustomGuyAssets {
    pub body: Texture,
    pub eyes: Texture,
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

#[derive(geng::Assets)]
pub struct GuyAssets {
    pub cheeks: Texture,
    pub eyes: Texture,
    pub skin: Texture,
    pub clothes_top: Texture,
    pub clothes_bottom: Texture,
    pub hair: Texture,
    #[asset(load_with = "load_custom_guy_assets(&geng, &base_path.join(\"custom\"))")]
    pub custom: HashMap<String, CustomGuyAssets>,
}

impl Game {
    pub fn draw_guys(&self, framebuffer: &mut ugli::Framebuffer) {
        for guy in itertools::chain![
            self.guys.iter().filter(|guy| guy.id != self.client_id),
            self.guys.iter().filter(|guy| guy.id == self.client_id),
        ] {
            let (eyes, cheeks, cheeks_color) = if let Some(custom) =
                self.assets.guy.custom.get(&guy.name)
            {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&custom.body)
                        .scale_uniform(self.config.guy_radius)
                        .transform(Mat3::rotate(guy.rot))
                        .translate(guy.pos),
                );
                (&custom.eyes, &custom.cheeks, Rgba::WHITE)
            } else {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(
                        &self.assets.guy.clothes_bottom,
                        guy.colors.bottom,
                    )
                    .scale_uniform(self.config.guy_radius)
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(
                        &self.assets.guy.clothes_top,
                        guy.colors.top,
                    )
                    .scale_uniform(self.config.guy_radius)
                    .transform(Mat3::rotate(guy.rot))
                    .translate(guy.pos),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(&self.assets.guy.hair, guy.colors.hair)
                        .scale_uniform(self.config.guy_radius)
                        .transform(Mat3::rotate(guy.rot))
                        .translate(guy.pos),
                );
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit_colored(&self.assets.guy.skin, guy.colors.skin)
                        .scale_uniform(self.config.guy_radius)
                        .transform(Mat3::rotate(guy.rot))
                        .translate(guy.pos),
                );
                (
                    &self.assets.guy.eyes,
                    &self.assets.guy.cheeks,
                    guy.colors.skin,
                )
            };
            let autofart_progress = guy.auto_fart_timer / self.config.auto_fart_interval;
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(eyes, {
                    let k = 0.8;
                    let t = ((autofart_progress - k) / (1.0 - k)).clamp(0.0, 1.0) * 0.5;
                    Rgba::new(1.0, 1.0 - t, 1.0 - t, 1.0)
                })
                .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * autofart_progress)
                .scale_uniform(self.config.guy_radius * (0.8 + 0.6 * autofart_progress))
                .transform(Mat3::rotate(guy.rot))
                .translate(guy.pos),
            );
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::TexturedQuad::unit_colored(
                    cheeks,
                    Rgba {
                        a: (0.5 + 1.0 * autofart_progress).min(1.0),
                        ..cheeks_color
                    },
                )
                .translate(vec2(self.noise(10.0), self.noise(10.0)) * 0.1 * autofart_progress)
                .scale_uniform(self.config.guy_radius * (0.8 + 0.7 * autofart_progress))
                .transform(Mat3::rotate(guy.rot))
                .translate(guy.pos),
            );
            if Some(guy.id) == self.my_guy || self.show_names {
                self.assets.font.draw(
                    framebuffer,
                    &self.camera,
                    &guy.name,
                    guy.pos + vec2(0.0, self.config.guy_radius * 1.1),
                    geng::TextAlign::CENTER,
                    0.1,
                    if guy.postjam {
                        Rgba::BLACK
                    } else {
                        Rgba::new(0.0, 0.0, 0.0, 0.5)
                    },
                );
            }
        }
        for &(_, id, emote) in &self.emotes {
            if let Some(guy) = self.guys.get(&id) {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&self.assets.emotes[emote])
                        .scale_uniform(0.1)
                        .translate(guy.pos + vec2(0.0, self.config.guy_radius * 2.0)),
                );
            }
        }
    }
}
