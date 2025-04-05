use shared::message::{Message, MessageBody, Node};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use tokio::net::TcpListener;

use std::net::IpAddr;
use std::ops::Deref;
use std::sync::Arc;

pub struct ServerSession {
    address: (IpAddr, u16),
    listener: TcpListener,
    // users: RwLock<HashMap<u16, User>>,
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
}

pub async fn process_connections(session: Arc<ServerSession>) -> Result<()> {
    info!("waiting. . .");

    loop {
        let (mut stream, _) = session.listener.accept().await?;

        // let session_clone = Arc::clone(&session);
        println!();
        tokio::spawn(async move {
            let echo_template = Message::builder().source(Node::Server);

            loop {
                let mut stream_buffer: Vec<u8> = Vec::new();
                stream.read_to_end(&mut stream_buffer).await.unwrap();
                // let buff_as_str = String::from_utf8(stream_buffer).unwrap();

                let deser_buff = Message::from_netstring(&stream_buffer).unwrap();
                info!("GOT: {:?}", &deser_buff);

                let echo_body = match deser_buff.get_body() {
                    MessageBody::Text(e) => MessageBody::Text(format!("ECHO: {}", e)),
                    _ => todo!(),
                };

                let echo_msg = echo_template
                    .clone()
                    .destination(deser_buff.get_source())
                    .body(echo_body)
                    .timestamp()
                    .unwrap()
                    .build();

                stream
                    .write_all(echo_msg.as_netstring().unwrap().as_bytes())
                    .await
                    .unwrap();

                stream.flush().await.unwrap();
            }
        });
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
