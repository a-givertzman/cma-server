//!
//! Display input points on the various types of diagrams.
//! 
//! **Note !** To activate fn Plot use:
//! - `cargo test --features=plot` or 
//! - `cargo run --features=plot`
pub mod fn_plot;
#[cfg(feature = "plot")]
pub mod ui_plot;
