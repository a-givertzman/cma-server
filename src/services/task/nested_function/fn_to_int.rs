use log::trace;
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    core_::{point::{point_hlr::PointHlr, point::Point}, types::{fn_in_out_ref::FnInOutRef, type_of::DebugTypeOf}},
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Converts input to Int
///  - bool: true -> 1, false -> 0
///  - real: 0.1 -> 0 | 0.5 -> 1 | 0.9 -> 1 | 1.1 -> 1
///  - string: try to parse int
#[derive(Debug)]
pub struct FnToInt {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
}
//
// 
impl FnToInt {
    ///
    /// Creates new instance of the FnToInt
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnToInt{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }    
}
//
// 
impl FnIn for FnToInt {}
//
// 
impl FnOut for FnToInt { 
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
    fn out(&mut self) -> FnResult<Point, String> {
        let input = self.input.borrow_mut().out();
        trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                let out = match &input {
                    Point::Bool(value) => {
                        if value.value.0 {1} else {0}
                    }
                    Point::Int(value) => {
                        value.value
                    }
                    Point::Real(value) => {
                        value.value.round() as i64
                    }
                    Point::Double(value) => {
                        value.value.round() as i64
                    }
                    _ => panic!("{}.out | {:?} type is not supported: {:?}", self.id, input.print_type_of(), input),
                };
                trace!("{}.out | out: {:?}", self.id, &out);
                FnResult::Ok(Point::Int(
                    PointHlr::new(
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
impl FnInOut for FnToInt {}
///
/// Global static counter of FnToInt instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
