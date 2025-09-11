use std::{fmt::Debug, io::BufRead};

use crate::BoxedError;

pub type DecodeResult<T> = Result<Box<T>, BoxedError>;

pub trait Decode: Debug {
    fn decode_reader(state: u8, reader: &mut impl BufRead) -> DecodeResult<Self>;
}
