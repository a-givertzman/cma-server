use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::{entity::{Cot, Point, PointHlr, PointTxId, Status}, types::Bool}, sync::channel::Sender};
use std::sync::{atomic::{AtomicUsize, Ordering}};
use chrono::Utc;
use crate::{domain::FnInOutRef, services::task::{FnIn, FnInOut, FnOut, FnKind, FnResult}};
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
    inputs: FxIndexMap<String, FnInOutRef>,
    values: FxIndexMap<String, Point>,
    state: State,
}
//
// 
impl FnRecOpCycleMetric {
    ///
    /// Creates new instance of the FnRecOpCycleMetric
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, send_to: Option<Sender<Point>>, op_cycle: FnInOutRef, inputs: impl IntoIterator<Item = (String, FnInOutRef)>) -> Self {
        let id = format!("{}/FnRecOpCycleMetric{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self { 
            kind:FnKind::Fn,
            enable,
            send_to,
            op_cycle,
            inputs: inputs.into_iter().collect(),
            values: FxIndexMap::default(),
            state: State::new(&id),
            id,
        }
    }
    ///
    /// Sends Point to the external service if 'send-to' specified
    fn send(&self, point: &Point) {
        match &self.send_to {
            Some(tx_send) => match tx_send.send(point.clone()) {
                Ok(_) => {
                    // log::trace!("{}.out | Point sent: {:#?}", self.id, point);
                }
                Err(err) => {
                    log::error!("{}.out | Send error: {:#?}\n\t point: {:#?}", self.id, err, point);
                }
            }
            None => log::warn!("{}.out | Point can't be sent - 'send-to' is not specified", self.id),
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
        let (enable, txid, status, cot, timestamp) = match &mut self.enable {
            Some(en) => match en.borrow_mut().out() {
                FnResult::Ok(en) => (en.to_bool().as_bool().value.0, en.txid(), en.status(), en.cot(), en.timestamp()),
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
        match self.state.add(op_cycle) {
            Cycle::None => {}
            Cycle::Started => {
                log::trace!("{}.out | Operating Cycle - values", self.id);
                for (input_name, input) in &self.inputs {
                    match input.borrow_mut().out() {
                        FnResult::Ok(input) => {
                            match input {
                                Point::String(p) => {
                                    // log::debug!("{}.out | '{}': {:?}", self.id, input_name, p.value);
                                    // p.name = input_name.to_owned();
                                    self.values.insert(input_name.to_owned(), Point::String(p));
                                }
                                _ => {
                                    log::warn!("{}.out | Input '{}': unexpected type {:?}, string sql requared", self.id, input_name, input.type_());
                                }
                            };
                        }
                        FnResult::None => {}
                        FnResult::Err(err) => {
                            log::warn!("{}.out | Input '{}' - SKIPPED, error: {:?}", self.id, input_name, err);
                        }
                    }
                }
            }
            Cycle::Finished => {
                log::debug!("{}.out | Operating Cycle - SENDING...", self.id);
                let log_values: Vec<String> = self.values.iter().map(|(key, point)| {
                    format!("'{}': '{}'", key, point.value().to_string())
                }).collect();
                log::debug!("{}.out | Operating Cycle - values ({}): {:#?}", self.id, self.values.len(), log_values);
                for (_, value) in &self.values {
                    self.send(value);
                }
                self.values.clear();
            }
        }
        FnResult::Ok(Point::Bool(
            PointHlr::new(
                txid,
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
        self.state.reset();
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
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// State of the Operating cycle
#[derive(Debug, Clone, Copy)]
enum Cycle {
    /// Initial state, nothing hapens while this state
    /// Used to detect the start of the operating cycle
    None,
    /// When op-cycle was raised from 0 to 1
    /// Previous state must be `None`
    Started,
    /// When op-cycle was reseted from 1 to 0.
    /// Used for performing the actions at the end of the operating cycle.
    /// After that immediately must be changed to `None`
    Finished,
}
#[derive(Debug)]
struct State {
    state: Cycle,
    dbg: Dbg,
}
//
impl State {
    ///
    /// Returns `State::None`
    pub fn new(parent: impl Into<String>) -> Self {
        Self {
            state: Cycle::None,
            dbg: Dbg::new(parent, "State")
        }
    }
    ///
    /// Returns current state depending on the OperatingCycle status
    /// `op_cycle` - the OperatinCycle is active as boolean
    pub fn add(&mut self, op_cycle: bool) -> Cycle {
        match self.state {
            Cycle::None => match op_cycle {
                true => {
                    log::debug!("{}.add | Operating Cycle - STARTED", self.dbg);
                    self.state = Cycle::Started;
                    self.state
                },
                false => Cycle::None,
            }
            Cycle::Started => match op_cycle {
                true => Cycle::Started,
                false => {
                    log::debug!("{}.add | Operating Cycle - FINISHED", self.dbg);
                    self.state = Cycle::None;
                    Cycle::Finished
                }
            }
            Cycle::Finished => unreachable!(),
        }
    }
    ///
    /// Reset the state to the initial
    #[allow(unused)]
    pub fn reset(&mut self) {
        self.state = Cycle::None;
    }
}
