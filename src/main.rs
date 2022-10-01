use geng::prelude::*;

#[derive(geng::Assets, Deserialize, Clone, Debug)]
#[asset(json)]
pub struct Config {}

#[derive(geng::Assets)]
pub struct Assets {
    pub config: Config,
}

struct Game {
    geng: Geng,
    config: Config,
    assets: Rc<Assets>,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            config: assets.config.clone(),
            assets: assets.clone(),
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = Geng::new("LD51");
    let state = geng::LoadingScreen::new(
        &geng,
        geng::EmptyLoadingScreen,
        <Assets as geng::LoadAsset>::load(&geng, &static_path()),
        {
            let geng = geng.clone();
            move |assets| {
                let assets = assets.expect("Failed to load assets");
                Game::new(&geng, &Rc::new(assets))
            }
        },
    );
    geng::run(&geng, state);
}
