use std::collections::HashMap;
use std::io::Result;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use shared::message::{Message, Request, Response};
use shared::message::{MessageBody, Node};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;

pub mod server_session;

// pwsh cmd: $env:RUST_LOG="trace"
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

// life cycle: listener accepts connection ->
//             split into reader and writer ->
//             send writer to thread and store sender in top level vec ->
//             send reader to thread with ref to vec of writers

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let users: Arc<Mutex<HashMap<u16, Sender<Message>>>> = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000))
        .await
        .unwrap();

    info!("Accepting connections");
    let users_handle = Arc::clone(&users);
    let (server_upstream, mut server_downstream) = mpsc::channel::<Message>(32);

    {
        let mut users_lock = users_handle.lock().await;
        users_lock.insert(0, server_upstream);
    }

    // spawn the server task to display received messages
    let users_handle = Arc::clone(&users);
    tokio::spawn(async move {
        while let Some(msg) = server_downstream.recv().await {
            match msg.body() {
                MessageBody::Request(req) => match req {
                    Request::ReqUserID => todo!(),
                    Request::Authenticate(_) => trace!(
                        "Server received authentication request from {:?}",
                        msg.source()
                    ),
                    Request::Echo(t) => {
                        let payload = Message::builder()
                            .source(Node::Server)
                            .destination(msg.source())
                            .body(MessageBody::Response(Response::Echo(t.to_owned())))
                            .timestamp()
                            .unwrap()
                            .build();

                        warn!("Attempting echo to user {}", msg.source().unwrap());

                        let users_lock = users_handle.lock().await;
                        match users_lock.get(msg.source().unwrap()) {
                            Some(upstream) => {
                                _ = upstream.send(payload).await;
                                info!("Echoed {t:?}\n");
                            }
                            None => todo!(),
                        }
                    }
                    Request::Ping => {
                        sleep(Duration::from_secs(3)).await;
                        let time_received = msg.timestamp();

                        let payload = Message::builder()
                            .source(Node::Server)
                            .destination(msg.source())
                            .body(MessageBody::Response(Response::Ping(time_received)))
                            .timestamp()
                            .unwrap()
                            .build();

                        warn!("Attempting return ping to user {}", msg.source().unwrap());

                        let users_lock = users_handle.lock().await;
                        match users_lock.get(msg.source().unwrap()) {
                            Some(upstream) => {
                                _ = upstream.send(payload).await;
                                info!("Returned ping to user {}\n", msg.source().unwrap());
                            }
                            None => todo!(),
                        }
                    }
                },
                _ => trace!("Server received {:?}", msg),
            }
        }
    });

    let users_handle = Arc::clone(&users);
    while let Ok((conn, _)) = listener.accept().await {
        let (rx, mut tx) = conn.into_split();

        let mut buf_reader = BufReader::new(rx);
        let auth = buf_reader.fill_buf().await.unwrap().to_vec();
        buf_reader.consume(auth.len());

        let user_id = match serde_json::from_slice::<Message>(&auth).unwrap().source() {
            shared::message::Node::UserID(i) => {
                trace!("Received connection from {i}\n");
                i
            }
            shared::message::Node::Server => {
                error!("Server cannot receive connections from another server\n");
                continue;
            }
        };

        let (w_upstream, mut w_downstream) = mpsc::channel::<Message>(8);

        {
            let mut writer_lock = users_handle.lock().await;
            writer_lock.insert(user_id, w_upstream);
            trace!("Inserted user to hashmap\n");
        }

        // spawn a thread that auto sends any messages it receives from unstream to the ownedreadhalf
        tokio::spawn(async move {
            while let Some(msg) = w_downstream.recv().await {
                let id = msg.destination();

                warn!("Attempting to write {:?} to {}", msg, id.unwrap());

                tx.write_all(&serde_json::to_vec(&msg).unwrap())
                    .await
                    .unwrap();

                tx.flush().await.unwrap();
                info!("Successfully wrote message to {}\n", id.unwrap());
            }
        });

        // spawn a thread to auto forward messages to the appropriate upstreams in the hashmap
        let users_handle = Arc::clone(&users);
        tokio::spawn(async move {
            let mut received: Vec<u8>;
            loop {
                received = buf_reader.fill_buf().await.unwrap().to_vec();
                if let Ok(msg) = serde_json::from_slice::<Message>(&received) {
                    trace!("Received {:?}", msg);

                    match msg.destination() {
                        shared::message::Node::UserID(id) => {
                            let mut users_lock = users_handle.lock().await;
                            match users_lock.get_mut(&id) {
                                Some(upstream) => {
                                    warn!("Attempting to send message downstream to {id}");
                                    _ = {
                                        upstream.send(msg).await.unwrap();
                                        info!("Forwarded message to user id {id}\n");
                                    }
                                }
                                None => debug!("implement a response of 'user not found'\n"),
                            }
                        }
                        shared::message::Node::Server => {
                            let mut users_lock = users_handle.lock().await;
                            match users_lock.get_mut(&0) {
                                Some(upstream) => {
                                    warn!("Attempting to send message downstream to server");
                                    _ = {
                                        upstream.send(msg).await.unwrap();
                                        info!("Forwarded message to server\n");
                                    }
                                }
                                None => todo!(),
                            }
                        }
                    }
                }
                buf_reader.consume(received.len());
            }
        });
    }

    // info!("Server is initializing. . .");

    // let mut session = TcpOrchestrator::new((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000));

    // info!("Ready to accept connections!");
    // session.process_loop().await
    Ok(())
}
