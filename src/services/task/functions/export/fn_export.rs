use sal_sync::{services::{entity::{Point, PointConf, PointConfType, PointHlr, PointTxId}, types::Bool}, sync::channel::Sender};
use std::sync::{atomic::{AtomicUsize, Ordering}};
use crate::{
    domain::FnOutRef, 
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | Used to export Point from Task service to another service
///  
/// - Point will be sent to the queue only if:
///     - the incoming data flow is explicitly `New`
///     - `enable` (handled via `FnEnable` decorator)
///         - if specified and is true (or `enable` > 0)
///         - if not specified - default is true
///     - `send-to` is specified
/// - If point conf is not specified - input Point will be sent as is.
/// - Returns the input Point wrapped in `FnFlow`.
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
    txid: usize,
    kind: FnKind,
    conf: Option<PointConf>,
    input: FnOutRef,
    tx_send: Option<Sender<Point>>,
}
//
impl FnExport {
    ///
    /// Creates new instance of FnExport
    /// - `parent` - the name of the parent entity
    /// - `conf` - the configuration of the Point to be produced. If None, input Point is sent
    /// - `input` - the incoming eval reference
    /// - `send` - the destination queue sender
    pub fn new(parent: impl Into<String>, conf: Option<PointConf>, input: FnOutRef, send: Option<Sender<Point>>) -> Self {
        let self_id = format!("{}/FnExport{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            id: self_id.clone(),
            txid: PointTxId::from_str(&self_id),
            kind: FnKind::Fn,
            conf,
            input,
            tx_send: send,
        }
    }
    ///
    /// Sends Point to the external service if 'send-to' specified
    /// - `point` will be renamed into `self.conf.name`
    fn send(&self, point: Point) {
        if let Some(tx_send) = &self.tx_send {
            let (type_, name) = match &self.conf {
                Some(conf) => (conf.type_.clone(), conf.name.clone()),
                None => (point.type_(), point.name()),
            };
            let point = match type_ {
                PointConfType::Bool => {
                    Point::Bool(PointHlr::new(
                        self.txid, 
                        &name, 
                        Bool(point.as_bool().value.0), 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfType::Int => {
                    Point::Int(PointHlr::new(
                        self.txid, 
                        &name, 
                        point.as_int().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfType::Real => {
                    Point::Real(PointHlr::new(
                        self.txid, 
                        &name, 
                        point.as_real().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfType::Double => {
                    Point::Double(PointHlr::new(
                        self.txid, 
                        &name, 
                        point.as_double().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfType::String => {
                    Point::String(PointHlr::new(
                        self.txid, 
                        &name, 
                        point.as_string().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfType::Bytes => {
                    Point::Bytes(PointHlr::new(
                        self.txid, 
                        &name, 
                        point.as_bytes().value, 
                        point.status(), 
                        point.cot(), 
                        point.timestamp(),
                    ))
                }
                PointConfType::Json => {
                    Point::String(PointHlr::new(
                        self.txid, 
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
impl FnOut for FnExport {
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
        self.input.borrow().inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(val) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:?}", self.id, val);
        if flow.is_new() {
            self.send(val.clone());
        }
        flow.wrap(val)
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnExport instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
