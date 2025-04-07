// use shared::message::{Message, MessageBody, Node};
// use shared::user::User;
// use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Result};
// use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
// use tokio::net::TcpListener;
// use tokio::sync::mpsc::Receiver;
// use tokio::sync::{mpsc, oneshot};

// use std::collections::HashMap;
// use std::fmt::Display;
// use std::net::IpAddr;
// use std::sync::{Arc, Mutex};

// // https://draft.ryhl.io/blog/actors-with-tokio/
// // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=1ebab17a7a2f00a1fc7f37c58992d612

// pub struct TcpOrchestrator {
//     sessions: Arc<Mutex<HashMap<u16, TcpActorHandle>>>,
//     addr: (IpAddr, u16),
// }

// impl TcpOrchestrator {
//     pub fn new(addr: (IpAddr, u16)) -> Self {
//         Self {
//             sessions: Arc::new(Mutex::new(HashMap::new())),
//             addr,
//         }
//     }

//     pub fn add_session(
//         &mut self,
//         id: u16,
//         r_owned: BufReader<OwnedReadHalf>,
//         w_owned: BufWriter<OwnedWriteHalf>,
//     ) {
//         let mut session_lock = self.sessions.lock().unwrap();
//         session_lock.insert(id, TcpActorHandle::new(r_owned, w_owned));
//     }

//     pub async fn process_loop(&mut self) -> Result<()> {
//         let listener = TcpListener::bind(self.addr).await.unwrap();

//         let msg_template = Message::builder().source(Node::Server);

//         loop {
//             let (incoming_stream, _) = listener.accept().await?;

//             let (r_owned, w_owned) = incoming_stream.into_split();

//             let mut init_reader = BufReader::new(r_owned);
//             let mut init_writer = BufWriter::new(w_owned);

//             let received = init_reader.fill_buf().await?.to_vec();

//             match serde_json::from_slice::<Message>(&received) {
//                 Ok(m) => match m.body() {
//                     MessageBody::Request(shared::message::Request::Authenticate(u)) => {
//                         info!("RECEIVED: {:?}", m);
//                         if self.authenticate_user(u) {
//                             info!("AUTHENTICATED USER: {}", u.id());
//                             self.add_session(u.id(), init_reader, init_writer);

//                             let mut session_lock = self.sessions.lock().unwrap();
//                             let handle = session_lock.get_mut(&u.id()).unwrap();
//                             _ = handle
//                                 .forward_message(
//                                     msg_template
//                                         .clone()
//                                         .destination(Node::UserID(u.id()))
//                                         .body(MessageBody::Response(
//                                             shared::message::Response::AuthSuccess,
//                                         ))
//                                         .timestamp()
//                                         .unwrap()
//                                         .build(),
//                                 )
//                                 .await;
//                             tokio::spawn(async move { loop {} });
//                         } else {
//                             warn!("FAIL AUTHENTICATION: {}", u.id());
//                             let msg_payload = msg_template
//                                 .clone()
//                                 .destination(Node::UserID(u.id()))
//                                 .body(MessageBody::Response(
//                                     shared::message::Response::AuthFailure,
//                                 ))
//                                 .timestamp()
//                                 .unwrap()
//                                 .build();

//                             init_writer
//                                 .write_all(&serde_json::to_vec(&msg_payload).unwrap())
//                                 .await
//                                 .unwrap();

//                             init_writer.flush().await.unwrap();
//                         }
//                     }
//                     _ => todo!(),
//                 },
//                 Err(_) => todo!(),
//             }
//         }
//     }

//     /// If user_id is present, fail authentication
//     pub fn authenticate_user(&mut self, user: &User) -> bool {
//         match self.sessions.lock() {
//             Ok(lock) => !lock.contains_key(&user.id()),
//             Err(e) => {
//                 warn!("Error authenticating user: {e}");
//                 false
//             }
//         }
//     }
// }

// // top level run fn to accept connections -> pas

// pub struct TcpReaderActor {
//     receiver: mpsc::Receiver<TcpActorMessage>,
//     reader: BufReader<OwnedReadHalf>,
// }

