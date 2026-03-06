//! Comparison functions compare two or more variables returning a bool Point containing TRUE or FALSE.
//! 
//!  Function | Operator | Description
//! :-------:|:--------:|-------------
//!   Gt     |    >     | Greater than
//!   Ge     |    >=    | Greater than or equal to
//!   Eq     |    =     | Equal
//!   Le     |    <=    | Less than or equal to
//!   Lt     |    <     | Less than
//!   Ne     |    <>    | Not equal to
//! 
//! Example
//! 
//! `Point.A >= 0.5`
//! 
//! ```yaml
//! fn Ge:
//!     input1: point real /App/Service/Point.A
//!     input2: const real 0.5
//! ```
mod fn_gt;
mod fn_ge;
mod fn_eq;
mod fn_le;
mod fn_lt;
mod fn_ne;

pub use fn_gt::*;
pub use fn_ge::*;
pub use fn_eq::*;
pub use fn_le::*;
pub use fn_lt::*;
pub use fn_ne::*;