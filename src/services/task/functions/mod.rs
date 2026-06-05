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

pub(crate) trait TryTo<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;
    /// Performs the conversion.
    fn try_to(self) -> Result<T, Self::Error>;
}

impl TryTo<bool> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<bool, Self::Error> {
        match self.type_() {
            sal_sync::services::entity::PointType::Bool |
            sal_sync::services::entity::PointType::Int |
            sal_sync::services::entity::PointType::Real |
            sal_sync::services::entity::PointType::Double => Ok(self.to_bool().as_bool().value.0),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<bool>").err(concat_string::concat_string!("Invalid type '", self.type_().to_string(), "'"))),
        }
    }
}
impl TryTo<i64> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<i64, Self::Error> {
        match self.type_() {
            sal_sync::services::entity::PointType::Bool |
            sal_sync::services::entity::PointType::Int |
            sal_sync::services::entity::PointType::Real |
            sal_sync::services::entity::PointType::Double => Ok(self.to_int().as_int().value),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<i64>").err(concat_string::concat_string!("Invalid type '", self.type_().to_string(), "'"))),
        }
    }
}
impl TryTo<f32> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<f32, Self::Error> {
        match self.type_() {
            sal_sync::services::entity::PointType::Bool |
            sal_sync::services::entity::PointType::Int |
            sal_sync::services::entity::PointType::Real |
            sal_sync::services::entity::PointType::Double => Ok(self.to_real().as_real().value),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<f32>").err(concat_string::concat_string!("Invalid type '", self.type_().to_string(), "'"))),
        }
    }
}
impl TryTo<f64> for &sal_sync::services::entity::Point {
    type Error = sal_core::error::Error;
    fn try_to(self) -> Result<f64, Self::Error> {
        match self.type_() {
            sal_sync::services::entity::PointType::Bool |
            sal_sync::services::entity::PointType::Int |
            sal_sync::services::entity::PointType::Real |
            sal_sync::services::entity::PointType::Double => Ok(self.to_double().as_double().value),
            _ => return Err(sal_core::error::Error::new("Point", "try_to<f64>").err(concat_string::concat_string!("Invalid type '", self.type_().to_string(), "'"))),
        }
    }
}
