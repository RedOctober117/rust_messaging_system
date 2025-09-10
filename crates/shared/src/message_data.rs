use crate::{
    communication_state_data::CommunicationStateData, encode::Encode,
    login_state_data::LoginStateData,
};

#[derive(Clone, Debug)]
pub enum MessageData {
    LoginStateData(LoginStateData),
    CommunicationStateData(CommunicationStateData),
}

impl Encode for MessageData {
    fn write_encoded(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        match self {
            MessageData::LoginStateData(login_state_data) => login_state_data.write_encoded(writer),
            MessageData::CommunicationStateData(communication_state_data) => {
                communication_state_data.write_encoded(writer)
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
