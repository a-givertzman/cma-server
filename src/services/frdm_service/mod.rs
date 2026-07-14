//!
//! # FRDM (Fiber Rope Defects Monitoring)
//! 
//! - Communication with Camera 
//! - Receives current rope position
//! - Scanning the rope for defects
//! - Calculates Rope Depreciation Rate
//! 
mod rope_defect;
mod rope_deprecation;
mod frdm_service_conf;
mod frdm_service;
mod inputs;

pub(crate) use rope_defect::*;
pub(crate) use rope_deprecation::*;
pub use frdm_service_conf::*;
pub use frdm_service::*;
pub use inputs::*;
