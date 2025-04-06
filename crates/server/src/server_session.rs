use shared::message::{Message, MessageBody, Node};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};

use std::collections::HashMap;
use std::fmt::Display;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

pub struct ServerSession {
    address: (IpAddr, u16),
    listener: TcpListener,
    // users: RwLock<HashMap<u16, Buffers>>,
    // router: RwLock<Router>,
}

impl ServerSession {
    // pub async fn new(address: (IpAddr, u16)) -> Result<Self> {
    //     let listener = TcpListener::bind(address).await?;
    //     Ok(Self {
    //         address,
    //         listener,
    //         // users: RwLock::new(HashMap::new()),
    //         // router: RwLock::new(Router::new()),
    //     })
    // }

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

// https://draft.ryhl.io/blog/actors-with-tokio/
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=1ebab17a7a2f00a1fc7f37c58992d612
// pub struct TcpActorOrchestrator {
//     users: HashMap<u16, TcpActorHandle>,
// }

pub struct TcpOrchestrator {
    sessions: Arc<Mutex<HashMap<u16, TcpActorHandle>>>,
    addr: (IpAddr, u16),
}

impl TcpOrchestrator {
    pub fn new(addr: (IpAddr, u16)) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            addr,
        }
    }

    pub fn add_session(
        &mut self,
        id: u16,
        r_owned: BufReader<OwnedReadHalf>,
        w_owned: BufWriter<OwnedWriteHalf>,
    ) {
        let mut session_lock = self.sessions.lock().unwrap();
        session_lock.insert(id, TcpActorHandle::new(r_owned, w_owned));
    }

    pub async fn process_loop(&mut self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await.unwrap();

        loop {
            let (incoming_stream, _) = listener.accept().await?;

            let (r_owned, w_owned) = incoming_stream.into_split();

            let mut init_reader = BufReader::new(r_owned);
            let init_writer = BufWriter::new(w_owned);

            let received = init_reader.fill_buf().await?.to_vec();

            match serde_json::from_slice::<Message>(&received) {
                Ok(m) => match m.get_body() {
                    MessageBody::Request(shared::message::Request::Authenticate(u)) => {
                        warn!("AUTHENTICATED: {}", u.id());

                        self.add_session(u.id(), init_reader, init_writer);

                        let mut session_lock = self.sessions.lock().unwrap();
                        let handle = session_lock.get_mut(&u.id()).unwrap();
                        _ = handle
                            .forward_message(
                                Message::builder()
                                    .source(Node::Server)
                                    .destination(Node::UserID(u.id()))
                                    .body(MessageBody::Response(
                                        shared::message::Response::AuthSuccess,
                                    ))
                                    .timestamp()
                                    .unwrap()
                                    .build(),
                            )
                            .await;
                    }
                    _ => todo!(),
                },
                Err(_) => todo!(),
            }
        }
    }
}

pub struct TcpReaderActor {
    receiver: mpsc::Receiver<TcpActorMessage>,
    reader: BufReader<OwnedReadHalf>,
}

impl TcpReaderActor {
    pub fn new(receiver: Receiver<TcpActorMessage>, reader: BufReader<OwnedReadHalf>) -> Self {
        Self { receiver, reader }
    }

    pub async fn handle_message(&mut self, msg: TcpActorMessage) {
        match msg {
            TcpActorMessage::FillBuffer { return_addr } => {
                if let Ok(bytes) = self.reader.fill_buf().await {
                    let received = bytes.to_vec();
                    _ = return_addr.send(received);
                }
            }
            _ => todo!(),
        }
    }
}

