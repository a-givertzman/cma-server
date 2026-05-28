use sal_sync::services::entity::Point;
use crate::services::task::FnFlow;

use super::{FnKind, FnResult};
///
/// Input side interface for nested function
/// Used for generic access to the different kinde of functions
/// for adding new value on input side
pub trait FnIn: std::fmt::Debug {
    ///
    /// Adds new value into Input
    fn add(&mut self, point: &Point);
    ///
    /// Returns 'Options hash' to identify unique set of options of the Input
    fn hash(&self) -> String;
}
///
/// Out side interface for the function
/// Used for generic access to the different kinde of functions
/// - to get the calculated value on out side
/// - to reset the state to the initial
pub trait FnOut: std::fmt::Debug {
    ///
    /// Retirns it unique idetificator
    fn id(&self) -> String;
    ///
    /// Returns enum kind of the FnOut
    fn kind(&self) -> FnKind;
    ///
    /// Returns names of inputs it depending on
    fn inputs(&self) -> Vec<String>;
    ///
    /// - Evaluate calculations
    /// - Returns calculated value
    /// - Returns error if:
    ///   - Calculations fails
    ///   - Input not initialized
    /// - Returns None if:
    ///   - Point filtered by any kind of filtering function
    fn out(&mut self) -> FnResult<FnFlow, String>;
    ///
    /// resets self state to the initial, calls reset method of all inputs 
    fn reset(&mut self);
}
///
/// Interface for nested function
/// Used for generic access to the different kinde of functions in the nested tree
pub trait FnInOut: FnIn + FnOut {}