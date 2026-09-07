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
// By Anton Lobanov 6.07.2026 FRDM disabled for GAZ-192103-release-with-registrator
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
// pub mod udp_client;      // Перенес функционал в vibro_monitir, оригинальный код оставил на месте
mod virtual_device;
pub use virtual_device::*;
mod services_factory;
pub use services_factory::*;
mod wear_monitor;
pub use wear_monitor::*;
mod vibro_monitor;
pub use vibro_monitor::*;
mod event_values;
pub use event_values::*;
