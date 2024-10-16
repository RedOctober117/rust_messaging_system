use std::path::Display;

use async_std::{io::WriteExt, net::TcpStream};

use serde::{Deserialize, Serialize};

use crate::user::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    source: Node,
    destination: Node,
    data: MessageBody,
}

impl Message {
    pub fn new(source: Node, destination: Node, data: MessageBody) -> Self {
        Self {
            source,
            destination,
            data,
        }
    }

    pub fn get_data(&self) -> &MessageBody {
        &self.data
    }

    fn as_json(&mut self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn get_destination(&self) -> Node {
        self.destination
    }

    pub fn get_source(&self) -> Node {
        self.source
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum MessageError {
    MessageTooLong,
    SerdeError(serde_json::Error),
    IoError(std::io::Error),
}

impl std::error::Error for MessageError {}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::MessageTooLong => write!(f, "MessageError: MessageTooLong"),
            MessageError::SerdeError(error) => write!(f, "MessageError: SerdeError: {}", error),
            MessageError::IoError(error) => write!(f, "MessageError: IoError: {}", error),
        }
    }
}

impl From<serde_json::Error> for MessageError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeError(value)
    }
}
impl From<std::io::Error> for MessageError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

pub type MessageResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// uses netstring: https://en.wikipedia.org/wiki/Netstring
/// Does not establish connection automatically
pub async fn send_message(mut connection: &TcpStream, mut message: Message) -> MessageResult<()> {
    let serialized_message = message.as_json()?;
    if serialized_message.len() > 2990 {
        return Err(Box::new(MessageError::MessageTooLong));
    }

    let size_component = format!("{}:", serialized_message.len());
    let normalized_message = format!("{}{}", size_component, serialized_message);
    println!("sent {}", normalized_message);

    connection.write_all(normalized_message.as_bytes()).await?;

    connection.flush().await?;

    Ok(())
    // connection.flush().await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageBody {
    Text(String),
    File(Vec<u8>),
    User(User),
    Failure(String),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Node {
    UserID(u16),
    Server,
}
