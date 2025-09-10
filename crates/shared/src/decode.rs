use std::{fmt::Debug, io::BufRead};

pub trait Decode: Debug {
    fn decode_reader(
        state: u8,
        reader: &mut impl BufRead,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>>;
}
