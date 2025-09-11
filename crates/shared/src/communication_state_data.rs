use std::io::{BufRead, Write};

use thiserror::Error;

use crate::{
    decode::{Decode, DecodeResult},
    encode::Encode,
    varuint::VarUInt, NO_STATE,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

#[derive(Debug, Error)]
pub enum CommunicationStateDataError {
    #[error("failed to convert {} to CommunicationStateData", .0)]
    Conversion(VarUInt),
}

impl TryFrom<VarUInt> for CommunicationStateData {
    type Error = CommunicationStateDataError;

    fn try_from(value: VarUInt) -> std::result::Result<Self, Self::Error> {
        match value.0 {
            0x00 => Ok(Self::RequestDisconnect),
            0x01 => Ok(Self::RequestPing),
            0x02 => Ok(Self::RequestEcho(vec![])),
            0x03 => Ok(Self::CommunicationText(vec![])),
            0x04 => Ok(Self::CommunicationFile(vec![])),
            _ => Err(CommunicationStateDataError::Conversion(value)),
        }
    }
}

impl Encode for CommunicationStateData {
    fn write_encoded(&self, _state: u8, writer: &mut impl Write) -> std::io::Result<()> {
        writer.write_all(&[self.discriminant()])?;

        let additional_data: &Vec<u8> = match self {
            CommunicationStateData::RequestEcho(e) => e,
            CommunicationStateData::CommunicationFile(f) => f,
            CommunicationStateData::CommunicationText(t) => t,
            _ => &vec![],
        };

        writer.write_all(additional_data)
    }

    fn as_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(NO_STATE, &mut buffer).unwrap();
        Ok(buffer)
    }
}

impl Decode for CommunicationStateData {
    fn decode_reader(_state: u8, reader: &mut impl BufRead) -> DecodeResult<Self> {
        let code: CommunicationStateData =
            (*VarUInt::decode_reader(NO_STATE, reader)?).try_into()?;
        let mut buf = vec![];

        reader.read_to_end(&mut buf)?;

        let decoded = match code {
            Self::RequestEcho(_) => Self::RequestEcho(buf),
            Self::CommunicationText(_) => Self::CommunicationText(buf),
            Self::CommunicationFile(_) => Self::CommunicationFile(buf),
            _ => code,
        };

        Ok(Box::new(decoded))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        communication_state_data::CommunicationStateData, decode::Decode, encode::Encode, NO_STATE,
    };

    #[test]
    fn comm_state_decode() {
        let mut reader: &[u8] = &[0x00];

        assert_eq!(
            *CommunicationStateData::decode_reader(NO_STATE, &mut reader).unwrap(),
            CommunicationStateData::RequestDisconnect
        )
    }

    #[test]
    fn comm_state_decode_error() {
        let mut reader: &[u8] = &[0xFF];

        assert!(CommunicationStateData::decode_reader(NO_STATE, &mut reader).is_err(),)
    }

    #[test]
    fn comm_state_encode() {
        let mut buffer = vec![];
        let value = CommunicationStateData::RequestDisconnect;

        value
            .write_encoded(NO_STATE, &mut buffer)
            .expect("buffer error");
        assert_eq!(buffer, [0x00])
    }

    #[test]
    fn comm_state_encode_text() {
        let mut buffer = vec![];
        let text = "hello";
        let value = CommunicationStateData::CommunicationText(text.into());

        let mut check_buf = vec![];
        check_buf.push(value.discriminant());
        text.as_bytes()
            .iter()
            .for_each(|b| check_buf.push(b.to_owned()));

        value
            .write_encoded(NO_STATE, &mut buffer)
            .expect("buffer error");
        assert_eq!(buffer, check_buf)
    }

    #[test]
    fn circular_read_write() {
        let mut buffer = vec![];
        let init_val = CommunicationStateData::RequestPing;

        init_val
            .write_encoded(NO_STATE, &mut buffer)
            .expect("buffer error");

        assert_eq!(
            *CommunicationStateData::decode_reader(NO_STATE, &mut buffer.as_slice()).unwrap(),
            init_val
        )
    }
}
