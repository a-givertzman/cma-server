//!
//! # Crane rope diagnosis
//! 
//! - Rope defect detection using frames from the Camera
//! - Rope deprecation calculated by the bends on the enter and exit winch drum and blocks
//! 
//! ## [Configuration](src/services/frdm_service/frdm_service_conf.rs)
//! 
mod rope_defect;
mod rope_deprecation;
mod frdm_service_conf;
mod frdm_service;

pub(crate) use rope_defect::*;
pub(crate) use rope_deprecation::*;
pub use frdm_service_conf::*;
pub use frdm_service::*;
