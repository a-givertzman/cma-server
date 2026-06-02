use std::sync::atomic::{Ordering, AtomicUsize};
use crate::{domain::FnOutRef, services::task::FnFlow};
use super::{FnOut, FnKind, FnResult};
///
/// ### Variable | Specific kinde of function
/// - has reference to calculations corresponding to the variable name
#[derive(Debug, Clone)]
pub struct FnVar {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    // value: Option<FnResult<Point, String>>,
}
//
// 
impl FnVar {
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnVar{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Var,
            input,
            // value: None, 
        }
    }
}
//
// 
// impl FnIn for FnVar {}
//
// 
impl FnOut for FnVar {
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> FnKind {
        self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        self.input.borrow().inputs()
    }
    ///
    /// - Evaluate calculations
    /// - Returns calculated value
    /// - Returns error if:
    ///   - Calculations fails
    ///   - Input not initialized
    /// - Returns None if:
    ///   - Point filtered by any kind of filtering function
    fn out(&mut self) -> FnResult<FnFlow, String> {
        log::trace!("{}.eval | evaluating...", self.id);
        let value = self.input.borrow_mut().out();
        log::trace!("{}.out | value: {:?}", self.id, value);
        value
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
//
// 
// impl FnInOut for FnVar {}
///
/// Global static counter of FnVar instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
