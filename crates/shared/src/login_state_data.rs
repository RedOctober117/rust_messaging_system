use std::io::Write;

use crate::{
    decode::{Decode, DecodeResult},
    encode::Encode,
    message_data::MessageData,
    varuint::VarUInt,
    BoxedError, NO_STATE,
};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoginStateData {
    RequestConnect = 0x00,
    RequestConnectAck = 0x01,
}

impl LoginStateData {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl TryFrom<MessageData> for LoginStateData {
    type Error = &'static str;

    fn try_from(value: MessageData) -> Result<Self, Self::Error> {
        match value {
            MessageData::LoginStateData(e) => Ok(e),
            _ => Err("could not convert to LoginStateData"),
        }
    }
}

#[derive(Debug, Error)]
pub enum LoginStateDataError {
    #[error("failed to convert {} to LoginStateData", .0)]
    Conversion(VarUInt),
}

impl TryFrom<VarUInt> for LoginStateData {
    type Error = LoginStateDataError;

    fn try_from(value: VarUInt) -> Result<Self, Self::Error> {
        match value.0 {
            0x00 => Ok(Self::RequestConnect),
            0x01 => Ok(Self::RequestConnectAck),
            _ => Err(LoginStateDataError::Conversion(value)),
        }
    }
}

impl Encode for LoginStateData {
    fn write_encoded(&self, _state: u8, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(&[self.discriminant()])
    }

    fn as_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(NO_STATE, &mut buffer)?;
        Ok(buffer)
    }
}

impl Decode for LoginStateData {
    fn decode_reader(_state: u8, reader: &mut impl std::io::BufRead) -> DecodeResult<Self> {
        let value = *VarUInt::decode_reader(NO_STATE, reader)?;
        LoginStateData::try_from(value).map(|v| Ok(Box::new(v)))?
    }
}

#[cfg(test)]
mod test {
    use crate::{decode::Decode, encode::Encode, login_state_data::LoginStateData, NO_STATE};

    #[test]
    fn comm_state_decode() {
        let mut reader: &[u8] = &[0x00];

        assert_eq!(
            *LoginStateData::decode_reader(NO_STATE, &mut reader).unwrap(),
            LoginStateData::RequestConnect
        )
    }

    #[test]
    fn comm_state_encode() {
        let mut buffer = vec![];
        let value = LoginStateData::RequestConnect;

        value
            .write_encoded(NO_STATE, &mut buffer)
            .expect("buffer error");
        assert_eq!(buffer, [0x00])
    }
}
