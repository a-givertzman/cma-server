use log::trace;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    core_::{point::{point_hlr::PointHlr, point_type::PointType}, types::{bool::Bool, fn_in_out_ref::FnInOutRef}},
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Returns true one tic (single computation cycle)
/// if input value falling true -> false (any positive -> 0 (or any negative))
#[derive(Debug)]
pub struct FnFallingEdge {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
    prev: bool,
}
//
// 
impl FnFallingEdge {
    ///
    /// Creates new instance of the FnFallingEdge
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnFallingEdge{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
            prev: false,
        }
    }    
}
//
// 
impl FnIn for FnFallingEdge {}
//
// 
impl FnOut for FnFallingEdge { 
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
    //
    //
    fn out(&mut self) -> FnResult<PointType, String> {
        let input = self.input.borrow_mut().out();
        trace!("{}.out | input: {:#?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                let input_value = input.to_bool().as_bool().value.0;
                let value = PointType::Bool(PointHlr::new(
                    input.tx_id(),
                    &input.name(),
                    Bool((! input_value) && self.prev),
                    input.status(),
                    input.cot(),
                    input.timestamp(),
                ));
                self.prev = input_value;
                trace!("{}.out | value: {:#?}", self.id, value);
                FnResult::Ok(value)
            }
            FnResult::None => {
                self.prev = false;
                FnResult::None
            }
            FnResult::Err(err) => {
                self.prev = false;
                FnResult::Err(err)
            }
        }
    }
    //
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
        self.prev = false;
    }
}
//
// 
impl FnInOut for FnFallingEdge {}
///
/// Global static counter of FnFallingEdge instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
