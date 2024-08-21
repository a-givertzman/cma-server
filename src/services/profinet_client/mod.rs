//!
//! Implements communication with Siemens device over PROFINET protocol.
//!
//! Cyclically reads adressess from the device 
//! and yields changed to the specified destination service.
//! Writes Point to the device specific address.

pub mod profinet_client;

pub mod profinet_db;

pub mod s7;

pub mod parse_point;