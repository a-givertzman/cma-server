use sal_sync::services::{
    entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr, point_tx_id::PointTxId}, status::status::Status},
    types::bool::Bool,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::Utc;
use log::trace;
use crate::{
    core_::types::fn_in_out_ref::FnInOutRef,
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Returns bitwise AND of all inputs
/// 
/// Example
/// 
/// ```yaml
/// fn BitAnd:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn BitAnd:
///     input1: point bool '/App/Service/Point.Name1'
///     input2: point bool '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnBitAnd {
    id: String,
    kind: FnKind,
    inputs: Vec<FnInOutRef>,
}
//
// 
impl FnBitAnd {
    ///
    /// Creates new instance of the FnBitAnd
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnInOutRef>) -> Self {
        Self { 
            id: format!("{}/FnBitAnd{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            inputs,
        }
    }
}
//
// 
impl FnIn for FnBitAnd {}
//
// 
impl FnOut for FnBitAnd {
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
        let mut inputs = vec![];
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<Point, String> {
        let tx_id = PointTxId::from_str(&self.id);
        let mut inputs = self.inputs.iter();
        let mut value: Point;
        match inputs.next() {
            Some(first) => {
                value = match first.borrow_mut().out() {
                    FnResult::Ok(first) => first,
                    FnResult::None => return FnResult::None,
                    FnResult::Err(err) => return FnResult::Err(err),
                };
                while let Some(input) = inputs.next() {
                    let input = input.borrow_mut().out();
                    match input {
                        FnResult::Ok(input) => {
                            trace!("{}.out | input '{}': {:?}", self.id, input.name(), input.value());
                            value = match &value {
                                Point::Bool(val) => {
                                    let input_val = input.try_as_bool().unwrap_or_else(|_| panic!("{}.out | Incopatable types, expected '{:?}', but input '{}' has type '{:?}'", self.id, value.type_(), input.name(), input.type_()));
                                    Point::Bool(
                                        PointHlr::new(
                                            tx_id,
                                            &format!("{}.out", self.id),
                                            Bool(val.value.0 & input_val.value.0),
                                            Status::Ok,
                                            Cot::Inf,
                                            Utc::now(),
                                        )
                                    )
                                }
                                Point::Int(val) => {
                                    let input_val = input.try_as_int().unwrap_or_else(|_| panic!("{}.out | Incopatable types, expected '{:?}', but input '{}' has type '{:?}'", self.id, value.type_(), input.name(), input.type_()));
                                    Point::Int(
                                        PointHlr::new(
                                            tx_id,
                                            &format!("{}.out", self.id),
                                            val.value & input_val.value,
                                            Status::Ok,
                                            Cot::Inf,
                                            Utc::now(),
                                        )
                                    )
                                }
                                Point::Real(_) => {
                                    panic!("{}.out | Not implemented for Real", self.id);
                                }
                                Point::Double(_) => {
                                    panic!("{}.out | Not implemented for Double", self.id);
                                }
                                Point::String(_) => {
                                    panic!("{}.out | Not implemented for String", self.id);
                                }
                            };
                        }
                        FnResult::None => return FnResult::None,
                        FnResult::Err(err) => return FnResult::Err(err),
                    }
                }
            },
            None => panic!("{}.out | At least one input must be specified", self.id),
        };
        // trace!("{}.out | value: {:#?}", self.id, value);
        FnResult::Ok(value)
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
//
// 
impl FnInOut for FnBitAnd {}
///
/// Global static counter of FnBitAnd instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
