use std::sync::{mpsc::Sender, atomic::{AtomicUsize, Ordering}};
use log::{error, trace};
use crate::{
    conf::point_config::{point_config::PointConfig, point_config_type::PointConfigType}, core_::{point::{point_hlr::PointHlr, point_tx_id::PointTxId, point::Point}, types::{bool::Bool, fn_in_out_ref::FnInOutRef}}, services::task::nested_function::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult}
};
///
/// Function | Used for export Point from Task service to another service
///  - Poiont will be sent to the queue only if:
///     - [send-to] - is specified
///     - if [changes-only] is specified and true - changes only will be sent, default false (sending all points)
///  - Returns input Point
/// 
/// Example
/// 
/// ```yaml
/// input point Point.Name:                     # full name will be: /App/Task/Point.Name
///     type: 'Real'                            # Bool / Int / Real / String / Json
///     history: r                              # Optional, r / w / rw
///     alarm: 1                                # Optional, 0..15
///     filters:                                # Optional, Filter conf, using such filter, point can be filtered immediately after input's parser
///         threshold: 5.0                      #   absolute threshold delta
///         factor: 0.1                         #   optional, multiplier for absolute threshold delta - in this case the delta will be accumulated
///     comment: Point produced from the Task   # Optional
///     input: point real '/App/Load'           # Optional
///     send-to: /App/MultiQueue.in-queue       # Optional
///     enable: const bool true                 # Optional, default true, enables the export if true (>0)
///     changes-only: const bool false          # Optional, default false
/// ```
#[derive(Debug)]
pub struct FnPoint {
    id: String,
    tx_id: usize,
    kind: FnKind,
    conf: PointConfig,
    enable: Option<FnInOutRef>,
    changes_only: Option<FnInOutRef>,
    input: Option<FnInOutRef>,
    send_to: Option<Sender<Point>>,
    state: Option<Point>,
}
//
//
impl FnPoint {
    ///
    /// Creates new instance of the FnPoint
    /// - id - just for proper debugging
    /// - input - incoming points
    /// - if [changes-only] is specified and true - changes only will be sent, default false (sending all points)
    pub fn new(parent: impl Into<String>, conf: PointConfig, enable: Option<FnInOutRef>, changes_only: Option<FnInOutRef>, input: Option<FnInOutRef>, send_to: Option<Sender<Point>>) -> Self {
        let self_id = format!("{}/FnPoint{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            id: self_id.clone(),
            tx_id: PointTxId::from_str(&self_id),
            kind: FnKind::Fn,
            conf,
            enable,
            changes_only,
            input,
            send_to,
            state: None,
        }
    }
    ///
    /// 
    fn send(&self, point: &Point) {
        if let Some(tx_send) = &self.send_to {
            let point = match self.conf.type_ {
                PointConfigType::Bool => {
                    Point::Bool(PointHlr::new(
                        self.tx_id, 
                        &self.conf.name, 
                        Bool(point.as_bool().value.0), 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::Int => {
                    Point::Int(PointHlr::new(
                        self.tx_id, 
                        &self.conf.name, 
                        point.as_int().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::Real => {
                    Point::Real(PointHlr::new(
                        self.tx_id, 
                        &self.conf.name, 
                        point.as_real().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::Double => {
                    Point::Double(PointHlr::new(
                        self.tx_id, 
                        &self.conf.name, 
                        point.as_double().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::String => {
                    Point::String(PointHlr::new(
                        self.tx_id, 
                        &self.conf.name, 
                        point.as_string().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::Json => {
                    Point::String(PointHlr::new(
                        self.tx_id, 
                        &self.conf.name, 
                        point.as_string().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
            };
            match tx_send.send(point.clone()) {
                Ok(_) => {
                    trace!("{}.out | Point sent: {:#?}", self.id, point);
                }
                Err(err) => {
                    error!("{}.out | Send error: {:#?}\n\t point: {:#?}", self.id, err, point);
                }
            };
        }
    }
}
//
//
impl FnIn for FnPoint {}
//
//
impl FnOut for FnPoint {
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
        if let Some(input) = &self.input {
            inputs.append(&mut input.borrow().inputs());
        }
        if let Some(changes_only) = &self.changes_only {
            inputs.append(&mut changes_only.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<Point, String> {
        match &self.input {
            Some(input) => {
                let enable = match &self.enable {
                    Some(enable) => match enable.borrow_mut().out() {
                        FnResult::Ok(enable) => enable.to_bool().as_bool().value.0,
                        FnResult::None => return FnResult::None,
                        FnResult::Err(err) => return FnResult::Err(err),
                    },
                    None => true,
                };
                let changes_only = match &self.changes_only {
                    Some(changes_only) => match changes_only.borrow_mut().out() {
                        FnResult::Ok(changes_only) => changes_only.to_bool().as_bool().value.0,
                        FnResult::None => return FnResult::None,
                        FnResult::Err(err) => return FnResult::Err(err),
                    }
                    None => false,
                };
                let input = input.borrow_mut().out();
                trace!("{}.out | input: {:?}", self.id, input);
                match input {
                    FnResult::Ok(point) => {
                        match &self.state {
                            Some(state) => {
                                if changes_only {
                                    if !point.cmp_value(state) {
                                        self.state = Some(point.clone());
                                        if enable {
                                            self.send(&point);
                                        }
                                    }
                                } else {
                                    self.state = Some(point.clone());
                                    if enable {
                                        self.send(&point);
                                    }
                                }
                            }
                            None => {
                                self.state = Some(point.clone());
                                if enable {
                                    self.send(&point);
                                }
                            }
                        }
                        FnResult::Ok(point)
                    }
                    FnResult::None => FnResult::None,
                    FnResult::Err(err) => FnResult::Err(err),
                }
            }
            None => panic!("{}.out | Input is not configured for the Point '{}'", self.id, self.conf.name),
        }
    }
    //
    fn reset(&mut self) {
        self.state = None;
        if let Some(input) = &self.input {
            input.borrow_mut().reset();
        }
        if let Some(changes_only) = &self.changes_only {
            changes_only.borrow_mut().reset();
        }
    }
}
//
//
impl FnInOut for FnPoint {}
///
/// Global static counter of FnPoint instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
