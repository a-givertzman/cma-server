//! 
//! # Implements communication with Modbus-TCP device
//! over Modbus protocol (ethernet).
//!
//! - Cyclically reads registers from the device 
//! and yields changed to the specified destination service.
//! - Writes Point to the device specific address.
//!
mod modbus_parse_bool;
mod modbus_parse_int;
mod modbus_parse_real;

pub use modbus_parse_bool::*;
pub use modbus_parse_int::*;
pub use modbus_parse_real::*;