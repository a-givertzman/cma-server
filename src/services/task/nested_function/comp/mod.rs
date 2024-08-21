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
pub mod fn_gt;
pub mod fn_ge;
pub mod fn_eq;
pub mod fn_le;
pub mod fn_lt;
pub mod fn_ne;
