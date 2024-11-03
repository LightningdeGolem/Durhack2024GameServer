use std::{net::SocketAddr, time::Duration};

use log::{info, trace, warn};
use prost::Message;
use tokio::{join, net::UdpSocket};

use crate::{
    protobuf::{self, server_message::PlayerLoc, ServerMessage},
    OBJECTS, PLAYERS, PORT,
};

pub async fn udp_server() {
    let udp_server = UdpSocket::bind("0.0.0.0:1026")
        .await
        .expect("Failed to bind to server port");
    join!(udp_listener(&udp_server), udp_sender(&udp_server));
}

async fn udp_listener(udp_server: &UdpSocket) {
    loop {
        let mut msg_buf = vec![0; 1024];
        let (l, addr) = udp_server.recv_from(&mut msg_buf).await.unwrap();
        trace!("Got message: {:?}", &msg_buf[..l]);

        let msg = match protobuf::ClientMessage::decode_length_delimited(msg_buf.as_slice()) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Cannot decode message: {e}");
                continue;
            }
        };
        for client in PLAYERS.lock().await.iter_mut() {
            if client.id == msg.from_id {
                if client.addr.port() == 0 {
                    client.addr.set_port(addr.port());
                }

                if client.secret == msg.secret {
                    client.x = msg.x;
                    client.y = msg.y;
                } else {
                    warn!("Invalid client secret");
                }
            }
        }
    }
}

async fn udp_sender(udp_server: &UdpSocket) {
    info!("Starting to broadcast player updates...");
    loop {
        let mut msg: Vec<_> = PLAYERS
            .lock()
            .await
            .iter()
            .map(|pl| PlayerLoc {
                id: pl.id,
                img_id: pl.image,
                x: pl.x,
                y: pl.y,
            })
            .collect();

        let msg_obj: Vec<_> = OBJECTS
            .lock()
            .await
            .iter()
            .map(|pl| PlayerLoc {
                id: pl.0,
                img_id: pl.1.img(),
                x: pl.1.x(),
                y: pl.1.y(),
            })
            .collect();

        msg.extend(msg_obj);

        let msg = ServerMessage { player_locs: msg }.encode_length_delimited_to_vec();
        for client in PLAYERS.lock().await.iter() {
            if client.addr.port() == 0 {
                continue;
            }
            udp_server.send_to(&msg, client.addr).await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
