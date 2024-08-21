//! 
//! Implements communication with Mitsubishi device
//! over SLMP protocol (FX5 Eth module).
//!
//! Cyclically reads adressess from the device 
//! and yields changed to the specified destination service.
//! Writes Point to the device specific address.
//!
pub mod slmp_cpu_error;
pub mod slmp_eth_error;

pub mod slmp_parse_bool;
pub mod slmp_parse_int;
pub mod slmp_parse_real;

pub mod device_code;

pub mod c_slmp_const;
pub mod c_slmp_info;

pub mod slmp_packet;