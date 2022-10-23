use super::*;

#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(not(target_arch = "wasm32"))]
pub use server::Server;

pub type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    Update(f32, Guy),
    Despawn,
    Emote(usize),
    ForceReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Pong,
    ForceReset,
    ClientId(Id),
    UpdateGuy(f32, Guy),
    Despawn(Id),
    Emote(Id, usize),
}
