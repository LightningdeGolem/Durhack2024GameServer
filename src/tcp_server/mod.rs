use std::net::SocketAddr;

use log::info;
use player_manager::watch_player;
use rand::random;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::watch,
};

use crate::{
    gamestate::Player,
    icon_store,
    protobuf::{self, ConnectedPlayersUpdate, RequestToJoin},
    PLAYERS,
};

mod player_manager;

pub async fn tcp_listener() {
    let tcp_server = TcpListener::bind("0.0.0.0:1026")
        .await
        .expect("Failed to bind to TCP port");

    let (update_send, update_recv) = watch::channel(ConnectedPlayersUpdate::default());

    loop {
        let (stream, addr) = tcp_server
            .accept()
            .await
            .expect("Failed to wait for more player connections");

        tokio::task::spawn(player_tcp(
            stream,
            addr,
            update_send.clone(),
            update_recv.clone(),
        ));
    }
}

async fn player_tcp(
    stream: TcpStream,
    addr: SocketAddr,
    update_send: watch::Sender<ConnectedPlayersUpdate>,
    update_recv: watch::Receiver<ConnectedPlayersUpdate>,
) {
    info!("New connection from: {:?}", addr);

    let mut stream = prost_stream::AsyncStream::new(stream);

    let data: RequestToJoin = match stream.recv().await {
        Ok(x) => x,
        Err(e) => {
            info!("Invalid request to join sent: {e}");
            return;
        }
    };

    info!("Player `{}` is joining...", data.name);
    let id = random();
    let secret = random();

    info!("...asigning id={}...", id);

    let icon_id = icon_store::add_icon(data.icon).await;

    PLAYERS.lock().await.push(Player {
        id,
        secret,
        name: data.name.clone(),
        image: icon_id,
        addr: SocketAddr::new(addr.ip(), 0),
        x: 1,
        y: 1,
    });

    info!("...responding to join request...");

    stream
        .send(&protobuf::ResponseJoined {
            player_id: id,
            secret,
        })
        .await
        .unwrap();

    info!("...sent - {} is in the game", data.name);

    watch_player(stream.into_inner(), id, update_recv).await;

    player_manager::send_game_update(update_send).await;
}
