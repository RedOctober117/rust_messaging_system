use shared::message::{Message, MessageBody, Node};
use tokio::io::{AsyncBufReadExt, BufReader, BufWriter, Result};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

use std::collections::HashMap;
use std::fmt::Display;
use std::net::IpAddr;

pub struct ServerSession {
    address: (IpAddr, u16),
    listener: TcpListener,
    // users: RwLock<HashMap<u16, Buffers>>,
    // router: RwLock<Router>,
}

impl ServerSession {
    pub async fn new(address: (IpAddr, u16)) -> Result<Self> {
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            address,
            listener,
            // users: RwLock::new(HashMap::new()),
            // router: RwLock::new(Router::new()),
        })
    }

    // pub fn incoming(&self) -> Incoming {
    //     self.listener.incoming()
    // }

    pub fn get_address(&self) -> (IpAddr, u16) {
        self.address
    }

    // pub fn get_users(&self) -> &HashMap<u16, User> {
    //     self.users.read()
    // }

    /// Authenticates user. If users exists in the HashMap, authentication is rejected.
    // async fn authenticate_user(self, user: User, buffers: Buffers) -> (bool, MessageBody) {
    //     let users_r_lock = self.users.read().await;

    //     if users_r_lock.contains_key(&user.id()) {
    //         return (false, MessageBody::Response(Response::AuthFailure));
    //     } else {
    //         let mut users_w_lock = self.users.blocking_write();
    //         users_w_lock.insert(user.id(), buffers);

    //         return (true, MessageBody::Response(Response::AuthSuccess));
    //     }
    // }

    pub async fn process_connections(self) -> Result<()> {
        info!("waiting. . .");

        loop {
            let (stream, _) = self.listener.accept().await?;

            // let mut user_write = self.users.write().await;
            // user_write.insert(0, stream);

            let (reader, writer) = stream.into_split();
            let mut buf_read = BufReader::new(reader);
            let mut buf_write = BufWriter::new(writer);

            tokio::spawn(async move {
                let echo_template = Message::builder().source(Node::Server);

                loop {
                    let received: Vec<u8> = buf_read.fill_buf().await.unwrap().to_vec();

                    if received.len() > 0 {
                        match serde_json::from_slice::<Message>(&received) {
                            Ok(m) => {
                                info!("RECEIVED: {:?}", m);
                                match m.get_body().to_owned() {
                                    MessageBody::Text(_) => todo!(),
                                    MessageBody::File(items) => todo!(),
                                    MessageBody::User(user) => {
                                        // let auth = self
                                        //     .authenticate_user(
                                        //         user,
                                        //         Buffers {
                                        //             read: buf_reader,
                                        //             write: buf_writer,
                                        //         },
                                        //     )
                                        //     .await;
                                    }
                                    MessageBody::Failure(_) => todo!(),
                                    MessageBody::Request(request) => todo!(),
                                    MessageBody::Response(response) => todo!(),
                                }

                                // let echo_body = match m.get_body() {
                                //     MessageBody::Text(m) => {
                                //         MessageBody::Text(format!("ECHO: {}", m))
                                //     }
                                //     _ => todo!(),
                                // };

                                // let echo_msg = echo_template
                                //     .clone()
                                //     .destination(m.get_source())
                                //     .body(echo_body)
                                //     .timestamp()
                                //     .unwrap()
                                //     .build();

                                // buf_writer
                                //     .write_all(&serde_json::to_vec(&echo_msg).unwrap())
                                //     .await
                                //     .unwrap();

                                // buf_writer.flush().await.unwrap();
                                // info!("SENT: {:?}", echo_msg);
                            }
                            Err(e) => warn!("Failed to parse message: {e}"),
                        }
                    }

                    buf_read.consume(received.len());

                    // stream
                    //     .write_all(echo_msg.as_netstring().unwrap().as_bytes())
                    //     .await
                    //     .unwrap();

                    // stream.flush().await.unwrap();
                }
            });
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthenticationError {
    UserAlreadyPresent,
    InvalidPassword,
}

impl std::error::Error for AuthenticationError {}

impl Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthenticationError::InvalidPassword => write!(f, "Invalid password."),
            AuthenticationError::UserAlreadyPresent => write!(f, "User is already present."),
        }
    }
}

// https://draft.ryhl.io/blog/actors-with-tokio/
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=1ebab17a7a2f00a1fc7f37c58992d612
pub struct TcpActorOrchestrator {
    users: HashMap<u16, TcpActorHandle>,
}

pub struct TcpActor {
    receiver: mpsc::Receiver<TcpActorMessage>,
}

pub enum TcpActorMessage {}

impl TcpActor {
    pub fn new(receiver: Receiver<TcpActorMessage>) -> Self {
        Self { receiver }
    }

    pub fn handle_message(&self, msg: TcpActorMessage) {
        match msg {}
    }
}

async fn run_actor(mut actor: TcpActor) {
    while let Some(_) = actor.receiver.recv().await {
        todo!()
    }
}

pub struct TcpActorHandle {
    sender: mpsc::Sender<TcpActorMessage>,
}

impl TcpActorHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = TcpActor::new(receiver);
        tokio::spawn(run_actor(actor));

        Self { sender }
    }
}
