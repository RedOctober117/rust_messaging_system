use tokio::{io::AsyncWriteExt, net::TcpStream};

use serde::{Deserialize, Serialize};

use crate::user::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    source: Node,
    destination: Node,
    data: MessageBody,
}
impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

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

    pub fn as_netstring(mut self) -> MessageResult<String> {
        let serialized_message = self.as_json()?;

        let normalized_message = format!("{}:{}", serialized_message.len(), serialized_message);

        Ok(normalized_message)
    }

    // uses netstring: https://en.wikipedia.org/wiki/Netstring
    // /// Does not establish connection automatically
    // pub async fn send(mut self, mut connection: TcpStream) -> MessageResult<()> {
    //     let serialized_message = self.as_json()?;
    //     if serialized_message.len() > 2990 {
    //         return Err(Box::new(MessageError::MessageTooLong));
    //     }

    //     let size_component = format!("{}:", serialized_message.len());
    //     let normalized_message = format!("{}{}", size_component, serialized_message);
    //     info!("Sent {}", normalized_message);

    //     connection.write_all(normalized_message.as_bytes()).await?;

    //     connection.flush().await?;

    //     Ok(())
    // }
}

pub struct MessageBuilder {
    source: Node,
    destination: Node,
    data: MessageBody,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            source: Node::Server,
            destination: Node::Server,
            data: MessageBody::Text("".into()),
        }
    }

    pub fn source(mut self, src: Node) -> Self {
        self.source = src;
        self
    }

    pub fn destination(mut self, dest: Node) -> Self {
        self.destination = dest;
        self
    }

    pub fn data(mut self, message_body: MessageBody) -> Self {
        self.data = message_body;
        self
    }

    pub fn build(&self) -> Message {
        Message {
            source: self.source,
            destination: self.destination,
            data: self.data.to_owned(),
        }
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self {
            source: Node::Server,
            destination: Node::Server,
            data: MessageBody::Text("".into()),
        }
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
// pub async fn send_message(mut connection: TcpStream, mut message: Message) -> MessageResult<()> {
//     let serialized_message = message.as_json()?;
//     if serialized_message.len() > 2990 {
//         return Err(Box::new(MessageError::MessageTooLong));
//     }

//     let size_component = format!("{}:", serialized_message.len());
//     let normalized_message = format!("{}{}", size_component, serialized_message);
//     info!("Sent {}", normalized_message);

//     connection.write_all(normalized_message.as_bytes()).await?;

//     connection.flush().await?;

//     Ok(())
//     // connection.flush().await
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageBody {
    Text(String),
    File(Vec<u8>),
    User(User),
    Failure(String),
    Request(Request),
    Response(Response),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request {
    ReqUserID,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    UserID(u16),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Node {
    UserID(u16),
    Server,
}
