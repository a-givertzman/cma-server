use log::trace;
use sal_sync::services::{entity::point::{point::Point, point_hlr::PointHlr}, types::type_of::DebugTypeOf};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    core_::types::fn_in_out_ref::FnInOutRef,
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Converts input to Real
///  - bool: true -> 1.0, false -> 0.0
///  - string: try to parse Real
#[derive(Debug)]
pub struct FnToReal {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
}
//
// 
impl FnToReal {
    ///
    /// Creates new instance of the FnToReal
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnToReal{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }    
}
//
// 
impl FnIn for FnToReal {}
//
// 
impl FnOut for FnToReal { 
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
                        if value.value.0 {1.0f32} else {0.0f32}
                    }
                    Point::Int(value) => {
                        value.value as f32
                    }
                    Point::Real(value) => {
                        value.value
                    }
                    Point::Double(value) => {
                        value.value as f32
                    }
                    _ => panic!("{}.out | {:?} type is not supported: {:?}", self.id, input.print_type_of(), input),
                };
                trace!("{}.out | out: {:?}", self.id, &out);
                FnResult::Ok(Point::Real(
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
impl FnInOut for FnToReal {}
///
/// Global static counter of FnToReal instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
