use std::io::{Read, Result, Write};

use crate::{decode::Decode, encode::Encode};

pub type VarUIntEnclosedType = u64;

const SEGMENT_BITS: u8 = 0x7F;
const LEADING_BIT: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarUInt(pub VarUIntEnclosedType);

impl VarUInt {
    // https://en.wikipedia.org/wiki/LEB128
    pub fn decode(reader: &mut impl Read) -> Self {
        let mut result: VarUIntEnclosedType = 0;
        let mut shift: u8 = 0;

        for byte in reader.bytes() {
            let byte = byte.unwrap();
            result |= (byte as VarUIntEnclosedType & SEGMENT_BITS as VarUIntEnclosedType) << shift;
            shift += 7;

            if byte & LEADING_BIT == 0 {
                break;
            }
        }

        Self(result)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = vec![];
        let mut val = self.0;

        loop {
            let mut byte = val & SEGMENT_BITS as VarUIntEnclosedType;
            val >>= 7;

            if val != 0 {
                byte |= LEADING_BIT as VarUIntEnclosedType;
            }
            result.push(byte as u8);

            if val == 0 {
                break;
            }
        }
        result
    }
}

impl From<VarUInt> for u64 {
    fn from(value: VarUInt) -> Self {
        value.0
    }
}

impl From<VarUInt> for usize {
    fn from(value: VarUInt) -> Self {
        value.0 as usize
    }
}

impl Encode for VarUInt {
    fn write_encoded(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(&self.encode())
    }

    fn as_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        self.write_encoded(&mut buffer).unwrap();
        Ok(buffer)
    }
}

impl Decode for VarUInt {
    fn decode_reader(reader: &mut impl std::io::Read) -> Result<Box<Self>> {
        let mut result: VarUIntEnclosedType = 0;
        let mut shift: u8 = 0;

        for byte in reader.bytes() {
            let byte = byte.unwrap();
            result |= (byte as VarUIntEnclosedType & SEGMENT_BITS as VarUIntEnclosedType) << shift;
            shift += 7;

            if byte & LEADING_BIT == 0 {
                break;
            }
        }

        Ok(Box::new(Self(result)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{message_builder::MessageBuilder, varuint::*};

    #[test]
    fn decode() {
        assert_eq!(
            VarUInt::decode(&mut vec![132_u8, 6_u8].as_slice()),
            VarUInt(772)
        );
        assert_eq!(
            VarUInt::decode(&mut vec![221, 199, 1].as_slice()),
            VarUInt(25565)
        );
    }

    #[test]
    fn decode_with_extra() {
        assert_eq!(
            VarUInt::decode(&mut vec![132, 6, 7].as_slice()),
            VarUInt(772)
        );
        assert_eq!(
            VarUInt::decode(&mut vec![221, 199, 1, 254].as_slice()),
            VarUInt(25565)
        );
    }

    #[test]
    fn encode() {
        assert_eq!(VarUInt(772).encode(), [132, 6]);
        assert_eq!(VarUInt(25565).encode(), [221, 199, 1]);
        assert_eq!(VarUInt(2147483647).encode(), [255, 255, 255, 255, 7]);
    }

    #[test]
    fn encode_buffer() {
        let mut buffer = vec![];

        let test_int = VarUInt(1);
        test_int.write_encoded(&mut buffer).unwrap();

        assert_eq!(buffer, [1]);

        buffer.clear();
        VarUInt(25565).write_encoded(&mut buffer).unwrap();
        assert_eq!(buffer, [221, 199, 1]);

        buffer.clear();
        VarUInt(25565).write_encoded(&mut buffer).unwrap();
        assert_eq!(buffer, [221, 199, 1]);

        buffer.clear();
        VarUInt(2147483647).write_encoded(&mut buffer).unwrap();
        assert_eq!(buffer, [255, 255, 255, 255, 7]);
    }

    #[test]
    fn decode_buffer() {
        assert_eq!(*VarUInt::decode_reader(&mut &[1][..]).unwrap(), VarUInt(1));

        assert_eq!(
            *VarUInt::decode_reader(&mut &[221, 199, 1][..]).unwrap(),
            VarUInt(25565)
        );

        assert_eq!(
            *VarUInt::decode_reader(&mut &[255, 255, 255, 255, 7][..]).unwrap(),
            VarUInt(2147483647)
        );
    }

    #[test]
    fn decode_buffer_with_extra() {
        let mut buf: &[u8] = &[1, 2];

        assert_eq!(*VarUInt::decode_reader(&mut buf).unwrap(), VarUInt(1));
        assert_eq!(buf.len(), 1);
        assert_eq!(buf, [2]);

        buf = &[221, 199, 1, 9, 6];

        assert_eq!(*VarUInt::decode_reader(&mut buf).unwrap(), VarUInt(25565));
        assert_eq!(buf.len(), 2);
        assert_eq!(buf, [9, 6]);

        buf = &[255, 255, 255, 255, 7, 254, 1];

        assert_eq!(
            *VarUInt::decode_reader(&mut buf).unwrap(),
            VarUInt(2147483647)
        );
        assert_eq!(buf.len(), 2);
        assert_eq!(buf, [254, 1]);
    }

    #[test]
    fn circular_coding() {
        let now = MessageBuilder::now();

        assert_eq!(
            now,
            VarUInt::decode(&mut VarUInt(now).encode().as_slice()).0
        );
    }
}
