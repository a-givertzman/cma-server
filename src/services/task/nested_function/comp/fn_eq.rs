use log::trace;
use concat_string::concat_string;
use sal_sync::services::{entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr}}, types::bool::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    core_::types::fn_in_out_ref::FnInOutRef,
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Equal to
/// FnEq ( input1, input2 ) === input1.value == input2.value
#[derive(Debug)]
pub struct FnEq {
    id: String,
    kind: FnKind,
    input1: FnInOutRef,
    input2: FnInOutRef,
}
//
// 
impl FnEq {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input1: FnInOutRef, input2: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnEq{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input1,
            input2,
        }
    }
}
//
// 
impl FnIn for FnEq {}
//
//
impl FnOut for FnEq {
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
        let mut inputs = self.input1.borrow().inputs();
        inputs.extend(self.input2.borrow().inputs());
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<Point, String> {
        let input1 = self.input1.borrow_mut().out();
        let input2 = self.input2.borrow_mut().out();
        trace!("{}.out | input1: {:?}", self.id, &input1);
        trace!("{}.out | input2: {:?}", self.id, &input2);
        match (input1, input2) {
            (FnResult::Ok(input1), FnResult::Ok(input2)) => {
                let value = input1.value() == input2.value();
                trace!("{}.out | value: {:?}", self.id, &value);
                let status = match input1.status().cmp(&input2.status()) {
                    std::cmp::Ordering::Less => input2.status(),
                    std::cmp::Ordering::Equal => input1.status(),
                    std::cmp::Ordering::Greater => input1.status(),
                };
                let (tx_id, timestamp) = match input1.timestamp().cmp(&input2.timestamp()) {
                    std::cmp::Ordering::Less => (input2.tx_id(), input2.timestamp()),
                    std::cmp::Ordering::Equal => (input1.tx_id(), input1.timestamp()),
                    std::cmp::Ordering::Greater => (input1.tx_id(), input1.timestamp()),
                };
                FnResult::Ok(Point::Bool(
                    PointHlr::new(
                        tx_id,
                        &format!("{}.out", self.id),
                        Bool(value),
                        status,
                        Cot::Inf,
                        timestamp,
                    )
                ))
            }
            (FnResult::Ok(_), FnResult::None) => FnResult::None,
            (FnResult::None, FnResult::Ok(_)) => FnResult::None,
            (FnResult::None, FnResult::None) => FnResult::None,
            (FnResult::Ok(_), FnResult::Err(err)) => FnResult::Err(err),
            (FnResult::None, FnResult::Err(err)) => FnResult::Err(err),
            (FnResult::Err(err), FnResult::Ok(_)) => FnResult::Err(err),
            (FnResult::Err(err), FnResult::None) => FnResult::Err(err),
            (FnResult::Err(err1), FnResult::Err(err2)) => FnResult::Err(concat_string!(err1, "\n", err2)),
        }
    }
    //
    //
    fn reset(&mut self) {
        self.input1.borrow_mut().reset();
        self.input2.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnEq {}
///
/// Global static counter of FnEq instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
