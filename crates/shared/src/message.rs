use std::{
    fmt::Display,
    io::{Result, Write},
};

use crate::{
    encode::Encode, message_builder::MessageBuilder, message_data::MessageData, node::Node,
    varuint::VarUInt,
};

#[derive(Clone, Debug)]
pub struct Message {
    pub(crate) data: MessageData,
    pub(crate) timestamp: VarUInt,
    pub(crate) source: Node,
    pub(crate) destination: Node,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

    pub fn body(&self) -> &MessageData {
        &self.data
    }

    pub fn destination(&self) -> Node {
        self.destination.clone()
    }

    pub fn source(&self) -> Node {
        self.source.clone()
    }

    pub fn timestamp(&self) -> VarUInt {
        self.timestamp
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ {{ Source: {:?} }}, {{ Destination: {:?} }}, {{ Data: {:?} }}, {{ Timestamp: {:?} }} }}",
            self.source, self.destination, self.data, self.timestamp
        )
    }
}

impl Encode for Message {
    fn write_encoded(&self, writer: &mut impl Write) -> Result<()> {
        let mut buffer = vec![];

        self.destination.write_encoded(&mut buffer).unwrap();
        self.source.write_encoded(&mut buffer).unwrap();
        self.timestamp.write_encoded(&mut buffer).unwrap();
        self.data.write_encoded(&mut buffer).unwrap();

        let buffer_len = VarUInt(buffer.len() as u64);

        buffer_len.write_encoded(writer).unwrap();
        writer.write_all(&buffer)
    }

    fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(&mut buffer).unwrap();
        Ok(buffer)
    }
}

#[cfg(test)]
mod test {

    use crate::{
        encode::Encode,
        message::Message,
        message_data::{LoginStateData, MessageData},
        node::Node,
    };

    #[test]
    fn message() {
        let mut buffer: Vec<u8> = vec![];
        let mut check_buffer = vec![];
        let builder = Message::builder();
        let msg = builder
            .destination(Node::new("user_2"))
            .source(Node::new("user_1"))
            .timestamp()
            .body(MessageData::LoginStateData(LoginStateData::RequestConnect))
            .build();

        msg.destination.write_encoded(&mut check_buffer).unwrap();
        msg.source.write_encoded(&mut check_buffer).unwrap();
        msg.timestamp.write_encoded(&mut check_buffer).unwrap();
        msg.data.write_encoded(&mut check_buffer).unwrap();

        msg.write_encoded(&mut buffer).unwrap();

        println!("{:?}", buffer);

        // this assertion only works because we know the len of the packet
        // fits in a single varuint length
        assert_eq!(buffer[1..], check_buffer);
    }
}
