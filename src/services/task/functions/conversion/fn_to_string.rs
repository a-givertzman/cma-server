use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// Function | Converts input to String
#[derive(Debug)]
pub struct FnToString {
    id: String,
    kind: FnKind,
    input: FnOutRef,
}
//
impl FnToString {
    ///
    /// Creates new instance of the FnToString
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnToString{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            kind: FnKind::Fn,
            input,
            id,
        })
    }    
}
// 
impl FnOut for FnToString { 
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
        //             Point::Bool(value) => &value.value.0.to_string(),
        //             Point::Int(value) => &value.value.to_string(),
        //             Point::Real(value) => &value.value.to_string(),
        //             Point::Double(value) => &value.value.to_string(),
        //             Point::String(value) => &value.value,
        //             Point::Bytes(value) => &value.to_string().value,
        //         };
        //         log::trace!("{}.out | out: {:?}", self.id, &out);
        //         FnResult::Ok(Point::String(
        //             PointHlr::new(
        //                 input.txid(),
        //                 &concat_string!(self.id, ".out"),
        //                 out.to_owned(),
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
    fn hard_reset(&mut self) {
        self.input.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnToString instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
