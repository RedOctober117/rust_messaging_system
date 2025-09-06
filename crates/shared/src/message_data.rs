use std::io::{Result, Write};

use crate::{decode::Decode, encode::Encode};

#[derive(Clone, Debug)]
pub enum MessageData {
    LoginStateData(LoginStateData),
    CommunicationStateData(CommunicationStateData),
}

impl Encode for MessageData {
    fn write_encoded(&self, writer: &mut impl Write) -> Result<()> {
        match self {
            MessageData::CommunicationStateData(c) => c.write_encoded(writer),
            MessageData::LoginStateData(l) => l.write_encoded(writer),
        }
    }

    fn as_bytes(&self) -> Result<Vec<u8>> {
        match self {
            MessageData::CommunicationStateData(c) => {
                let mut buffer = vec![];
                c.write_encoded(&mut buffer)?;
                Ok(buffer)
            }
            MessageData::LoginStateData(l) => {
                let mut buffer = vec![];
                l.write_encoded(&mut buffer)?;
                Ok(buffer)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum LoginStateData {
    RequestConnect = 0x00,
    RequestConnectAck = 0x01,
}
impl LoginStateData {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl Encode for LoginStateData {
    fn write_encoded(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(&[self.discriminant()])
    }

    fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(&mut buffer)?;
        Ok(buffer)
    }
}

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum CommunicationStateData {
    RequestDisconnect = 0x00,
    RequestPing = 0x01,
    RequestEcho(Vec<u8>) = 0x02,
    CommunicationText(Vec<u8>) = 0x03,
    CommunicationFile(Vec<u8>) = 0x04,
}

impl CommunicationStateData {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl Encode for CommunicationStateData {
    fn write_encoded(&self, writer: &mut impl Write) -> Result<()> {
        writer
            .write_all(&[self.discriminant()])
            .map_err(|e| return e)?;

        let additional_data: &Vec<u8> = match self {
            CommunicationStateData::RequestEcho(e) => e,
            CommunicationStateData::CommunicationFile(f) => f,
            CommunicationStateData::CommunicationText(t) => t,
            _ => &vec![],
        };

        writer.write_all(additional_data)
    }

    fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(&mut buffer).unwrap();
        Ok(buffer)
    }
}

// account for state
impl Decode for CommunicationStateData {
    fn decode_reader(reader: &mut impl std::io::Read) -> Result<Box<Self>> {
        todo!()
    }
}
