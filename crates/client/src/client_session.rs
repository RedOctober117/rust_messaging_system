use async_std::io::ReadExt;
use async_std::net::{IpAddr, TcpListener, TcpStream};
use async_std::stream::StreamExt;
use std::io::Result;

use async_std::task;
use shared::message::{Message, MessageData, Node};
use shared::user::User;

pub struct ClientSession {
    user: User,
    location: (IpAddr, u16),
    listener: TcpListener,
    server: TcpStream,
}

impl ClientSession {
    pub async fn new(user: User, location: (IpAddr, u16), server: (IpAddr, u16)) -> Result<Self> {
        Ok(Self {
            user,
            location,
            listener: TcpListener::bind(location).await?,
            server: TcpStream::connect(server).await?,
        })
    }

    pub async fn send_user(&mut self) -> Result<()> {
        self.user
            .send_self(&self.server, shared::message::Node::Server)
            .await?;
        Ok(())
    }

    pub fn self_as_node(&self) -> Node {
        Node::UserID(self.user.id())
    }

    pub async fn send_message(&mut self, message: MessageData, destination: Node) -> Result<()> {
        self.user
            .send_message(&self.server, message, destination)
            .await
    }
}

pub fn request_user_info() {
    todo!()
}

pub async fn process_loop(session: ClientSession) -> Result<()> {
    let mut incoming = session.listener.incoming();

    while let Some(connection) = incoming.next().await {
        task::spawn(async move {
            let mut connection = connection.unwrap();
            let mut deser_message_str = String::new();
            connection.read_to_string(&mut deser_message_str);
            match serde_json::from_str::<Message>(&deser_message_str) {
                Ok(data) => match data.get_data() {
                    MessageData::Text(s) => println!("{} got {s}", session.user.id()),
                    MessageData::File(_) => println!("{} got file", session.user.id()),
                    MessageData::User(user) => {
                        println!("{} got user {:?}", session.user.id(), user)
                    }
                    MessageData::Failure(_) => println!("{} got failure", session.user.id()),
                },
                Err(e) => println!("{} Could not process connection, {}", session.user.id(), e),
            }
            // Ok(())
        });
    }
    Ok(())
}
