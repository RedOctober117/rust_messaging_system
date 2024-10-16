use async_std::io::ReadExt;
use async_std::net::{IpAddr, TcpListener, TcpStream};
use async_std::stream::StreamExt;
use std::io::Result;

use async_std::task;
use shared::message::{Message, MessageBody, Node};
use shared::user::User;

pub struct ClientSession {
    user: User,

    listener: TcpListener,
    server_addr: (IpAddr, u16),
}

impl ClientSession {
    pub async fn new(user: User, server: (IpAddr, u16)) -> Result<Self> {
        let addr = user.location();
        Ok(Self {
            user,
            listener: TcpListener::bind(addr).await?,
            server_addr: server,
        })
    }

    pub async fn send_user(&mut self) -> shared::message::MessageResult<()> {
        let conn = TcpStream::connect(self.server_addr).await?;

        self.user
            .send_self(&conn, shared::message::Node::Server)
            .await?;

        conn.shutdown(std::net::Shutdown::Both).unwrap();
        Ok(())
    }

    pub fn self_as_node(&self) -> Node {
        Node::UserID(self.user.id())
    }

    pub async fn send_message(
        &mut self,
        message: MessageBody,
        destination: Node,
    ) -> shared::message::MessageResult<()> {
        let conn = TcpStream::connect(self.server_addr).await?;

        self.user.send_message(&conn, message, destination).await?;

        conn.shutdown(std::net::Shutdown::Both).unwrap();
        Ok(())
    }
}

pub fn request_user_info() {
    todo!()
}

pub async fn process_loop(session: ClientSession) -> shared::message::MessageResult<()> {
    let mut incoming = session.listener.incoming();

    while let Some(connection) = incoming.next().await {
        task::spawn(async move {
            let mut connection = connection.unwrap();
            let mut deser_message_str = String::new();
            connection.read_to_string(&mut deser_message_str);
            match serde_json::from_str::<Message>(&deser_message_str) {
                Ok(data) => match data.get_data() {
                    MessageBody::Text(s) => println!("got {s}",),
                    MessageBody::File(_) => println!("got file",),
                    MessageBody::User(user) => {
                        println!(" got user {:?}", user)
                    }
                    MessageBody::Failure(_) => println!("got failure",),
                },
                Err(e) => println!("Could not process connection, {}", e),
            }
            // Ok(())
        });
    }
    Ok(())
}
