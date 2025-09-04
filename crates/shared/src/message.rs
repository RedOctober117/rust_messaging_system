use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{message_body::MessageBody, message_builder::MessageBuilder, node::Node, user::User};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub(crate) body: MessageBody,
    pub(crate) timestamp: u64,
    pub(crate) source: Node,
    pub(crate) destination: Node,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

    pub fn body(&self) -> &MessageBody {
        &self.body
    }

    pub fn destination(&self) -> Node {
        self.destination.clone()
    }

    pub fn source(&self) -> Node {
        self.source.clone()
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ {{ Source: {} }}, {{ Destination: {} }}, {{ Body: {} }}, {{ Timestamp: {} }} }}",
            self.source, self.destination, self.body, self.timestamp
        )
    }
}

// impl PartialEq for Node {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Self::User(l0), Self::User(r0)) => l0 == r0,
//             _ => core::mem::discriminant(self) == core::mem::discriminant(other),
//         }
//     }
// }

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
