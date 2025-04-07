use std::collections::HashMap;
use std::io::Result;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use shared::message::{Message, Request, Response};
use shared::message::{MessageBody, Node};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
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

const SERVER_ADDR: (IpAddr, u16) = (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);

pub struct UserMap {
    map: Mutex<HashMap<u16, Sender<Message>>>,
}

impl UserMap {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            map: Mutex::new(HashMap::new()),
        })
    }

    pub fn insert(&self, id: u16, upstream: Sender<Message>) -> Option<Sender<Message>> {
        let mut lock = self.map.lock().unwrap();
        lock.insert(id, upstream)
    }

    pub fn get(&self, id: &u16) -> Option<Sender<Message>> {
        let lock = self.map.lock().unwrap();
        match lock.get(&id) {
            Some(upstream) => Some(upstream.clone()),
            None => None,
        }
    }

    //     pub fn get_mut(&mut self, id: &u16) -> Option<&mut Sender<Message>> {
    //         let mut lock = self.map.lock().unwrap();
    //         match lock.get_mut(id) {
    //             Some(upstream) => Some(&mut upstream.clone()),
    //             None => None,
    //         }
    //     }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    // let user_map: Arc<Mutex<HashMap<u16, Sender<Message>>>> = Arc::new(Mutex::new(HashMap::new()));
    let user_map = UserMap::new();

    let listener = TcpListener::bind(SERVER_ADDR).await.unwrap();

    info!(
        "Accepting connections on {}:{}",
        SERVER_ADDR.0, SERVER_ADDR.1
    );

    let users_handle = Arc::clone(&user_map);
    tokio::spawn(spawn_server(users_handle));

    let users_handle = Arc::clone(&user_map);
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

        users_handle.insert(user_id, w_upstream);
        trace!("Inserted user to hashmap\n");

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
        let users_handle = Arc::clone(&user_map);
        tokio::spawn(async move {
            let mut received: Vec<u8>;
            loop {
                received = buf_reader.fill_buf().await.unwrap().to_vec();
                if let Ok(msg) = serde_json::from_slice::<Message>(&received) {
                    trace!("Received {:?}", msg);

                    match msg.destination() {
                        shared::message::Node::UserID(id) => match users_handle.get(&id) {
                            Some(upstream) => {
                                warn!("Attempting to send message downstream to {id}");
                                _ = {
                                    upstream.send(msg).await.unwrap();
                                    info!("Forwarded message to user id {id}\n");
                                }
                            }
                            None => debug!("implement a response of 'user not found'\n"),
                        },
                        shared::message::Node::Server => {
                            // let mut users_lock = users_handle.lock().await;
                            match users_handle.get(&0) {
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

async fn spawn_server(users_handle: Arc<UserMap>) {
    let (server_upstream, mut server_downstream) = mpsc::channel::<Message>(32);

    users_handle.insert(0, server_upstream);

    // spawn the server task to display received messages
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

                    match users_handle.get(msg.source().unwrap()) {
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

                    match users_handle.get(msg.source().unwrap()) {
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
}
