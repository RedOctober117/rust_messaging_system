use std::{
    fmt::Display,
    io::{BufReader, Read, Result, Write},
};

use crate::{
    decode::{Decode, DecodeResult},
    encode::Encode,
    login_state_data::LoginStateData,
    message_builder::MessageBuilder,
    message_data::MessageData,
    source_or_destination::SourceOrDestination,
    varuint::VarUInt,
    BoxedError, NO_STATE,
};

#[derive(Clone, Debug)]
pub struct Message {
    pub(crate) data: MessageData,
    pub(crate) timestamp: VarUInt,
    pub(crate) source: SourceOrDestination,
    pub(crate) destination: SourceOrDestination,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

    pub fn data(&self) -> &MessageData {
        &self.data
    }

    pub fn destination(&self) -> SourceOrDestination {
        self.destination.clone()
    }

    pub fn source(&self) -> SourceOrDestination {
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
    fn write_encoded(&self, _state: u8, writer: &mut impl Write) -> Result<()> {
        let mut buffer = vec![];

        self.destination
            .write_encoded(NO_STATE, &mut buffer)
            .unwrap();
        self.source.write_encoded(NO_STATE, &mut buffer).unwrap();
        self.timestamp.write_encoded(NO_STATE, &mut buffer).unwrap();
        self.data.write_encoded(NO_STATE, &mut buffer).unwrap();

        let buffer_len = VarUInt(buffer.len() as u64);

        buffer_len.write_encoded(NO_STATE, writer).unwrap();
        writer.write_all(&buffer)
    }

    fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(NO_STATE, &mut buffer).unwrap();
        Ok(buffer)
    }
}

impl Decode for Message {
    fn decode_reader(state: u8, reader: &mut impl std::io::BufRead) -> DecodeResult<Self> {
        let len = *VarUInt::decode_reader(state, reader)?;

        let take = reader.take(u64::from(len));
        let mut sized_buffer = BufReader::new(take);

        let destination: SourceOrDestination =
            *SourceOrDestination::decode_reader(state, &mut sized_buffer)?;
        let source: SourceOrDestination =
            *SourceOrDestination::decode_reader(state, &mut sized_buffer)?;
        let timestamp: VarUInt = *VarUInt::decode_reader(state, &mut sized_buffer)?;
        let data: MessageData = (*LoginStateData::decode_reader(state, &mut sized_buffer)?).into();

        Ok(Box::new(Message {
            destination,
            source,
            timestamp,
            data,
        }))
    }
}

#[cfg(test)]
mod test {

    use crate::{
        decode::Decode, encode::Encode, login_state_data::LoginStateData, message::Message,
        message_data::MessageData, source_or_destination::SourceOrDestination, NO_STATE,
    };

    #[test]
    fn encode_msg() {
        let mut buffer: Vec<u8> = vec![];
        let mut check_buffer = vec![];
        let builder = Message::builder();
        let msg = builder
            .destination(SourceOrDestination::new("user_2"))
            .source(SourceOrDestination::new("user_1"))
            .timestamp()
            .data(MessageData::LoginStateData(LoginStateData::RequestConnect))
            .build();

        msg.destination
            .write_encoded(NO_STATE, &mut check_buffer)
            .unwrap();
        msg.source
            .write_encoded(NO_STATE, &mut check_buffer)
            .unwrap();
        msg.timestamp
            .write_encoded(NO_STATE, &mut check_buffer)
            .unwrap();
        msg.data.write_encoded(NO_STATE, &mut check_buffer).unwrap();

        msg.write_encoded(NO_STATE, &mut buffer).unwrap();

        println!("{:?}", buffer);

        // this assertion only works because we know the len of the packet
        // fits in a single varuint length
        assert_eq!(buffer[1..], check_buffer);
    }

    #[test]
    fn decode_msg() {
        let mut buf: &[u8] = &[
            20, 6, 117, 115, 101, 114, 95, 50, 6, 117, 115, 101, 114, 95, 49, 204, 155, 134, 198,
            6, 0,
        ];

        let msg = *Message::decode_reader(0x00, &mut buf).unwrap();

        assert_eq!(
            msg.data().to_owned(),
            MessageData::LoginStateData(LoginStateData::RequestConnect)
        );
        assert_eq!(msg.source(), SourceOrDestination::new("user_1"));
        assert_eq!(msg.destination(), SourceOrDestination::new("user_2"));
    }
}
