use std::sync::atomic::{AtomicUsize, Ordering};
use log::trace;
use crate::{conf::point_config::point_config_type::PointConfigType, core_::{
    point::{point::Point, point_type::PointType},
    types::fn_in_out_ref::FnInOutRef,
}};
use super::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult};
///
/// Accumulates numeric incoming Point's value
/// - if input is not numeric - will panic
/// - if input is bool, false = 0, true = 1
#[derive(Debug)]
pub struct FnAcc {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
    acc: Option<PointType>,
    initial: Option<FnInOutRef>,
}
//
// 
impl FnAcc {
    ///
    /// Creates new instance of the FnAcc
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, initial: Option<FnInOutRef>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnAcc{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            input,
            acc: None,
            initial,
        }
    }
}
//
// 
impl FnIn for FnAcc {}
//
// 
impl FnOut for FnAcc {
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
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.borrow().inputs());
        }
        inputs.append(&mut self.input.borrow().inputs());
        inputs
    }
    ///
    fn out(&mut self) -> FnResult<PointType, String> {
        let input = self.input.borrow_mut().out();
        // trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                let acc = match self.acc.clone() {
                    Some(acc) => acc,
                    None => {
                        match &mut self.initial {
                            Some(initial) => {
                                match initial.borrow_mut().out() {
                                    FnResult::Ok(initial) => initial,
                                    FnResult::None => return FnResult::None,
                                    FnResult::Err(err) => return FnResult::Err(err),
                                }
                            }
                            None => match input.type_() {
                                PointConfigType::Bool | PointConfigType::Int  => PointType::Int(Point::new(
                                    input.tx_id(), &input.name(), 0, input.status(), input.cot(), input.timestamp(),
                                )),
                                PointConfigType::Real => PointType::Real(Point::new(
                                    input.tx_id(), &input.name(), 0.0, input.status(), input.cot(), input.timestamp(),
                                )),
                                PointConfigType::Double => PointType::Double(Point::new(
                                    input.tx_id(), &input.name(), 0.0, input.status(), input.cot(), input.timestamp(),
                                )),
                                _ => panic!("{}.out | Invalit input type '{:?}'", self.id, input.type_()),
                            }
                        }
                    }
                };
                let acc = match &input {
                    PointType::Bool(_) => acc + input.to_int(),
                    _ => acc + input,
                };
                trace!("{}.out | out: {:?}", self.id, acc);
                self.acc = Some(acc.clone());
                FnResult::Ok(acc)
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    fn reset(&mut self) {
        if let Some(initial) = &self.initial {
            initial.borrow_mut().reset();
        }
        self.acc = None;
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnAcc {}
///
/// Global static counter of FnAcc instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
