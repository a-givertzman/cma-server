use std::sync::atomic::{Ordering, AtomicUsize};
use log::trace;
use crate::core_::{point::point::Point, types::fn_in_out_ref::FnInOutRef};
use super::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult};
///
/// Specific kinde of function
/// - has additional method .eval(), 
/// nothing returns, 
/// but evaluete all calculations,
/// result stores into inner
/// - calculated result returns in .out() method
/// - out() method do not evaluete calculations, just returns the result
#[derive(Debug, Clone)]
pub struct FnVar {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
    value: Option<FnResult<Point, String>>,
}
//
// 
impl FnVar {
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self {
            id: format!("{}/FnVar{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Var,
            input,
            value: None, 
        }
    }
}
//
// 
impl FnIn for FnVar {}
//
// 
impl FnOut for FnVar {
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> &FnKind {
        &self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        self.input.borrow().inputs()
    }
    ///
    /// Returns nothing, 
    /// - Evaluetes all calculations,
    /// - Result stores into inner
    /// - calculated result returns in .out() method
    fn eval(&mut self) {
        trace!("{}.eval | evaluating...", self.id);
        self.value = Some(self.input.borrow_mut().out());
    }
    ///
    /// Do not evaluete calculations, 
    /// just returns the result if evalueted, evaluate
    fn out(&mut self) -> FnResult<Point, String> {
        let value = match &self.value {
            Some(value) => {
                trace!("{}.out | value: {:?}", self.id, &self.value);
                value.clone()
            }
            None => {
                trace!("{}.eval | evaluating...", self.id);
                let value = self.input.borrow_mut().out();
                self.value = Some(value.clone());
                value
                // panic!("{}.out | not initialised", self.id);
            }
        };
        value
    }
    //
    fn reset(&mut self) {
        self.value = None;
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnVar {}
///
/// Global static counter of FnVar instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
