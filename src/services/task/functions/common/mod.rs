//!
//! `Task` Service common purposes functions
//! 
mod fn_acc;
mod fn_average;
mod fn_count;
mod fn_debug;
mod fn_is_changed_value;
mod fn_hold;
mod fn_max;
mod fn_min;
mod fn_piecewise_linear;
mod fn_piecewise_step;
mod fn_point_id;

pub use fn_acc::*;
pub use fn_average::*;
pub use fn_count::*;
pub use fn_debug::*;
pub use fn_is_changed_value::*;
pub use fn_hold::*;
pub use fn_max::*;
pub use fn_min::*;
pub use fn_piecewise_linear::*;
pub use fn_piecewise_step::*;
pub use fn_point_id::*;
