use std::sync::{mpsc::Sender, atomic::{AtomicUsize, Ordering}};
use log::{error, trace};
use crate::{
    conf::point_config::{point_config::PointConfig, point_config_type::PointConfigType},
    core_::{
        point::{point_hlr::PointHlr, point_tx_id::PointTxId, point::Point}, 
        types::{bool::Bool, fn_in_out_ref::FnInOutRef}
    }, 
    services::task::nested_function::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult},
};
///
/// Function | Used for export Point from Task service to another service
///  - Poiont will be sent to the queue only if:
///     - [enable] 
///         - if specified and is true (or [enable] > 0)
///         - if not specified - default is true
///     - send-to - is specified
///  - if point conf is not specified - input Point will be sent
///  - Returns input Point
/// 
/// Example
/// 
/// ```yaml
/// fn Export:
///     enable: const bool true         # optional, default true
///     send-to: /AppTest/MultiQueue.in-queue
///     conf point Point.Name:          # full name will be: /App/Task/Point.Name
///         type: 'Bool'
///     input: point string /AppTest/Exit
/// ```
#[derive(Debug)]
pub struct FnExport {
    id: String,
    tx_id: usize,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    conf: Option<PointConfig>,
    input: FnInOutRef,
    tx_send: Option<Sender<Point>>,
}
//
//
impl FnExport {
    ///
    /// Creates new instance of the FnExport
    /// - parent - the name of the parent entitie
    /// - enable - boolean (numeric) input enables the export if true (> 0)
    /// - conf - the configuration of the Point to be prodused, if None - input Point will be sent
    /// - input - incoming points
    /// - send-to - destination queue
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, conf: Option<PointConfig>, input: FnInOutRef, send: Option<Sender<Point>>) -> Self {
        let self_id = format!("{}/FnExport{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            id: self_id.clone(),
            tx_id: PointTxId::from_str(&self_id),
            kind: FnKind::Fn,
            enable,
            conf,
            input,
            tx_send: send,
        }
    }
    ///
    /// Sending Point to the external service if 'send-to' specified
    fn send(&self, point: Point) {
        if let Some(tx_send) = &self.tx_send {
            let (type_, name) = match &self.conf {
                Some(conf) => (conf.type_.clone(), conf.name.clone()),
                None => (point.type_(), point.name()),
            };
            let point = match type_ {
                PointConfigType::Bool => {
                    Point::Bool(PointHlr::new(
                        self.tx_id, 
                        &name, 
                        Bool(point.as_bool().value.0), 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::Int => {
                    Point::Int(PointHlr::new(
                        self.tx_id, 
                        &name, 
                        point.as_int().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::Real => {
                    Point::Real(PointHlr::new(
                        self.tx_id, 
                        &name, 
                        point.as_real().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::Double => {
                    Point::Double(PointHlr::new(
                        self.tx_id, 
                        &name, 
                        point.as_double().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::String => {
                    Point::String(PointHlr::new(
                        self.tx_id, 
                        &name, 
                        point.as_string().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfigType::Json => {
                    Point::String(PointHlr::new(
                        self.tx_id, 
                        &name, 
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
impl FnIn for FnExport {}
//
//
impl FnOut for FnExport {
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
    fn out(&mut self) -> FnResult<Point, String> {
        let enable = match &self.enable {
            Some(enable) => match enable.borrow_mut().out() {
                FnResult::Ok(enable) => enable.to_bool().as_bool().value.0,
                FnResult::None => return FnResult::None,
                FnResult::Err(err) => return FnResult::Err(err),
            },
            None => true,
        };
        let input = self.input.borrow_mut().out();
        trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                if enable {
                    self.send(input.clone());
                }
                FnResult::Ok(input)
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    //
    fn reset(&mut self) {
        if let Some(enable) = &self.enable {
            enable.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
    }
}
//
//
impl FnInOut for FnExport {}
///
/// Global static counter of FnExport instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
