// use std::io::Result;
// use std::net::IpAddr;
// use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest};
// use tokio::net::TcpStream;

// use shared::message::{Message, MessageBody, MessageResult, Node};
// use shared::user::User;

// pub struct ClientSession {
//     user: User,

//     stream: TcpStream,
//     server_addr: (IpAddr, u16),
// }

// impl ClientSession {
//     pub async fn new(user: User, server: (IpAddr, u16)) -> Result<Self> {
//         let addr = user.location();
//         Ok(Self {
//             user,
//             stream: TcpStream::connect(addr).await?,
//             server_addr: server,
//         })
//     }

//     pub async fn send_user(&mut self) -> MessageResult<()> {
//         let register_msg = Message::builder()
//             .source(self.self_as_node())
//             .destination(Node::Server)
//             .data(MessageBody::User(self.user.to_owned()))
//             .build();

//         self.send_message(register_msg).await?;

//         Ok(())
//     }

//     pub fn self_as_node(&self) -> Node {
//         Node::UserID(self.user.id())
//     }

//     pub async fn send_message(&mut self, message: Message) -> MessageResult<()> {
//         self.stream
//             .write_all(message.as_netstring()?.as_bytes())
//             .await?;
//         self.stream.flush();

//         Ok(())
//     }

//     pub async fn process_loop(mut self) -> MessageResult<()> {
//         loop {
//             let is_ready = self.stream.ready(Interest::READABLE).await?.is_readable();

//             if is_ready {
//                 let mut deser_message_str = String::new();
//                 self.stream.read_to_string(&mut deser_message_str);

//                 match serde_json::from_str::<Message>(&deser_message_str) {
//                     Ok(data) => match data.get_data() {
//                         MessageBody::Text(s) => info!("Received {s}",),
//                         MessageBody::File(_) => info!("got file",),
//                         MessageBody::User(user) => {
//                             info!(" got user {:?}", user)
//                         }
//                         MessageBody::Failure(_) => warn!("got failure",),
//                         MessageBody::Request(_) => todo!(),
//                         MessageBody::Response(_) => todo!(),
//                     },
//                     Err(e) => error!("Error processing stream data: {e}"),
//                 }
//             }
//         }
//     }
// }

// pub fn request_user_info() {
//     todo!()
// }
