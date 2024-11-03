use gamestate::Player;
use objects::MyObject;
use simple_logger::SimpleLogger;
use tokio::sync::Mutex;

mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/game.rs"));
}

const PORT: u16 = 1026;

static PLAYERS: Mutex<Vec<Player>> = Mutex::const_new(Vec::new());
static OBJECTS: Mutex<Vec<(u64, Box<dyn MyObject>)>> = Mutex::const_new(Vec::new());

mod gamestate;
mod icon_store;
mod objects;
mod tcp_server;
mod udp_server;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    tokio::join!(tcp_server::tcp_listener(), udp_server::udp_server());
}
