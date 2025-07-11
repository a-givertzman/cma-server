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
mod bendings_conf;
mod frdm_service_conf;
mod frdm_service;
mod rope_conf;

pub use bendings_conf::*;
pub use frdm_service_conf::*;
pub use frdm_service::*;
pub use rope_conf::*;