// impl TcpReaderActor {
//     pub fn new(receiver: Receiver<TcpActorMessage>, reader: BufReader<OwnedReadHalf>) -> Self {
//         Self { receiver, reader }
//     }

//     pub async fn handle_message(&mut self, msg: TcpActorMessage) {
//         match msg {
//             TcpActorMessage::FillBuffer { return_addr } => {
//                 if let Ok(bytes) = self.reader.fill_buf().await {
//                     let received = bytes.to_vec();
//                     self.reader.consume(received.len());
//                     _ = return_addr.send(received);
//                 }
//             }
//             _ => todo!(),
//         }
//     }
// }

// pub struct TcpWriterActor {
//     receiver: mpsc::Receiver<TcpActorMessage>,
//     writer: BufWriter<OwnedWriteHalf>,
// }

// impl TcpWriterActor {
//     pub fn new(receiver: Receiver<TcpActorMessage>, writer: BufWriter<OwnedWriteHalf>) -> Self {
//         Self { receiver, writer }
//     }

//     pub async fn handle_message(&mut self, msg: TcpActorMessage) {
//         match msg {
//             TcpActorMessage::Forward {
//                 return_addr,
//                 message,
//             } => {
//                 match self
//                     .writer
//                     .write_all(&serde_json::to_vec(&message).unwrap())
//                     .await
//                 {
//                     Ok(_) => _ = return_addr.send(Response::Success),
//                     Err(_) => _ = return_addr.send(Response::Failure),
//                 };

//                 self.writer.flush().await.unwrap();
//             }
//             TcpActorMessage::FillBuffer { return_addr } => todo!(),
//         }
//     }
// }

// pub struct TcpActorHandle {
//     r_sender: mpsc::Sender<TcpActorMessage>,
//     w_sender: mpsc::Sender<TcpActorMessage>,
// }

// impl TcpActorHandle {
//     pub fn new(r_owned: BufReader<OwnedReadHalf>, w_owned: BufWriter<OwnedWriteHalf>) -> Self {
//         let (r_sender, r_receiver) = mpsc::channel(8);
//         let (w_sender, w_receiver) = mpsc::channel(8);

//         let mut r_actor = TcpReaderActor::new(r_receiver, r_owned);
//         let mut w_actor = TcpWriterActor::new(w_receiver, w_owned);

//         tokio::spawn(async move {
//             while let Some(msg) = r_actor.receiver.recv().await {
//                 r_actor.handle_message(msg).await;
//             }
//         });

//         tokio::spawn(async move {
//             while let Some(msg) = w_actor.receiver.recv().await {
//                 w_actor.handle_message(msg).await;
//             }
//         });

//         Self { r_sender, w_sender }
//     }

//     pub async fn forward_message(
//         &mut self,
//         msg: Message,
//     ) -> std::result::Result<Response, oneshot::error::RecvError> {
//         let (send, recv) = oneshot::channel();
//         let payload = TcpActorMessage::Forward {
//             return_addr: send,
//             message: msg,
//         };

//         let _ = self.w_sender.send(payload).await;
//         recv.await
//     }

//     pub async fn fill_buf(&mut self) -> std::result::Result<Vec<u8>, oneshot::error::RecvError> {
//         let (send, recv) = oneshot::channel();
//         let payload = TcpActorMessage::FillBuffer { return_addr: send };

//         let _ = self.r_sender.send(payload).await;
//         recv.await
//     }
// }

// pub enum TcpActorMessage {
//     Forward {
//         //                           type of data to return
//         return_addr: oneshot::Sender<Response>,
//         message: Message,
//     },

//     FillBuffer {
//         return_addr: oneshot::Sender<Vec<u8>>,
//     },
// }

// pub enum Request {}

// pub enum Response {
//     Failure,
//     Success,
// }

// #[derive(Debug, Clone)]
// pub enum AuthenticationError {
//     UserAlreadyPresent,
//     InvalidPassword,
// }

// impl std::error::Error for AuthenticationError {}

// impl Display for AuthenticationError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             AuthenticationError::InvalidPassword => write!(f, "Invalid password."),
//             AuthenticationError::UserAlreadyPresent => write!(f, "User is already present."),
//         }
//     }
// }
