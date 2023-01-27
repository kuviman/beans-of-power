use super::*;

use geng::net;

const TICKS_PER_SECOND: f32 = 1.0;

struct ClientState {
    sender: Box<dyn net::Sender<ServerMessage>>,
}

struct ServerState {
    next_client_id: Id,
    messages: Vec<ServerMessage>,
    clients: HashMap<Id, ClientState>,
}

impl ServerState {
    fn send_updates(&mut self) {
        let messages = mem::replace(&mut self.messages, Vec::new());
        for (&client_id, client) in &mut self.clients {
            for message in &messages {
                if match message {
                    ServerMessage::Pong => unreachable!(),
                    ServerMessage::ClientId(_) => unreachable!(),
                    ServerMessage::UpdateGuy(_, guy) => guy.id != client_id,
                    ServerMessage::Despawn(id) => *id != client_id,
                    ServerMessage::Emote(..) => true,
                    ServerMessage::ForceReset => true,
                } {
                    client.sender.send(message.clone());
                }
            }
        }
    }
}

struct Client {
    client_id: Id,
    server_state: Arc<Mutex<ServerState>>,
}

impl net::Receiver<ClientMessage> for Client {
    fn handle(&mut self, message: ClientMessage) {
        let mut state = self.server_state.lock().unwrap();
        let state: &mut ServerState = &mut state;
        let client = state.clients.get_mut(&self.client_id).unwrap();
        match message {
            ClientMessage::ForceReset => state.messages.push(ServerMessage::ForceReset),
            ClientMessage::Ping => client.sender.send(ServerMessage::Pong),
            ClientMessage::Update(t, guy) => state.messages.push(ServerMessage::UpdateGuy(t, guy)),
            ClientMessage::Despawn => state.messages.push(ServerMessage::Despawn(self.client_id)),
            ClientMessage::Emote(emote) => state
                .messages
                .push(ServerMessage::Emote(self.client_id, emote)),
        }
        state.send_updates();
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let mut state = self.server_state.lock().unwrap();
        let state: &mut ServerState = &mut state;
        state.messages.push(ServerMessage::Despawn(self.client_id));
        state.clients.remove(&self.client_id);
    }
}

struct ServerApp {
    state: Arc<Mutex<ServerState>>,
}

pub struct Server {
    state: Arc<Mutex<ServerState>>,
    inner: net::Server<ServerApp>,
}

impl Server {
    pub fn new<A: std::net::ToSocketAddrs + Debug + Copy>(addr: A) -> Self {
        let state = Arc::new(Mutex::new(ServerState {
            messages: Vec::new(),
            next_client_id: 0,
            clients: HashMap::new(),
        }));
        Self {
            state: state.clone(),
            inner: net::Server::new(ServerApp { state }, addr),
        }
    }
    pub fn handle(&self) -> net::ServerHandle {
        self.inner.handle()
    }
    pub fn run(self) {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let server_thread = std::thread::spawn({
            let state = self.state;
            let running = running.clone();
            let mut timer = Timer::new();
            let mut unprocessed_time = 0.0;
            move || {
                while running.load(std::sync::atomic::Ordering::Relaxed) {
                    unprocessed_time += timer.tick().as_secs_f64() as f32;
                    unprocessed_time = unprocessed_time.min(10.0 / TICKS_PER_SECOND); // Max skip 10 ticks
                    {
                        let mut state = state.lock().unwrap();
                        let state: &mut ServerState = &mut state;
                        while unprocessed_time > 1.0 / TICKS_PER_SECOND {
                            unprocessed_time -= 1.0 / TICKS_PER_SECOND;
                            // TODO: do we need it? state.tick();
                        }
                        state.send_updates();
                    }
                    std::thread::sleep(std::time::Duration::from_secs_f32(
                        1.0 / TICKS_PER_SECOND - unprocessed_time,
                    ));
                }
            }
        });
        self.inner.run();
        running.store(false, std::sync::atomic::Ordering::Relaxed);
        server_thread.join().expect("Failed to join server thread");
    }
}

impl net::server::App for ServerApp {
    type Client = Client;
    type ServerMessage = ServerMessage;
    type ClientMessage = ClientMessage;
    fn connect(&mut self, mut sender: Box<dyn net::Sender<ServerMessage>>) -> Client {
        let mut state = self.state.lock().unwrap();
        let state: &mut ServerState = &mut state;
        let client_id = state.next_client_id;
        sender.send(ServerMessage::ClientId(client_id));
        state.clients.insert(client_id, ClientState { sender });
        state.next_client_id += 1;
        Client {
            client_id,
            server_state: self.state.clone(),
        }
    }
}
