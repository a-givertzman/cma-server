use log::trace;
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    core_::{point::{point::Point, point_type::PointType}, types::{fn_in_out_ref::FnInOutRef, type_of::DebugTypeOf}},
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind,
    },
};

use super::fn_result::FnResult;
///
/// Function converts input to Double
///  - bool: true -> 1.0, false -> 0.0
///  - string: try to parse double
#[derive(Debug)]
pub struct FnToDouble {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
}
//
// 
impl FnToDouble {
    ///
    /// Creates new instance of the FnToDouble
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnToDouble{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }    
}
//
// 
impl FnIn for FnToDouble {}
//
// 
impl FnOut for FnToDouble { 
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
        trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                let out = match &input {
                    PointType::Bool(value) => {
                        if value.value.0 {1.0f64} else {0.0f64}
                    }
                    PointType::Int(value) => {
                        value.value as f64
                    }
                    PointType::Real(value) => {
                        value.value as f64
                    }
                    PointType::Double(value) => {
                        value.value
                    }
                    _ => panic!("{}.out | {:?} type is not supported: {:?}", self.id, input.print_type_of(), input),
                };
                trace!("{}.out | out: {:?}", self.id, &out);
                FnResult::Ok(PointType::Double(
                    Point::new(
                        input.tx_id(),
                        &concat_string!(self.id, ".out"),
                        out,
                        input.status(),
                        input.cot(),
                        input.timestamp(),
                    )
                ))
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    //
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnToDouble {}
///
/// Global static counter of FnToDouble instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
