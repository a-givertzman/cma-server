//!
//! Allows to build a declorative logic in the Task service using specific syntax in the yaml file
//! 
//! Fallowing example implements logging a result of the comparation '/App/Ied001/Load' >= 5.5:
//!   - if point '/App/Ied001/Load' has value > 5.5 'true' will be logged
//!   - if point '/App/Ied001/Load' has value < 5.5 'false' will be logged
//! ```yaml
//! fn Debug:
//!     input fn Ge:
//!         input1: point real '/App/Ied001/Load'
//!         input2: const real 5.5
//! ```
//! 
//! The embedded functions and keywords must be used in the lower case:
//! - var
//! - input
//! - const
//! 
//! Another functions must be used in CamelCase:
//! - Add
//! - Ge  
//! etc...

mod application;
mod common;
mod comp;
mod conversion;
mod core;
mod edge_detection;
mod export;
mod filter;
mod import;
mod io;
mod ops;
mod plot;
mod sql;
mod timers;
mod fn_builder;
mod functions;

pub use application::*;
pub use common::*;
pub use comp::*;
pub use conversion::*;
pub use core::*;
pub use edge_detection::*;
pub use export::*;
pub use filter::*;
pub use import::*;
pub use io::*;
pub use ops::*;
pub use plot::*;
pub use sql::*;
pub use timers::*;
pub use fn_builder::*;
