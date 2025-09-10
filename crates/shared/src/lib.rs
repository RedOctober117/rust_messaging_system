pub mod communication_state_data;
pub mod decode;
pub mod encode;
pub mod login_state_data;
pub mod message;
pub mod message_builder;
pub mod message_data;
pub mod node;
pub mod varuint;

pub const NO_STATE: u8 = 0;

extern crate pretty_env_logger;
