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
/// Function | Converts input to Int
///  - bool: true -> 1, false -> 0
///  - real: 0.1 -> 0 | 0.5 -> 1 | 0.9 -> 1 | 1.1 -> 1
///  - string: try to parse int
#[derive(Debug)]
pub struct FnToInt {
    id: String,
    kind: FnKind,
    input: FnOutRef,
}
//
// 
impl FnToInt {
    ///
    /// Creates new instance of the FnToInt
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnToInt{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }    
}
//
// 
impl FnOut for FnToInt { 
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
                log::trace!("{}.out | out: {:?}", self.id, &out);
                FnResult::Ok(Point::Int(
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
/// Global static counter of FnToInt instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
