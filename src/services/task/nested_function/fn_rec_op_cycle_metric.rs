use sal_sync::services::{entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr, point_tx_id::PointTxId}, status::status::Status}, types::bool::Bool};
use std::sync::{atomic::{AtomicUsize, Ordering}, mpsc::Sender};
use chrono::Utc;
use indexmap::IndexMap;
use log::{debug, error, trace, warn};
use crate::core_::types::fn_in_out_ref::FnInOutRef;
use super::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult};
///
/// Function | Creates SQL requests on [op-cycle] falling edge:
/// - Operating cycle SQL request (id, start, stop)
/// - Operating cycle metrics SQL requests (cycle_id, pid, metric_id, value)
/// - Returns [enable] input if all inputs are Ok
/// 
/// Example
/// 
/// ```yaml
/// ```
#[derive(Debug)]
pub struct FnRecOpCycleMetric {
    id: String,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    send_to: Option<Sender<Point>>,
    op_cycle: FnInOutRef,
    inputs: IndexMap<String, FnInOutRef>,
    values: Vec<Point>,
    prev: bool,
    rising: bool,
    falling: bool,
}
//
// 
impl FnRecOpCycleMetric {
    ///
    /// Creates new instance of the FnRecOpCycleMetric
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, send_to: Option<Sender<Point>>, op_cycle: FnInOutRef, inputs: IndexMap<String, FnInOutRef>) -> Self {
        Self { 
            id: format!("{}/FnRecOpCycleMetric{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            enable,
            send_to,
            op_cycle,
            inputs,
            values: vec![],
            prev: false,
            rising: false,
            falling: false,
        }
    }
    ///
    /// Sends Point to the external service if 'send-to' specified
    fn send(&self, point: &Point) {
        match &self.send_to {
            Some(tx_send) => match tx_send.send(point.clone()) {
                Ok(_) => {
                    debug!("{}.out | Point sent: {:#?}", self.id, point);
                }
                Err(err) => {
                    error!("{}.out | Send error: {:#?}\n\t point: {:#?}", self.id, err, point);
                }
            }
            None => warn!("{}.out | Point can't be sent - send-to is not specified", self.id),
        }
    }
}
//
// 
impl FnIn for FnRecOpCycleMetric {}
//
// 
impl FnOut for FnRecOpCycleMetric {
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
        inputs.append(&mut self.op_cycle.borrow().inputs());
        for (_, input) in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<Point, String> {
        let (enable, tx_id, status, cot, timestamp) = match &mut self.enable {
            Some(en) => match en.borrow_mut().out() {
                FnResult::Ok(en) => (en.to_bool().as_bool().value.0, en.tx_id(), en.status(), en.cot(), en.timestamp()),
                FnResult::None => return FnResult::None,
                FnResult::Err(err) => return FnResult::Err(err),
            }
            None => (true, PointTxId::from_str(&self.id), Status::Ok, Cot::Inf, Utc::now()),
        };
        let op_cycle = {
            let op_cycle = self.op_cycle.borrow_mut().out();
            match op_cycle {
                FnResult::Ok(op_cycle) => op_cycle.to_bool().as_bool().value.0,
                FnResult::None => return FnResult::None,
                FnResult::Err(err) => return FnResult::Err(err),
            }
        };
        if op_cycle && (! self.prev) {
            debug!("{}.out | Operating Cycle - STARTED", self.id);
            self.rising = true;
            self.falling = false
        };
        if (! op_cycle) && self.prev {
            debug!("{}.out | Operating Cycle - FINISHED", self.id);
            self.falling = true;
            self.rising = false;
        };
        self.prev = op_cycle;
        if self.falling {
            self.falling = false;
            debug!("{}.out | Operating Cycle - SENDING...", self.id);
            let log_values: Vec<String> = self.values.iter().map(|point| {
                format!("{}: {}", point.name(), point.value().to_string())
            }).collect();
            debug!("{}.out | Operating Cycle - values ({}): {:#?}", self.id, self.values.len(), log_values);
            for value in &self.values {
                self.send(value);
            }
            self.values.clear();
        }
        if self.rising && enable {
            trace!("{}.out | Operating Cycle - values", self.id);
            self.values.clear();
            for (input_name, input) in &self.inputs {
                let input = input.borrow_mut().out();
                match input {
                    FnResult::Ok(input) => {
                        trace!("{}.out | Input '{}': {:?}", self.id, input_name, input.value());
                        let value = match input {
                            Point::Bool(mut p) => {
                                p.name = input_name.to_owned();
                                Point::Bool(p)
                            }
                            Point::Int(mut p) => {
                                p.name = input_name.to_owned();
                                Point::Int(p)
                            }
                            Point::Real(mut p) => {
                                p.name = input_name.to_owned();
                                Point::Real(p)
                            }
                            Point::Double(mut p) => {
                                p.name = input_name.to_owned();
                                Point::Double(p)
                            }
                            Point::String(mut p) => {
                                p.name = input_name.to_owned();
                                Point::String(p)
                            }
                        };
                        self.values.push(value)
                    }
                    FnResult::None => {}
                    FnResult::Err(err) => return FnResult::Err(err),
                }
            }
        }
        FnResult::Ok(Point::Bool(
            PointHlr::new(
                tx_id,
                &self.id,
                Bool(enable),
                status,
                cot,
                timestamp,
            )
        ))
    }
    //
    fn reset(&mut self) {
        if let Some(enable) = &self.enable {
            enable.borrow_mut().reset();
        }
        self.op_cycle.borrow_mut().reset();
        for (_, input) in &self.inputs {
            input.borrow_mut().reset();
        }

    }
}
//
// 
impl FnInOut for FnRecOpCycleMetric {}
///
/// Global static counter of FnRecOpCycleMetric instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
