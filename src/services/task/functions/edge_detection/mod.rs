//!
//! `Task` Service Functions intend for signal edge dectection, used in the Task service 
//! 
mod fn_falling_edge;
mod fn_rising_edge;

pub use fn_falling_edge::*;
pub use fn_rising_edge::*;