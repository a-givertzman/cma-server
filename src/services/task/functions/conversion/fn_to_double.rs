use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr}, types::DebugTypeOf};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// Function | Converts input to Double
///  - bool: true -> 1.0, false -> 0.0
///  - string: try to parse double
#[derive(Debug)]
pub struct FnToDouble {
    kind: FnKind,
    input: FnOutRef,
    id: String,
}
// 
impl FnToDouble {
    ///
    /// Creates new instance of the FnToDouble
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnToDouble{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            kind: FnKind::Fn,
            input,
            id,
        })
    }    
}
//
// 
impl FnOut for FnToDouble { 
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
    fn out(&mut self) -> FnResult<FnFlow, String> {
        unimplemented!();
        // let mut flow = FlowContext::new();
        // let input = self.input.borrow_mut().out();
        // log::trace!("{}.out | input: {:?}", self.id, input);
        // match input {
        //     FnResult::Ok(input) => {
        //         let out = match &input {
        //             Point::Bool(value) => {
        //                 if value.value.0 {1.0f64} else {0.0f64}
        //             }
        //             Point::Int(value) => {
        //                 value.value as f64
        //             }
        //             Point::Real(value) => {
        //                 value.value as f64
        //             }
        //             Point::Double(value) => {
        //                 value.value
        //             }
        //             _ => panic!("{}.out | {:?} type is not supported: {:?}", self.id, input.print_type_of(), input),
        //         };
        //         log::trace!("{}.out | out: {:?}", self.id, &out);
        //         FnResult::Ok(Point::Double(
        //             PointHlr::new(
        //                 input.txid(),
        //                 &concat_string!(self.id, ".out"),
        //                 out,
        //                 input.status(),
        //                 input.cot(),
        //                 input.timestamp(),
        //             )
        //         ))
        //     }
        //     FnResult::None => FnResult::None,
        //     FnResult::Err(err) => FnResult::Err(err),
        // }
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToDouble instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
