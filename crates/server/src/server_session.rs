use futures::io::FillBuf;
use futures::TryFutureExt;
use shared::message::{Message, MessageBody, Node, Response};
use shared::user::User;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

use std::collections::HashMap;
use std::fmt::Display;
use std::net::IpAddr;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

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

//     let mut stream = tokio::io::BufReader::new(stream);
//     let mut stream_buffer: Vec<u8> = vec![];

//     stream.read_to_end(&mut stream_buffer).await.unwrap();
//     let incoming_message = String::from_utf8(stream_buffer.into()).unwrap();

//     let incoming_mess_len_raw = incoming_message.split(':').nth(0).unwrap();
//     let incoming_message_len: u16 = incoming_mess_len_raw.parse().unwrap();

//     let offset = incoming_mess_len_raw.len() + 1;
//     // add 3 to offset for the xx: part of the stream_buffer
//     let incoming_message_data = String::from_utf8(
//         incoming_message[offset..(incoming_message_len + offset as u16).into()].into(),
//     )
//     .unwrap();

//     let deserialized_data = serde_json::from_str::<Message>(&incoming_message_data);

//     // println!(
//     //     "raw data: {:?}\n data component: {}",
//     //     incoming_message, incoming_message_data
//     // );

//     match deserialized_data {
//         Ok(message) => match message.get_destination() {
//             Node::UserID(id) => {
//                 let session = Arc::clone(&session_clone);
//                 let mut writer = session.router.blocking_write();
//                 writer.deref_mut().send_message(message).await.unwrap();
//             }
//             Node::Server => {
//                 info!("Server received: {:?}", message);
//                 match message.get_data() {
//                     message::MessageBody::Text(s) => {
//                         info!("server received message: {}", s)
//                     }
//                     message::MessageBody::File(vec) => todo!(),
//                     message::MessageBody::User(user) => {
//                         let session = Arc::clone(&session_clone);
//                         let mut writer = session.router.blocking_write();
//                         writer.deref_mut().route_user(user.id(), user.location());
//                     }
//                     message::MessageBody::Failure(_) => todo!(),
//                     message::MessageBody::Request(request) => match request {
//                         message::Request::ReqUserID => {
//                             let id_length = session_clone.users.read().await.len();
//                             // message::send_message(
//                             //     &stream.into_inner(),
//                             //     Message::new(
//                             //         Node::Server,
//                             //         message.get_source(),
//                             //         message::MessageBody::Response(
//                             //             message::Response::UserID((id_length + 1) as u16),
//                             //         ),
//                             //     ),
//                             // )
//                             // .await
//                             // .unwrap();
//                         }
//                     },
//                     message::MessageBody::Response(response) => todo!(),
//                 }
//             }
//         },
//         // ::Body(v) => match v.get_destination() {
//         //     Node::UserID(id) => println!("destination: {}", id),
//         //     Node::Server => println!("destination: server"),
//         //     // Node::Server => match v.get_data() {
//         //     //     MessageData::User(user) => {
//         //     //         _ = session
//         //     //             .users
//         //     //             .write()
//         //     //             .await
//         //     //             .insert(user.id(), user.to_owned());
//         //     //         session.router.write().await.route_user(user.id(), stream);
//         //     //         println!("inserted user {}", user.id())
//         //     //     }
//         Err(e) => error!(
//             "Could not process data '{:?}': {}",
//             incoming_message_data, e
//         ),
//     };

// struct Router {
//     addresses: HashMap<u16, (IpAddr, u16)>,
// }

// impl Router {
//     pub fn new() -> Self {
//         Self {
//             addresses: HashMap::new(),
//         }
//     }
//     pub fn route_user(&mut self, id: u16, loc: (IpAddr, u16)) {
//         _ = self.addresses.insert(id, loc)
//     }

//     fn get_location(&mut self, id: &u16) -> Option<(IpAddr, u16)> {
//         self.addresses.get(id).copied()
//     }

//     pub async fn send_message(&mut self, message: Message) -> shared::message::MessageResult<()> {
//         if let Node::UserID(i) = &message.get_destination() {
//             match self.get_location(i) {
//                 Some(i) => {
//                     let conn = TcpStream::connect(i).await?;
//                     // message::send_message(&conn, message).await?;
//                 }
//                 None => warn!("wanted to send to user {}", i),
//             }
//         }

//         Ok(())
//     }
// }
