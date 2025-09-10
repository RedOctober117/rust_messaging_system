use std::{fmt::Debug, io::Read};

pub trait Decode: Debug {
    fn decode_reader(
        state: u8,
        reader: &mut impl Read,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>>;
}
