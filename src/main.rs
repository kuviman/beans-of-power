// TODO: custom fart texture/color/sound

// TODO: write the rest of this comment
use geng::prelude::*;

mod ui;

mod assets;
mod customizer;
mod editor;
mod farticle;
mod game;
mod guy;
mod leaderboard;
mod level;
mod logic;
mod net;
mod remote;
mod util;

pub use assets::*;
pub use customizer::*;
pub use editor::*;
pub use farticle::*;
pub use game::*;
pub use guy::*;
pub use leaderboard::*;
pub use level::*;
pub use logic::*;
pub use net::*;
pub use remote::*;
pub use util::*;

#[derive(clap::Parser, Clone)]
pub struct Opt {
    #[clap(long)]
    pub editor: bool,
    #[clap(long)]
    pub server: Option<String>,
    #[clap(long)]
    pub connect: Option<String>,
}

fn main() {
    geng::setup_panic_handler();
    let mut opt: Opt = program_args::parse();

    if opt.connect.is_none() && opt.server.is_none() {
        if cfg!(target_arch = "wasm32") {
            opt.connect = Some(
                option_env!("CONNECT")
                    .expect("Set CONNECT compile time env var")
                    .to_owned(),
            );
        } else {
            opt.server = Some("127.0.0.1:1155".to_owned());
            opt.connect = Some("ws://127.0.0.1:1155".to_owned());
        }
    }

    logger::init().unwrap();

    if opt.server.is_some() && opt.connect.is_none() {
        #[cfg(not(target_arch = "wasm32"))]
        net::Server::new(opt.server.as_deref().unwrap()).run();
    } else {
        #[cfg(not(target_arch = "wasm32"))]
        let server = if let Some(addr) = &opt.server {
            let server = net::Server::new(addr);
            let server_handle = server.handle();
            let server_thread = std::thread::spawn(move || {
                server.run();
            });
            Some((server_handle, server_thread))
        } else {
            None
        };

        let geng = Geng::new_with(geng::ContextOptions {
            title: "LD51 - Getting Farted On".to_owned(),
            fixed_delta_time: 1.0 / 200.0,
            vsync: false,
            ..default()
        });
        let connection = future::OptionFuture::<_>::from(match opt.connect.as_deref().unwrap() {
            "singleplayer" => None,
            addr => Some(geng::net::client::connect::<ServerMessage, ClientMessage>(
                addr,
            )),
        })
        .then(|connection| {
            future::OptionFuture::from(connection.map(|connection| async {
                let (message, mut connection) = connection.into_future().await;
                let id = match message {
                    Some(ServerMessage::ClientId(id)) => id,
                    _ => unreachable!(),
                };
                connection.send(ClientMessage::Ping);
                (id, connection)
            }))
        });
        let state = geng::LoadingScreen::new(
            &geng,
            geng::EmptyLoadingScreen,
            future::join(
                future::join(
                    <Assets as geng::LoadAsset>::load(&geng, &run_dir().join("assets")),
                    <String as geng::LoadAsset>::load(
                        &geng,
                        &run_dir().join("assets").join("level.json"),
                    ),
                ),
                connection,
            ),
            {
                let geng = geng.clone();
                move |((assets, level), connection_info)| {
                    let mut assets = assets.expect("Failed to load assets");
                    let level = Level::new(serde_json::from_str(&level.unwrap()).unwrap());
                    assets.process();
                    let assets = Rc::new(assets);
                    Game::new(&geng, &assets, level, opt, connection_info)
                }
            },
        );
        geng::run(&geng, state);

        #[cfg(not(target_arch = "wasm32"))]
        if let Some((server_handle, server_thread)) = server {
            server_handle.shutdown();
            server_thread.join().unwrap();
        }
    }
}
