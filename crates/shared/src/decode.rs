use std::io::{Read, Result};

pub trait Decode {
    fn decode_reader(reader: &mut impl Read) -> Result<Box<Self>>;
}
