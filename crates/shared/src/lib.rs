pub mod communication_state_data;
pub mod decode;
pub mod encode;
pub mod login_state_data;
pub mod message;
pub mod message_builder;
pub mod message_data;
pub mod source_or_destination;
pub mod varuint;

pub const NO_STATE: u8 = 0;
pub type BoxedError = Box<dyn std::error::Error>;

extern crate pretty_env_logger;
