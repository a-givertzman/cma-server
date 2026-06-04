use concat_string::concat_string;
use sal_sync::services::{entity::{Point, PointHlr, Status, ToPoint}, task::functions::{FnConfPointType, FnConfig}, types::Bool};
use std::{fmt::Debug, sync::atomic::{AtomicUsize, Ordering}};
use crate::services::task::{EvalCycleRef, FnFlow, FnInOut};

use super::{FnIn, FnOut, FnKind, FnResult};
///
/// 
#[derive(Debug, Clone)]
pub struct FnInput {
    dbg: String,
    kind: FnKind,
    name: String,
    typ: PointType_,
    point: Option<Point>,
    #[allow(unused)]
    initial: Option<Point>,
    /// Фильтр входных евентов по статусу (из конфига), если задан, берем только евенты с указанным статусом, остальное игнорим
    status: Option<Status>,
    options_hash: String,
    /// Текущий номер вычислительного цикла, инкремнтируется в `TaskNodes` с каждым входом в `self.eval`
    cycle: EvalCycleRef,
    /// Локальное значение вычислительного цикла в котором было оновление
    updated_at: usize,
}
//
// 
impl FnInput {
    // pub fn new(parent: &str, name: impl Into<String>, initial: Option<PointType>, type_: FnConfPointType) -> Self {
    pub fn new(parent: impl Into<String>, tx_id: usize, conf: &mut FnConfig, cycle: &EvalCycleRef) -> Self {
        let dbg = format!("{}/FnInput{}", parent.into(), COUNT.fetch_add(1, Ordering::AcqRel));
        let (typ, initial) = match conf.type_.clone() {
            FnConfPointType::Bool => (PointType_::Bool, conf.options.default.as_ref().map_or(None, |d| match d.parse::<bool>() {
                Ok(d) => Some(d.to_point(tx_id, &conf.name)),
                Err(_) => panic!("{}.function | Error parsing Point default as Bool in: {:?}", dbg, conf),
            })),
            FnConfPointType::Int => (PointType_::Int, conf.options.default.as_ref().map_or(None, |d| match d.parse::<i64>() {
                Ok(d) => Some(d.to_point(tx_id, &conf.name)),
                Err(_) => panic!("{}.function | Error parsing Point default as Int in: {:?}", dbg, conf),
            })),
            FnConfPointType::Real => (PointType_::Real, conf.options.default.as_ref().map_or(None, |d| match d.parse::<f32>() {
                Ok(d) => Some(d.to_point(tx_id, &conf.name)),
                Err(_) => panic!("{}.function | Error parsing Point default as Real in: {:?}", dbg, conf),
            })),
            FnConfPointType::Double => (PointType_::Double, conf.options.default.as_ref().map_or(None, |d| match d.parse::<f64>() {
                Ok(d) => Some(d.to_point(tx_id, &conf.name)),
                Err(_) => panic!("{}.function | Error parsing Point default as Double in: {:?}", dbg, conf),
            })),
            FnConfPointType::String => (PointType_::String, conf.options.default.as_ref().map(|d| d.to_point(tx_id, &conf.name))),
            FnConfPointType::Any => (PointType_::Any, Some(false.to_point(tx_id, &conf.name))),
            FnConfPointType::Unknown => panic!("{}.function | Point type required", dbg),
        };
        log::trace!("{}.function | Input initial: {:?}", dbg, initial);
        Self {
            dbg,
            kind: FnKind::Input,
            name: conf.name.clone(),
            typ,
            point: initial.clone(), 
            initial,
            status: conf.options.status,
            options_hash: conf.options.hash(),
            cycle: cycle.clone(),
            updated_at: usize::MAX,
        }
    }
    ///
    /// ### Returns true if has new value
    fn is_new(&self) -> bool {
        self.updated_at == self.cycle.get()
    } 
}
//
// 
impl FnIn for FnInput {
    /// 
    /// Adds new value to the FnInput
    fn add(&mut self, point: &Point) {
        log::trace!("{}.add | value: {:?}", self.dbg, self.point);
        if let Some(status) = self.status {
            if point.status() != status {
                return
            }
        }
        let point = match self.typ {
            PointType_::Bool => {
                match point {
                    Point::Bool(_) => point.clone(),
                    Point::Int(p) => Point::Bool(PointHlr::new(p.txid, &p.name, Bool(p.value > 0), p.status, p.cot, p.timestamp)),
                    Point::Real(p) => Point::Bool(PointHlr::new(p.txid, &p.name, Bool(p.value > 0.0), p.status, p.cot, p.timestamp)),
                    Point::Double(p) => Point::Bool(PointHlr::new(p.txid, &p.name, Bool(p.value > 0.0), p.status, p.cot, p.timestamp)),
                    Point::String(_) | Point::Bytes(_) => {
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.type_(), self.typ);
                        return;
                    }
                }
            }
            PointType_::Int => {
                match point {
                    Point::Bool(p) => Point::Int(PointHlr::new(p.txid, &p.name, if p.value.0 {1} else {0}, p.status, p.cot, p.timestamp)),
                    Point::Int(p) => Point::Int(PointHlr::new(p.txid, &p.name, p.value, p.status, p.cot, p.timestamp)),
                    Point::Real(_) | Point::Double(_) | Point::String(_) | Point::Bytes(_) => {
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.type_(), self.typ);
                        return;
                    }
                }
            }
            PointType_::Real => {
                match point {
                    Point::Bool(p) => Point::Real(PointHlr::new(p.txid, &p.name, if p.value.0 {1.0} else {0.0}, p.status, p.cot, p.timestamp)),
                    Point::Int(p) => Point::Real(PointHlr::new(p.txid, &p.name, p.value as f32, p.status, p.cot, p.timestamp)),
                    Point::Real(_) => point.clone(),
                    Point::Double(_) | Point::String(_) | Point::Bytes(_) => {
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.type_(), self.typ);
                        return;
                    }
                }
            }
            PointType_::Double => {
                match point {
                    Point::Bool(p) => Point::Double(PointHlr::new(p.txid, &p.name, if p.value.0 {1.0} else {0.0}, p.status, p.cot, p.timestamp)),
                    Point::Int(p) => Point::Double(PointHlr::new(p.txid, &p.name, p.value as f64, p.status, p.cot, p.timestamp)),
                    Point::Real(p) => Point::Double(PointHlr::new(p.txid, &p.name, p.value as f64, p.status, p.cot, p.timestamp)),
                    Point::Double(_) => point.clone(),
                    Point::String(_) | Point::Bytes(_) => {
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.type_(), self.typ);
                        return;
                    }
                }
            }
            PointType_::String => {
                match point {
                    Point::Bool(p) => Point::String(PointHlr::new(p.txid, &p.name, p.value.to_string(), p.status, p.cot, p.timestamp)),
                    Point::Int(p) => Point::String(PointHlr::new(p.txid, &p.name, p.value.to_string(), p.status, p.cot, p.timestamp)),
                    Point::Real(p) => Point::String(PointHlr::new(p.txid, &p.name, p.value.to_string(), p.status, p.cot, p.timestamp)),
                    Point::Double(p) => Point::String(PointHlr::new(p.txid, &p.name, p.value.to_string(), p.status, p.cot, p.timestamp)),
                    Point::String(p) => Point::String(PointHlr::new(p.txid, &p.name, p.value.clone(), p.status, p.cot, p.timestamp)),
                    Point::Bytes(_) => {
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.type_(), self.typ);
                        return;
                    }
                }
            }
            PointType_::Any => point.clone(),
        };
        self.updated_at = self.cycle.get();
        self.point = Some(point)
    }
    ///
    /// Returns a hash of the `FnInput`
    fn hash(&self) -> String {
        self.options_hash.clone()
    }
}
//
// 
impl FnOut for FnInput {
    //
    fn id(&self) -> String {
        self.dbg.clone()
    }
    //
    fn kind(&self) -> FnKind {
        self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        vec![self.name.clone()]
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        log::trace!("{}.out | value: {:?}", self.dbg, &self.point);
        match self.point.as_ref() {
            Some(point) => if self.is_new() {
                FnResult::Ok(Some(FnFlow::New(point.clone())))
            } else {
                FnResult::Ok(Some(FnFlow::Old(point.clone())))
            },
            None => FnResult::Err(concat_string!(self.dbg, ".out | Not initialized")),
        }
    }
    //
    fn reset(&mut self) {
        self.point = self.initial.clone();
    }
}
//
// 
impl FnInOut for FnInput {}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// ### Локальные типы Point
/// - Соответствуют FnConfPointType, но без Unknown
#[derive(Debug, Clone, Copy)]
enum PointType_ {
    Bool,
    Int,
    Real,
    Double,
    String,
    Any,
}