pub struct TcpWriterActor {
    receiver: mpsc::Receiver<TcpActorMessage>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl TcpWriterActor {
    pub fn new(receiver: Receiver<TcpActorMessage>, writer: BufWriter<OwnedWriteHalf>) -> Self {
        Self { receiver, writer }
    }

    pub async fn handle_message(&mut self, msg: TcpActorMessage) {
        match msg {
            TcpActorMessage::Forward {
                return_addr,
                message,
            } => {
                match self
                    .writer
                    .write_all(&serde_json::to_vec(&message).unwrap())
                    .await
                {
                    Ok(_) => _ = return_addr.send(Response::Success),
                    Err(_) => _ = return_addr.send(Response::Failure),
                };

                self.writer.flush().await.unwrap();
            }
            TcpActorMessage::FillBuffer { return_addr } => todo!(),
        }
    }
}

pub struct TcpActorHandle {
    r_sender: mpsc::Sender<TcpActorMessage>,
    w_sender: mpsc::Sender<TcpActorMessage>,
}

impl TcpActorHandle {
    pub fn new(r_owned: BufReader<OwnedReadHalf>, w_owned: BufWriter<OwnedWriteHalf>) -> Self {
        let (r_sender, r_receiver) = mpsc::channel(8);
        let (w_sender, w_receiver) = mpsc::channel(8);

        let mut r_actor = TcpReaderActor::new(r_receiver, r_owned);
        let mut w_actor = TcpWriterActor::new(w_receiver, w_owned);

        tokio::spawn(async move {
            while let Some(msg) = r_actor.receiver.recv().await {
                r_actor.handle_message(msg).await;
            }
        });

        tokio::spawn(async move {
            while let Some(msg) = w_actor.receiver.recv().await {
                w_actor.handle_message(msg).await;
            }
        });

        Self { r_sender, w_sender }
    }

    pub async fn forward_message(
        &mut self,
        msg: Message,
    ) -> std::result::Result<Response, oneshot::error::RecvError> {
        let (send, recv) = oneshot::channel();
        let payload = TcpActorMessage::Forward {
            return_addr: send,
            message: msg,
        };

        let _ = self.w_sender.send(payload).await;
        recv.await
    }

    pub async fn fill_buf(&mut self) -> std::result::Result<Vec<u8>, oneshot::error::RecvError> {
        let (send, recv) = oneshot::channel();
        let payload = TcpActorMessage::FillBuffer { return_addr: send };

        let _ = self.r_sender.send(payload).await;
        recv.await
    }
}

pub enum TcpActorMessage {
    Forward {
        //                           type of data to return
        return_addr: oneshot::Sender<Response>,
        message: Message,
    },

    FillBuffer {
        return_addr: oneshot::Sender<Vec<u8>>,
    },
}

pub enum Request {}

pub enum Response {
    Failure,
    Success,
}

// union OwnedHalf {
//     reader: std::mem::ManuallyDrop<OwnedReadHalf>,
//     writer: std::mem::ManuallyDrop<OwnedWriteHalf>,
// }

// pub struct TcpActor {
//     receiver: mpsc::Receiver<TcpActorMessage>,
//     stream: OwnedHalf,
// }

// pub enum TcpActorMessage {
//     ForwardMessage {
//         return_addr: oneshot::Sender<Message>,
//         forwarding_message: Message,
//     },
// }

// impl TcpActor {
//     pub fn new(receiver: Receiver<TcpActorMessage>, stream: OwnedHalf) -> Self {
//         Self { receiver, stream }
//     }

//     pub fn handle_message(&mut self, msg: TcpActorMessage) {
//         match msg {
//             TcpActorMessage::ForwardMessage {
//                 return_addr,
//                 forwarding_message,
//             } => match self.forward_message(forwarding_message) {
//                 Ok(_) => todo!(),
//                 Err(_) => todo!(),
//             },
//         }
//     }

//     fn forward_message(&mut self, forwarding_msg: Message) {}
// }

// pub struct TcpActorHandle {
//     writer_sender: mpsc::Sender<TcpActorMessage>,
//     reader_sender: mpsc::Sender<TcpActorMessage>,
// }

// impl TcpActorHandle {
//     pub fn new(stream: TcpStream) -> Self {
//         let (owned_reader, owned_writer) = stream.into_split();
//         let (r_sender, r_receiver) = mpsc::channel(8);
//         let (w_sender, w_receiver) = mpsc::channel(8);

//         let reader_actor = TcpActor::new(
//             r_receiver,
//             OwnedHalf {
//                 reader: ManuallyDrop::new(owned_reader),
//             },
//         );

//         let writer_actor = TcpActor::new(
//             w_receiver,
//             OwnedHalf {
//                 writer: ManuallyDrop::new(owned_writer),
//             },
//         );

//         tokio::spawn(run_actor(reader_actor));
//         tokio::spawn(run_actor(writer_actor));

//         Self {
//             reader_sender: r_sender,
//             writer_sender: w_sender,
//         }
//         // let (sender, receiver) = mpsc::channel(8);
//         // let actor = TcpActor::new(receiver, stream);
//         // tokio::spawn(run_actor(actor));

//         // Self { sender }
//     }
// }

// async fn run_actor(mut actor: TcpActor) {
//     while let Some(msg) = actor.receiver.recv().await {
//         actor.handle_message(msg);
//     }
// }

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
