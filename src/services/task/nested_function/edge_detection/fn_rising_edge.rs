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
/// if input value rising false -> true (0 (or any negative) -> any positive)
#[derive(Debug)]
pub struct FnRisingEdge {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
    prev: bool,
}
//
// 
impl FnRisingEdge {
    ///
    /// Creates new instance of the FnRisingEdge
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnRisingEdge{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
            prev: false,
        }
    }    
}
//
// 
impl FnIn for FnRisingEdge {}
//
// 
impl FnOut for FnRisingEdge { 
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
                    Bool(input_value && (! self.prev)),
                    input.status(),
                    input.cot(),
                    input.timestamp(),
                ));
                self.prev = input_value;
                trace!("{}.out | value: {:#?}", self.id, value);
                FnResult::Ok(value)
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
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
impl FnInOut for FnRisingEdge {}
///
/// Global static counter of FnRisingEdge instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
