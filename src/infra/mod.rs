//!
//! Infrastructure modules
//! 
//! - File system access
//! - API access
//! - Database access
//! - External modules integration
//! - External devices integration
//! 
mod api_client_conf;
mod api_client;
pub mod message;

pub use api_client_conf::*;
pub use api_client::*;