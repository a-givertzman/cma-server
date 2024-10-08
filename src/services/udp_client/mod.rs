//!
//! # Implements communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol.
//! 
//! - Cyclically reads data from the device 
//! and yields changed to the specified destination service.
//! 
//! - Writes Point to the device specific address.
//! 
//! Configuration example for single Sub MC:
//! 
//! ```yaml
//! service UdpClient UdpClientSencor01:
//!     cycle: 10ms
//!     ...
//! ```
//! 
pub mod parse_point;
pub mod udp_client_db;
pub mod udp_client;
pub mod udpc_parse_i16;
