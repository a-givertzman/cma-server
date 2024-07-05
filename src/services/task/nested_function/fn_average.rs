use std::sync::atomic::{AtomicUsize, Ordering};
use log::trace;
use concat_string::concat_string;
use crate::{conf::point_config::point_config_type::PointConfigType, core_::{
    point::{point::Point, point_type::PointType},
    types::fn_in_out_ref::FnInOutRef,
}};
use super::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult};
///
/// Returns an average value (in Double) of the input
#[derive(Debug)]
pub struct FnAverage {
    id: String,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    input: FnInOutRef,
    count: i64,
    sum: f64,
}
//
// 
impl FnAverage {
    ///
    /// Creates new instance of the FnAverage
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnAverage{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            enable,
            input,
            count: 0,
            sum: 0.0,
        }
    }
}
//
// 
impl FnIn for FnAverage {}
//
// 
impl FnOut for FnAverage {
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
        if let Some(enable) = &self.enable {
            inputs.append(&mut enable.borrow().inputs());
        }
        inputs.append(&mut self.input.borrow().inputs());
        inputs
    }
    //
    fn out(&mut self) -> FnResult<PointType, String> {
        let enable = match &self.enable {
            Some(enable) => {
                let enable = enable.borrow_mut().out();
                trace!("{}.out | enable: {:?}", self.id, enable);
                match enable {
                    FnResult::Ok(enable) => enable.to_bool().as_bool().value.0,
                    FnResult::None => return FnResult::None,
                    FnResult::Err(err) => return FnResult::Err(err),
                }
            }
            None => true,
        };
        if enable {
            let input = self.input.borrow_mut().out();
            // trace!("{}.out | input: {:?}", self.id, input);
            match input {
                FnResult::Ok(input) => {
                    let value = input.to_double().as_double().value;
                    self.sum += value;
                    self.count += 1;
                    let average = if self.count != 0 {
                        self.sum / (self.count as f64)
                    } else {
                        0.0
                    };
                    trace!("{}.out | sum: {:?}", self.id, self.sum);
                    trace!("{}.out | count: {:?}", self.id, self.count);
                    trace!("{}.out | average: {:?}", self.id, average);
                    match input.type_() {
                        PointConfigType::Int => {
                            FnResult::Ok(PointType::Int(
                                Point::new(
                                    input.tx_id(),
                                    &self.id,
                                    average.round() as i64,
                                    input.status(),
                                    input.cot(),
                                    input.timestamp(),
                                )
                            ))
                        }
                        PointConfigType::Real => {
                            FnResult::Ok(PointType::Real(
                                Point::new(
                                    input.tx_id(),
                                    &self.id,
                                    average as f32,
                                    input.status(),
                                    input.cot(),
                                    input.timestamp(),
                                )
                            ))
                        }
                        PointConfigType::Double => {
                            FnResult::Ok(PointType::Double(
                                Point::new(
                                    input.tx_id(),
                                    &self.id,
                                    average,
                                    input.status(),
                                    input.cot(),
                                    input.timestamp(),
                                )
                            ))
                        }
                        _ => FnResult::Err(concat_string!(self.id, ".out | Invalid input type '", input.type_().to_string(), "'")),
                    }
                }
                FnResult::None => FnResult::None,
                FnResult::Err(err) => FnResult::Err(err),
            }
        } else {
            self.count = 0;
            self.sum = 0.0;
            FnResult::None
        }
    }
    //
    fn reset(&mut self) {
        self.count = 0;
        self.sum = 0.0;
        if let Some(enable) = &self.enable {
            enable.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnAverage {}
///
/// Global static counter of FnAverage instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
