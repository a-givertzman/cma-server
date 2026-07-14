use sal_core::error::Error;
use sal_sync::{services::{entity::{Point, PointConf, PointType, PointHlr, PointTxId}, types::Bool}, sync::channel::Sender};
use std::sync::{atomic::{AtomicUsize, Ordering}};
use crate::{
    domain::FnOutRef, services::task::{FnFlow, FnKind, FnOut, FnResult},
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
    txid: usize,
    kind: FnKind,
    conf: PointConf,
    enable: Option<FnOutRef>,
    changes_only: Option<FnOutRef>,
    input: Option<FnOutRef>,
    send_to: Option<Sender<Point>>,
    state: Option<Point>,
    id: String,
}
//
//
impl FnPoint {
    ///
    /// Creates new instance of the FnPoint
    /// - id - just for proper debugging
    /// - input - incoming points
    /// - if [changes-only] is specified and true - changes only will be sent, default false (sending all points)
    pub fn new(parent: impl Into<String>, conf: PointConf, enable: Option<FnOutRef>, changes_only: Option<FnOutRef>, input: Option<FnOutRef>, send_to: Option<Sender<Point>>) -> Result<Self, Error> {
        let id = format!("{}/FnPoint{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            conf,
            enable,
            changes_only,
            input,
            send_to,
            state: None,
            id,
        })
    }
    ///
    /// 
    fn send(&self, point: &Point) {
        if let Some(tx_send) = &self.send_to {
            let point = match self.conf.type_ {
                PointType::Bool => {
                    Point::Bool(PointHlr::new(
                        self.txid, 
                        &self.conf.name, 
                        Bool(point.as_bool().value.0), 
                        point.status(), 
                        point.cot(), 
                        point.ts(),
                    ))
                }
                PointType::Int => {
                    Point::Int(PointHlr::new(
                        self.txid, 
                        &self.conf.name, 
                        point.as_int().value, 
                        point.status(), 
                        point.cot(), 
                        point.ts(),
                    ))
                }
                PointType::Real => {
                    Point::Real(PointHlr::new(
                        self.txid, 
                        &self.conf.name, 
                        point.as_real().value, 
                        point.status(), 
                        point.cot(), 
                        point.ts(),
                    ))
                }
                PointType::Double => {
                    Point::Double(PointHlr::new(
                        self.txid, 
                        &self.conf.name, 
                        point.as_double().value, 
                        point.status(), 
                        point.cot(), 
                        point.ts(),
                    ))
                }
                PointType::String => {
                    Point::String(PointHlr::new(
                        self.txid, 
                        &self.conf.name, 
                        point.as_string().value, 
                        point.status(), 
                        point.cot(), 
                        point.ts(),
                    ))
                }
                PointType::Bytes => {
                    Point::Bytes(PointHlr::new(
                        self.txid, 
                        &self.conf.name, 
                        point.as_bytes().value, 
                        point.status(), 
                        point.cot(), 
                        point.ts(),
                    ))
                }
                PointType::Json => {
                    Point::String(PointHlr::new(
                        self.txid, 
                        &self.conf.name, 
                        point.as_string().value, 
                        point.status(), 
                        point.cot(), 
                        point.ts(),
                    ))
                }
            };
            match tx_send.send(point.clone()) {
                Ok(_) => {
                    log::trace!("{}.out | Point sent: {:#?}", self.id, point);
                }
                Err(err) => {
                    log::error!("{}.out | Send error: {:#?}\n\t point: {:#?}", self.id, err, point);
                }
            };
        }
    }
}
//
//
impl FnOut for FnPoint {
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
        if let Some(input) = &self.input {
            inputs.append(&mut input.borrow().inputs());
        }
        if let Some(changes_only) = &self.changes_only {
            inputs.append(&mut changes_only.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        unimplemented!()
        // let mut flow = FlowContext::new();
        // match &self.input {
        //     Some(input) => {
        //         let enable = match &self.enable {
        //             Some(enable) => match enable.borrow_mut().out() {
        //                 FnResult::Ok(enable) => enable.to_bool().as_bool().value.0,
        //                 FnResult::None => return FnResult::None,
        //                 FnResult::Err(err) => return FnResult::Err(err),
        //             },
        //             None => true,
        //         };
        //         let changes_only = match &self.changes_only {
        //             Some(changes_only) => match changes_only.borrow_mut().out() {
        //                 FnResult::Ok(changes_only) => changes_only.to_bool().as_bool().value.0,
        //                 FnResult::None => return FnResult::None,
        //                 FnResult::Err(err) => return FnResult::Err(err),
        //             }
        //             None => false,
        //         };
        //         let input = input.borrow_mut().out();
        //         log::trace!("{}.out | input: {:?}", self.id, input);
        //         match input {
        //             FnResult::Ok(point) => {
        //                 match &self.state {
        //                     Some(state) => {
        //                         if changes_only {
        //                             if !point.cmp_value(state) {
        //                                 self.state = Some(point.clone());
        //                                 if enable {
        //                                     self.send(&point);
        //                                 }
        //                             }
        //                         } else {
        //                             self.state = Some(point.clone());
        //                             if enable {
        //                                 self.send(&point);
        //                             }
        //                         }
        //                     }
        //                     None => {
        //                         self.state = Some(point.clone());
        //                         if enable {
        //                             self.send(&point);
        //                         }
        //                     }
        //                 }
        //                 FnResult::Ok(point)
        //             }
        //             FnResult::None => FnResult::None,
        //             FnResult::Err(err) => FnResult::Err(err),
        //         }
        //     }
        //     None => panic!("{}.out | Input is not configured for the Point '{}'", self.id, self.conf.name),
        // }
    }
    //
    fn hard_reset(&mut self) {
        self.state = None;
        if let Some(input) = &self.input {
            input.borrow_mut().hard_reset();
        }
        if let Some(changes_only) = &self.changes_only {
            changes_only.borrow_mut().hard_reset();
        }
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnPoint instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
