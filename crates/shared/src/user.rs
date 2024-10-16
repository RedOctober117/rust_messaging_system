use std::net::IpAddr;

use async_std::{io::WriteExt, net::TcpStream};
use serde::{Deserialize, Serialize};

use crate::message::{self, Message, MessageBody, Node};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    id: u16,
    name: String,
    location: (IpAddr, u16),
}

impl User {
    pub fn new(id: u16, name: String, location: (IpAddr, u16)) -> Self {
        Self { id, name, location }
    }

    pub fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn location(&self) -> (IpAddr, u16) {
        self.location
    }

    pub async fn send_message(
        &mut self,
        connection: &TcpStream,
        data: MessageBody,
        destination: Node,
    ) -> message::MessageResult<()> {
        let message = Message::new(Node::UserID(self.id), destination, data);
        message::send_message(&connection, message).await
    }

    pub async fn send_self(
        &mut self,
        mut connection: &TcpStream,
        server: Node,
    ) -> message::MessageResult<()> {
        message::send_message(
            connection,
            Message::new(
                Node::UserID(self.id),
                server,
                MessageBody::User(self.to_owned()),
            ),
        )
        .await?;
        connection.flush().await?;

        Ok(())
    }
}
