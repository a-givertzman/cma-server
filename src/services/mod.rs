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
pub mod api_cient;

pub mod tcp_client;

pub mod profinet_client;

pub mod task;

pub mod services;

pub mod multi_queue;

pub mod server;

pub mod app;

pub mod safe_lock;

pub mod history;

pub mod cache;

pub mod diagnosis;

pub mod slmp_client;

pub mod udp_client;
