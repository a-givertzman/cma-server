use log::trace;
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    core_::{point::{point_hlr::PointHlr, point::Point}, types::fn_in_out_ref::FnInOutRef},
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Converts input to String
#[derive(Debug)]
pub struct FnToString {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
}
//
// 
impl FnToString {
    ///
    /// Creates new instance of the FnToString
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnToString{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }    
}
//
// 
impl FnIn for FnToString {}
//
// 
impl FnOut for FnToString { 
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
                    Point::Bool(value) => &value.value.0.to_string(),
                    Point::Int(value) => &value.value.to_string(),
                    Point::Real(value) => &value.value.to_string(),
                    Point::Double(value) => &value.value.to_string(),
                    Point::String(value) => &value.value,
                };
                trace!("{}.out | out: {:?}", self.id, &out);
                FnResult::Ok(Point::String(
                    PointHlr::new(
                        input.tx_id(),
                        &concat_string!(self.id, ".out"),
                        out.to_owned(),
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
impl FnInOut for FnToString {}
///
/// Global static counter of FnToString instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
