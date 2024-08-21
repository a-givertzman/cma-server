//!
//! Cyclically reads adressess from the PROFINET device and yields changed to the MultiQueue
//! Writes Point to the protocol (PROFINET device) specific address
pub mod slmp_client;

pub mod slmp_db;

pub mod slmp;

pub mod parse_point;

pub mod slmp_read;

pub mod slmp_write;
