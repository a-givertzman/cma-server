//! ### Services implemented for the application
//! **Service**:
//! - executed in the separate thread, can be multi thread
//! - basicaly must be defined in the main configuration file like:
//! ```yaml
//! service ServiceName Id:
//!     in queue in-queue:
//!         max-length: 10000
//!     send-to: MultiQueue.in-queue
//! ```
///
mod api_cient;
pub use api_cient::*;
pub mod app;
mod cache;
pub use cache::*;
pub mod diagnosis;
mod frdm_service;
pub use frdm_service::*;
pub mod history;
mod modbus_tcp;
pub use modbus_tcp::*;
pub mod profinet_client;
pub mod server;
pub mod slmp_client;
pub mod task;
pub mod tcp_client;
pub mod udp_client;
mod virtual_device;
pub use virtual_device::*;
mod services_factory;
pub use services_factory::*;

