use std::time::{SystemTime, SystemTimeError};

use serde::{Deserialize, Serialize};

use crate::user::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    body: MessageBody,
    timestamp: u64,
    source: Node,
    destination: Node,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

    pub fn body(&self) -> &MessageBody {
        &self.body
    }

    pub fn destination(&self) -> Node {
        self.destination
    }

    pub fn source(&self) -> Node {
        self.source
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

#[derive(Clone)]
pub struct MessageBuilder {
    body: MessageBody,
    timestamp: u64,
    source: Node,
    destination: Node,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            source: Node::Server,
            destination: Node::Server,
            timestamp: 0,
            body: MessageBody::Text("".into()),
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

    pub fn body(mut self, message_body: MessageBody) -> Self {
        self.body = message_body;
        self
    }

    pub fn now() -> Result<u64, SystemTimeError> {
        Ok(SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs())
    }

    pub fn timestamp(mut self) -> Result<Self, SystemTimeError> {
        self.timestamp = Self::now()?;
        Ok(self)
    }

    pub fn build(&self) -> Message {
        Message {
            source: self.source,
            destination: self.destination,
            timestamp: self.timestamp,
            body: self.body.to_owned(),
        }
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self {
            source: Node::Server,
            destination: Node::Server,
            timestamp: 0,
            body: MessageBody::Text("".into()),
        }
    }
}

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
    Authenticate(User),
    Echo(Vec<u8>),
    Ping,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    UserID(u16),
    AuthSuccess,
    AuthFailure,
    Echo(Vec<u8>),
    Ping(u64),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Node {
    Server,
    UserID(u16),
}

impl Node {
    pub fn unwrap(&self) -> &u16 {
        match self {
            Node::Server => &0,
            Node::UserID(id) => id,
        }
    }
}

// pub fn as_netstring(mut self) -> Result<String, serde_json::Error> {
//     let serialized_message = self.as_json()?;

//     let normalized_message = format!("{}:{}", serialized_message.len(), serialized_message);

//     Ok(normalized_message)
// }

// pub fn from_netstring(netstr: &[u8]) -> Result<Message, serde_json::Error> {
//     serde_json::from_slice::<Message>(&netstr)
// }

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
