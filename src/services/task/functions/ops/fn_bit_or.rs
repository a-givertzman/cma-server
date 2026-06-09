use sal_core::error::Error;
use sal_sync::services::{
    entity::{Cot, {Point, PointHlr, PointTxId}, Status},
    types::Bool,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::Utc;
use crate::{
    domain::FnOutRef,
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },

};
///
/// Function | `FnBitOr`
/// 
/// Example
/// 
/// ```yaml
/// fn BitOr:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn BitOr:
///     input1: point bool '/App/Service/Point.Name1'
///     input2: point bool '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnBitOr {
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
//
// 
impl FnBitOr {
    ///
    /// Creates new instance of the FnBitOr
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnBitOr{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self { 
            kind:FnKind::Fn,
            inputs,
            id,
        })
    }
}
//
// 
impl FnOut for FnBitOr {
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
        let mut inputs = vec![];
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        unimplemented!();
        let mut inputs = self.inputs.iter();
        // let mut value: Point;
        // match inputs.next() {
        //     Some(first) => {
        //         value = match first.borrow_mut().out() {
        //             FnResult::Ok(first) => first,
        //             FnResult::None => return FnResult::None,
        //             FnResult::Err(err) => return FnResult::Err(err),
        //         };
        //         while let Some(input) = inputs.next() {
        //             let input = input.borrow_mut().out();
        //             match input {
        //                 FnResult::Ok(input) => {
        //                     log::trace!("{}.out | input '{}': {:?}", self.id, input.name(), input.value());
        //                     value = match &value {
        //                         Point::Bool(val) => {
        //                             let input_val = input.try_as_bool().unwrap_or_else(|_| panic!("{}.out | Incopatable types, expected '{:?}', but input '{}' has type '{:?}'", self.id, value.type_(), input.name(), input.type_()));
        //                             Point::Bool(
        //                                 PointHlr::new(
        //                                     tx_id,
        //                                     &format!("{}.out", self.id),
        //                                     Bool(val.value.0 | input_val.value.0),
        //                                     Status::Ok,
        //                                     Cot::Inf,
        //                                     Utc::now(),
        //                                 )
        //                             )
        //                         }
        //                         Point::Int(val) => {
        //                             let input_val = input.try_as_int().unwrap_or_else(|_| panic!("{}.out | Incopatable types, expected '{:?}', but input '{}' has type '{:?}'", self.id, value.type_(), input.name(), input.type_()));
        //                             Point::Int(
        //                                 PointHlr::new(
        //                                     tx_id,
        //                                     &format!("{}.out", self.id),
        //                                     val.value | input_val.value,
        //                                     Status::Ok,
        //                                     Cot::Inf,
        //                                     Utc::now(),
        //                                 )
        //                             )
        //                         }
        //                         Point::Real(_) => {
        //                             panic!("{}.out | Not implemented for Real", self.id);
        //                         }
        //                         Point::Double(_) => {
        //                             panic!("{}.out | Not implemented for Double", self.id);
        //                         }
        //                         Point::String(_) => {
        //                             panic!("{}.out | Not implemented for String", self.id);
        //                         }
        //                         Point::Bytes(_) => {
        //                             panic!("{}.out | Not implemented for Bytes", self.id);
        //                         }
        //                     };
        //                 }
        //                 FnResult::None => return FnResult::None,
        //                 FnResult::Err(err) => return FnResult::Err(err),
        //             }
        //         }
        //     },
        //     None => panic!("{}.out | At least one input must be specified", self.id),
        // };
        // // trace!("{}.out | value: {:#?}", self.id, value);
        // FnResult::Ok(value)
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnBitOr instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
