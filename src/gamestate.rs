use std::net::SocketAddr;

pub struct Player {
    pub id: u64,
    pub secret: u64,
    pub name: String,
    pub image: u64,
    pub addr: SocketAddr,
    pub x: u64,
    pub y: u64,
}
