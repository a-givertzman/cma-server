use sal_sync::services::{entity::{Point, PointHlr}, types::DebugTypeOf};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::functions::{
        FnOut, FnKind, FnResult,
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
    input: FnOutRef,
}
//
// 
impl FnToReal {
    ///
    /// Creates new instance of the FnToReal
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnToReal{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }    
}
//
// 
impl FnOut for FnToReal { 
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
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = self.input.borrow_mut().out();
        log::trace!("{}.out | input: {:?}", self.id, input);
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
                log::trace!("{}.out | out: {:?}", self.id, &out);
                FnResult::Ok(Point::Real(
                    PointHlr::new(
                        input.txid(),
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
///
/// Global static counter of FnToReal instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
