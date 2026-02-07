//!
//! Display input points on the various types of diagrams.
//! 
//! **Note !** To activate fn Plot use:
//! - `cargo test --features=plot` or 
//! - `cargo run --features=plot`
mod fn_plot;
#[cfg(feature = "plot")]
mod ui_plot;

pub use fn_plot::*;
#[cfg(feature = "plot")]
pub use ui_plot::*;


