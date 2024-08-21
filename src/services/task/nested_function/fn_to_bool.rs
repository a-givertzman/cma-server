use log::trace;
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    core_::{point::{point::Point, point_type::PointType}, types::{bool::Bool, fn_in_out_ref::FnInOutRef, type_of::DebugTypeOf}},
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Converts input to Bool
///  - bool: true -> 1, false -> 0
///  - real: 0.1 -> 0 | 0.5 -> 1 | 0.9 -> 1 | 1.1 -> 1
///  - string: try to parse bool
#[derive(Debug)]
pub struct FnToBool {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
}
//
// 
impl FnToBool {
    ///
    /// Creates new instance of the FnToBool
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnToBool{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }    
}
//
// 
impl FnIn for FnToBool {}
//
// 
impl FnOut for FnToBool { 
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
        match input {
            FnResult::Ok(input) => {
                trace!("{}.out | input: {:?}", self.id, input);
                let out = match &input {
                    PointType::Bool(value) => {
                        value.value.0
                    }
                    PointType::Int(value) => {
                        value.value > 0
                    }
                    PointType::Real(value) => {
                        value.value > 0.0
                    }
                    PointType::Double(value) => {
                        value.value > 0.0
                    }
                    _ => panic!("{}.out | {:?} type is not supported: {:?}", self.id, input.print_type_of(), input),
                };
                trace!("{}.out | out: {:?}", self.id, &out);
                FnResult::Ok(PointType::Bool(
                    Point::new(
                        input.tx_id(),
                        &concat_string!(self.id, ".out"),
                        Bool(out),
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
impl FnInOut for FnToBool {}
///
/// Global static counter of FnToBool instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
