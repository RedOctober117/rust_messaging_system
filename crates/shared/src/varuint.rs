pub type VarUInt = u32;
pub type Result<T> = std::result::Result<T, VarUIntError>;

const SEGMENT_BITS: u8 = 0x7F;
const LEADING_BIT: u8 = 0x80;
const MAX_VARINT_SIZE: u8 = 5;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum VarUIntError {
    DecodedValueTooLong,
}

// https://en.wikipedia.org/wiki/LEB128
pub fn decode_varuint(bytes: &[u8]) -> VarUInt {
    let mut result: VarUInt = 0;
    let mut shift: u8 = 0;

    for byte in bytes {
        result |= (byte.to_owned() as u32 & SEGMENT_BITS as u32) << shift;
        shift += 7;

        if byte & LEADING_BIT == 0 {
            break;
        }
    }

    result
}

pub fn encode_varuint(to_be_encoded: VarUInt) -> Result<Vec<u8>> {
    let mut result = vec![];
    let mut val = to_be_encoded;

    loop {
        let mut byte = val & SEGMENT_BITS as u32;
        val >>= 7;
        if val != 0 {
            byte |= LEADING_BIT as u32;
            result.push(byte as u8);
        } else {
            result.push(byte as u8);
            break;
        }
    }

    if result.len() > MAX_VARINT_SIZE.into() {
        Err(VarUIntError::DecodedValueTooLong)
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::varuint::*;

    #[test]
    fn decode_test() {
        assert_eq!(decode_varuint(&[132_u8, 6_u8]), 772);
        assert_eq!(decode_varuint(&[221, 199, 1]), 25565);
    }

    #[test]
    fn encode_test() {
        assert_eq!(encode_varuint(772).unwrap(), [132_u8, 6_u8]);
        assert_eq!(encode_varuint(25565).unwrap(), [221, 199, 1]);
    }
}
