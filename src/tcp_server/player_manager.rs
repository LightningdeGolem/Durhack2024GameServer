use log::{error, info};
use tokio::{net::TcpStream, sync::watch};

use crate::{
    icon_store::ICON_STORE,
    protobuf::{
        self, connected_players_update::ConnectedPlayer, server_message::PlayerLoc,
        ConnectedPlayersUpdate,
    },
    OBJECTS, PLAYERS,
};

async fn player_watcher(
    stream: TcpStream,
    id: u64,
    mut broadcast_state: watch::Receiver<ConnectedPlayersUpdate>,
) {
    let mut stream = prost_stream::AsyncStream::new(stream);
    info!("Watching player {id}");
    loop {
        tokio::select! {
            x = stream.recv::<PlayerLoc>() => {
                match x {
                    Ok(_) => (),
                    Err(prost_stream::Error::IoError(e)) => {
                        info!("Player {id} TCP stream err ({e}), assuming disconnected");
                        PLAYERS.lock().await.retain(|x| x.id != id);
                        break;
                    }
                    Err(_) => (),
                }
            }

            d = broadcast_state.changed() => {
                match d {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Player update watch channel broke: {e}");
                        break;
                    }
                }

                let d = broadcast_state.borrow().clone();
                match stream.send(&d).await {
                    Ok(_) => (),
                    Err(prost_stream::Error::IoError(e)) => {
                        info!("Player {id} TCP stream err ({e}), assuming disconnected");
                        PLAYERS.lock().await.retain(|x| x.id != id);
                        break;
                    }
                    Err(_) => (),
                }
            }


        }
    }
}

pub async fn send_game_update(channel: watch::Sender<ConnectedPlayersUpdate>) {
    info!("Sending player update!");
    let mut players: Vec<_> = PLAYERS
        .lock()
        .await
        .iter()
        .map(|pl| ConnectedPlayer {
            id: pl.id,
            name: pl.name.clone(),
            icon: pl.image.clone(),
        })
        .collect();

    let objects: Vec<_> = OBJECTS
        .lock()
        .await
        .iter()
        .map(|pl| ConnectedPlayer {
            id: pl.0,
            name: String::new(),
            icon: pl.1.img().to_owned(),
        })
        .collect();

    players.extend(objects);

    let update = protobuf::ConnectedPlayersUpdate {
        players,
        icons: ICON_STORE.lock().await.clone(),
    };
    channel.send(update).expect("Failed to send message");
}

pub async fn watch_player(
    stream: TcpStream,
    id: u64,
    updates: watch::Receiver<ConnectedPlayersUpdate>,
) {
    tokio::task::spawn(player_watcher(stream, id, updates));
}
