use crate::{
    communication_state_data::CommunicationStateData,
    decode::{Decode, DecodeResult},
    encode::Encode,
    login_state_data::LoginStateData,
    BoxedError, NO_STATE,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageData {
    LoginStateData(LoginStateData),
    CommunicationStateData(CommunicationStateData),
}

impl From<LoginStateData> for MessageData {
    fn from(value: LoginStateData) -> Self {
        match value {
            LoginStateData::RequestConnect => Self::LoginStateData(LoginStateData::RequestConnect),
            LoginStateData::RequestConnectAck => {
                Self::LoginStateData(LoginStateData::RequestConnectAck)
            }
        }
    }
}

impl Encode for MessageData {
    fn write_encoded(&self, _state: u8, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            MessageData::LoginStateData(login_state_data) => {
                login_state_data.write_encoded(NO_STATE, writer)
            }
            MessageData::CommunicationStateData(communication_state_data) => {
                communication_state_data.write_encoded(NO_STATE, writer)
            }
        }
    }

    fn as_bytes(&self) -> std::io::Result<Vec<u8>> {
        match self {
            MessageData::LoginStateData(login_state_data) => login_state_data.as_bytes(),
            MessageData::CommunicationStateData(communication_state_data) => {
                communication_state_data.as_bytes()
            }
        }
    }
}

impl Decode for MessageData {
    fn decode_reader(state: u8, reader: &mut impl std::io::BufRead) -> DecodeResult<Self> {
        match state {
            0x00 => Ok(Box::new(MessageData::LoginStateData(
                *LoginStateData::decode_reader(state, reader)?,
            ))),
            0x01 => Ok(Box::new(MessageData::CommunicationStateData(
                *CommunicationStateData::decode_reader(state, reader)?,
            ))),
            _ => Err("invalid state passed".into()),
        }
    }
}
