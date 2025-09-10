use std::{fmt::Debug, io::Read};

pub trait Decode: Debug {
    fn decode_reader(reader: &mut impl Read) -> Result<Box<Self>, Box<dyn std::error::Error>>;
}
