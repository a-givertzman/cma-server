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
mod boom_conf;
mod crane_conf;
mod rope_defect;
mod rope_deprecation;
mod frdm_service_conf;
mod frdm_service;
mod rope_conf;
mod rope_slice;
mod rope_slices;

pub(crate) use bendings_conf::*;
pub(crate) use boom_conf::*;
pub(crate) use crane_conf::*;
pub(crate) use rope_defect::*;
use rope_deprecation::*;
pub use frdm_service_conf::*;
pub use frdm_service::*;
pub(crate) use rope_conf::*;
pub(crate) use rope_slice::*;
pub(crate) use rope_slices::*;
