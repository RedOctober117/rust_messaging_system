use std::io::{Read, Write};

// use serde::{Deserialize, Serialize};

use crate::{
    decode::{Decode, DecodeResult},
    encode::Encode,
    varuint::VarUInt,
    NO_STATE,
};

pub const SERVER_NODE: [u8; 2] = [1, 0];

#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceOrDestination(Vec<u8>);

impl SourceOrDestination {
    pub fn new(name: impl Into<Vec<u8>>) -> Self {
        Self(name.into())
    }
}

impl Encode for SourceOrDestination {
    fn write_encoded(&self, _state: u8, writer: &mut impl Write) -> std::io::Result<()> {
        VarUInt(self.0.len() as u64).write_encoded(NO_STATE, writer)?;
        writer.write_all(&self.0)
    }

    fn as_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(NO_STATE, &mut buffer).unwrap();
        Ok(buffer)
    }
}

impl Decode for SourceOrDestination {
    fn decode_reader(_state: u8, reader: &mut impl std::io::BufRead) -> DecodeResult<Self> {
        let length = *VarUInt::decode_reader(NO_STATE, reader)?;

        let mut take = reader.take(length.into());
        let mut buf: Vec<u8> = vec![];

        take.read_to_end(&mut buf)?;

        Ok(Box::new(SourceOrDestination(buf)))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        boxed_error::BoxedError, decode::Decode, encode::Encode,
        source_or_destination::SourceOrDestination, varuint::VarUInt, NO_STATE,
    };

    #[test]
    fn user_encoding() -> Result<(), BoxedError> {
        let user: &[u8] = "TestUser".as_ref();
        let user_as_src = SourceOrDestination::new(user);
        let mut buf = vec![];

        user_as_src.write_encoded(NO_STATE, &mut buf).unwrap();

        let mut check_buf = vec![];
        VarUInt::try_from(user.len())?.write_encoded(NO_STATE, &mut check_buf)?;
        check_buf.append(&mut Vec::from(user));

        assert_eq!(buf, check_buf);
        Ok(())
    }

    #[test]
    fn user_decoding() {
        let mut raw_bytes: &[u8] = &[8, 84, 101, 115, 116, 85, 115, 101, 114];

        assert_eq!(
            *SourceOrDestination::decode_reader(NO_STATE, &mut raw_bytes).unwrap(),
            SourceOrDestination::new("TestUser")
        )
    }
}
