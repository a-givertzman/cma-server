//! 
//! # Implements communication with Modbus-TCP device
//! over Modbus protocol (ethernet).
//!
//! - Cyclically reads registers from the device 
//! and yields changed to the specified destination service.
//! - Writes Point to the device specific address.
//!
mod function_code;
mod modbus_error;
mod modbus_message;
mod modbus_parse_bool;
mod modbus_parse_int;
mod modbus_parse_real;
mod parse_point;

pub use function_code::*;
pub use modbus_error::*;
pub use modbus_message::*;
pub use modbus_parse_bool::*;
pub use modbus_parse_int::*;
pub use modbus_parse_real::*;
pub(super) use parse_point::*;