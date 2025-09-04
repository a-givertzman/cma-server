//! 
//! # Implements communication with Modbus-TCP device
//! over Modbus protocol (ethernet).
//!
//! - Cyclically reads registers from the device 
//! and yields changed to the specified destination service.
//! - Writes Point to the device specific address.
//!
mod function_code;
mod modbus_message;
mod modbus_parse_bool;
mod modbus_parse_int;
mod modbus_parse_real;

pub use function_code::*;
pub use modbus_message::*;
pub use modbus_parse_bool::*;
pub use modbus_parse_int::*;
pub use modbus_parse_real::*;