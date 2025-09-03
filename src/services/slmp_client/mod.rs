//!
//! Communication with the Mitsubishi SLMP device
//! 
//! - Cyclically reads adressess from the Mitsubishi SLMP device and yields changes
//! - Writes Events to the protocol (SLMP device) specific address
//! 
pub mod slmp_client;
pub mod slmp_db;
pub mod slmp;
pub mod parse_point;
pub mod slmp_read;
pub mod slmp_write;
