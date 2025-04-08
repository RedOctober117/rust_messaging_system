use std::{io::Result, net::IpAddr, sync::Arc};

use shared::message::{Message, MessageBody, Node, Request, Response};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::mpsc::{self, Receiver},
};

use crate::user_map::UserMap;

pub struct ServerSession {
    user_map: Arc<UserMap>,
    listener: TcpListener,
}

impl ServerSession {
    pub async fn new(addr: (IpAddr, u16)) -> Result<Self> {
        match TcpListener::bind(addr).await {
            Ok(l) => {
                info!("Listening on {}:{}", addr.0, addr.1);
                Ok(Self {
                    user_map: UserMap::new(),
                    listener: l,
                })
            }
            Err(e) => {
                error!("Could not open server at {addr:?}, {e}");
                Err(e)
            }
        }
    }

    pub async fn start(self) {
        tokio::spawn(spawn_server_thread(Arc::clone(&self.user_map)));

        while let Ok((conn, _)) = self.listener.accept().await {
            tokio::spawn(handle_connection(Arc::clone(&self.user_map), conn));
        }
    }
}

async fn spawn_server_thread(users_handle: Arc<UserMap>) {
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

async fn spawn_writer(mut downstream: Receiver<Message>, mut tx: OwnedWriteHalf) {
    while let Some(msg) = downstream.recv().await {
        let id = msg.destination();

        warn!("Attempting to write {:?} to {}", msg, id.unwrap());

        tx.write_all(&serde_json::to_vec(&msg).unwrap())
            .await
            .unwrap();

        tx.flush().await.unwrap();
        info!("Successfully wrote message to {}\n", id.unwrap());
    }
}

async fn spawn_reader(users_handle: Arc<UserMap>, mut buf_reader: BufReader<OwnedReadHalf>) {
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
}

async fn handle_connection(users_handle: Arc<UserMap>, conn: TcpStream) {
    let (rx, tx) = conn.into_split();

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
            return;
        }
    };

    let (w_upstream, w_downstream) = mpsc::channel::<Message>(8);

    users_handle.insert(user_id, w_upstream);
    trace!("Inserted user to hashmap\n");

    // spawn a thread that auto sends any messages it receives from unstream to the ownedreadhalf
    tokio::spawn(spawn_writer(w_downstream, tx));

    // spawn a thread to auto forward messages to the appropriate upstreams in the hashmap
    tokio::spawn(spawn_reader(users_handle, buf_reader));
}
