mod fn_eval_once {
use sal_sync::services::entity::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{
        EvalCycle, FnFlow, FnKind, FnOut, FnResult
    },
};
#[derive(Debug)]
pub struct FnEvalOnce {
    id: String,
    cycle: usize,
    eval_cycle: EvalCycle,
    input: FnOutRef,
    state: FnResult<FnFlow, String>,
}
impl FnEvalOnce {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, eval_cycle: EvalCycle, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnEvalOnce{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            cycle: usize::MAX,
            eval_cycle,
            input,
            state: Ok(None),
        }
    }
}
impl FnOut for FnEvalOnce {
    fn id(&self) -> String {
        self.input.borrow().id()
    }
    fn kind(&self) -> FnKind {
        self.input.borrow().kind()
    }
    fn inputs(&self) -> Vec<String> {
        self.input.borrow().inputs()
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let eval_cycle = self.eval_cycle.get();
        if self.eval_cycle.get() == self.cycle {
            return self.state.clone();
        }
        self.cycle = eval_cycle;
        match self.input.borrow_mut().out() {
            Ok(Some(v)) => {
                self.state = FnResult::Ok(Some(v.clone()));
                FnResult::Ok(Some(v))
            }
            Ok(None) => {
                self.state = Ok(None);
                Ok(None)
            }
            Err(err) => {
                let err = FnResult::Err(format!("{}.out | Error: {}", self.id, err));
                self.state = err.clone();
                err
            }
        }
    }
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod functions {
mod core {
mod fn_change {
use crate::{domain::FnOutRef, services::task::{FnFlow, FnKind, FnOut, FnResult}};
use sal_sync::services::entity::Point;
use std::fmt::Debug;
#[derive(Debug)]
pub struct FnChange {
    input: FnOutRef,
    last_val: Option<Point>,
}
impl FnChange {
    pub fn new(inp: FnOutRef) -> Self {
        Self { input: inp, last_val: None }
    }
}
impl FnOut for FnChange {
    fn id(&self) -> String { self.input.borrow().id() }
    fn kind(&self) -> FnKind { self.input.borrow().kind() }
    fn inputs(&self) -> Vec<String> { self.input.borrow().inputs() }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let result = self.input.borrow_mut().out()?;
        match result {
            Some(FnFlow::New(point)) | Some(FnFlow::Old(point)) => {
                let is_changed = match &self.last_val {
                    Some(last) => {
                        last.value() != point.value() || last.status() != point.status()
                    },
                    None => true,
                };
                if is_changed {
                    self.last_val = Some(point.clone());
                    Ok(Some(FnFlow::New(point)))
                } else {
                    Ok(Some(FnFlow::Old(point)))
                }
            }
            other => Ok(other),
        }
    }
    fn reset(&mut self) {
        self.last_val = None;
        self.input.borrow_mut().reset();
    }
}
}
mod fn_const {
use std::sync::atomic::{Ordering, AtomicUsize};
use sal_sync::services::entity::Point;
use crate::services::task::FnFlow;
use super::{FnOut, FnKind, FnResult};
#[derive(Debug, Clone)]
pub struct FnConst {
    id: String,
    kind: FnKind,
    point: Point,
}
impl FnConst {
    pub fn new(parent: &str, value: Point) -> Self {
        Self {
            id: format!("{}/FnConst{}", parent, COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Input,
            point: value
        }
    }
}
impl FnOut for FnConst {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        vec![]
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        log::trace!("{}.out | value: {:?}", self.id, &self.point);
        Ok(Some(FnFlow::Old(self.point.clone())))
    }
    fn reset(&mut self) {}
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_enable {
use sal_sync::services::entity::Point;
use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow}};
use super::{FnOut, FnKind, FnResult};
#[derive(Debug, Clone)]
pub struct FnEnable<T: FnOut> {
    origin: T,
    mode: FnEnableMode,
    enable: FnOutRef,
    prev_en: bool,
    last_val: Option<Point>,
}
impl<T: FnOut> FnEnable<T> {
    pub fn new(origin: T, mode: FnEnableMode, enable: FnOutRef) -> Self {
        Self {
            origin,
            mode,
            enable,
            prev_en: false,
            last_val: None,
        }
    }
}
impl<T: FnOut> FnOut for FnEnable<T> {
    fn id(&self) -> String {
        self.origin.id()
    }
    fn kind(&self) -> FnKind {
        self.origin.kind()
    }
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        inputs.append(&mut self.enable.borrow().inputs());
        inputs.append(&mut self.origin.inputs());
        inputs
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let en = match self.enable.borrow_mut().out()? {
            Some(en) => en.into_value().to_bool().as_bool().value.0,
            None => self.prev_en,
        };
        let rising_edge = !self.prev_en && en;
        let falling_edge = self.prev_en && !en;
        self.prev_en = en;
        match self.mode {
            FnEnableMode::Cold => {
                if en {
                    if rising_edge {
                        self.origin.reset();
                    }
                    let mut flow = FlowContext::new();
                    let Some(val) = flow.map(self.origin.out())? else { return Ok(None) };
                    self.last_val = Some(val.clone());
                    if rising_edge {
                        return Ok(Some(FnFlow::New(val)));
                    }
                    flow.wrap(val)
                } else {
                    if falling_edge {
                        self.origin.reset();
                        self.last_val = None;
                    }
                    Ok(None)
                }
            }
            FnEnableMode::Warm => {
                let mut flow = FlowContext::new();
                let val = flow.map(self.origin.out())?;
                if !en {
                    return Ok(self.last_val.clone().map(FnFlow::Old));
                }
                if let Some(val) = val {
                    self.last_val = Some(val.clone());
                    if rising_edge {
                        return Ok(Some(FnFlow::New(val)));
                    }
                    flow.wrap(val)
                } else {
                    Ok(None)
                }
            }
        }
    }
    fn reset(&mut self) {
        self.prev_en = false;
        self.last_val = None;
        self.enable.borrow_mut().reset();
        self.origin.reset();
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnEnableMode {
    Cold,
    Warm,
}
}
mod fn_flow {
use std::fmt::{Debug, Display};
use sal_sync::services::entity::Point;
use crate::services::task::FnResult;
#[derive(Debug, Clone)]
pub enum FnFlow {
    New(Point),
    Old(Point),
}
impl FnFlow {
    pub fn value(&self) -> &Point {
        match self {
            FnFlow::New(p) => p,
            FnFlow::Old(p) => p,
        }
    }
    pub fn into_value(self) -> Point {
        match self {
            FnFlow::New(p) => p,
            FnFlow::Old(p) => p,
        }
    }
    pub fn is_new(&self) -> bool {
        match self {
            FnFlow::New(_) => true,
            FnFlow::Old(_) => false,
        }
    }
}
pub struct FlowContext {
    is_new: bool,
}
impl FlowContext {
    pub fn new() -> Self {
        Self {
            is_new: false,
        }
    }
    pub fn ignore(&self, v: FnResult<FnFlow, String>) -> FnResult<Point, String> {
        let Some(flow) = v? else { return Ok(None) };
        Ok(Some(flow.into_value()))
    }
    pub fn map(&mut self, v: FnResult<FnFlow, String>) -> FnResult<Point, String> {
        let Some(flow) = v? else { return Ok(None) };
        self.is_new |= flow.is_new();
        Ok(Some(flow.into_value()))
    }
    pub fn wrap(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(match self.is_new {
            true => FnFlow::New(p),
            false => FnFlow::Old(p)
        }))
    }
    pub fn wrap_new(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(FnFlow::New(p)))
    }
    pub fn wrap_old(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(FnFlow::Old(p)))
    }
    pub fn is_new(&self) -> bool {
        self.is_new
    }
    pub fn is_old(&self) -> bool {
        !self.is_new
    }
}
impl Display for FlowContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_new {
            write!(f, "Flow::New")
        } else {
            write!(f, "Flow::Old")
        }
    }
}
//
impl Debug for FlowContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
}
mod fn_in_out {
use sal_sync::services::entity::Point;
use crate::services::task::FnFlow;
use super::{FnKind, FnResult};
pub trait FnIn: std::fmt::Debug {
    fn add(&mut self, point: &Point);
    fn hash(&self) -> String;
}
pub trait FnOut: std::fmt::Debug {
    fn id(&self) -> String;
    fn kind(&self) -> FnKind;
    fn inputs(&self) -> Vec<String>;
    fn out(&mut self) -> FnResult<FnFlow, String>;
    fn reset(&mut self);
}
pub trait FnInOut: FnIn + FnOut {}}
mod fn_input {
use concat_string::concat_string;
use sal_sync::services::{entity::{Point, PointHlr, Status, ToPoint}, task::functions::{FnConfPointType, FnConfig}, types::Bool};
use std::{fmt::Debug, sync::atomic::{AtomicUsize, Ordering}};
use crate::services::task::{EvalCycle, FnFlow, FnInOut};
use super::{FnIn, FnOut, FnKind, FnResult};
#[derive(Debug, Clone)]
pub struct FnInput {
    dbg: String,
    kind: FnKind,
    name: String,
    typ: PointType_,
    point: Option<Point>,
    #[allow(unused)]
    initial: Option<Point>,
    status: Option<Status>,
    options_hash: String,
    cycle: EvalCycle,
    updated_at: usize,
}
impl FnInput {
    pub fn new(parent: impl Into<String>, tx_id: usize, conf: &mut FnConfig, cycle: &EvalCycle) -> Self {
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
    fn is_new(&self) -> bool {
        self.updated_at == self.cycle.get()
    }
}
impl FnIn for FnInput {
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
    fn hash(&self) -> String {
        self.options_hash.clone()
    }
}
impl FnOut for FnInput {
    fn id(&self) -> String {
        self.dbg.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        vec![self.name.clone()]
    }
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
    fn reset(&mut self) {
        self.point = self.initial.clone();
    }
}
impl FnInOut for FnInput {}
static COUNT: AtomicUsize = AtomicUsize::new(1);
#[derive(Debug, Clone, Copy)]
enum PointType_ {
    Bool,
    Int,
    Real,
    Double,
    String,
    Any,
}
}
mod fn_kind {
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnKind {
    Input,
    Var,
    Fn,
}}
mod fn_result {
pub type FnResult<T, E> = Result<Option<T>, E>;
}
mod fn_var {
use std::sync::atomic::{Ordering, AtomicUsize};
use crate::{domain::FnOutRef, services::task::FnFlow};
use super::{FnOut, FnKind, FnResult};
#[derive(Debug, Clone)]
pub struct FnVar {
    id: String,
    kind: FnKind,
    input: FnOutRef,
}
impl FnVar {
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnVar{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Var,
            input,
        }
    }
}
impl FnOut for FnVar {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        self.input.borrow().inputs()
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        log::trace!("{}.eval | evaluating...", self.id);
        let value = self.input.borrow_mut().out();
        log::trace!("{}.out | value: {:?}", self.id, value);
        value
    }
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
pub use fn_change::*;
pub use fn_const::*;
pub use fn_enable::*;
pub use fn_flow::*;
pub use fn_in_out::*;
pub use fn_input::*;
pub use fn_kind::*;
pub use fn_result::*;
pub use fn_var::*;
}
mod fn_builder {
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{LinkName, Services, conf::ConfDuration, entity::{Name, Point, ToPoint}, task::functions::{FnConfKind, FnConfPointType, FnConfig}};
use std::{cell::RefCell, rc::Rc, str::FromStr, sync::Arc};
use indexmap::IndexMap;
use crate::{
    domain::FnOutRef,
    services::task::{
        FnAcc, FnAverage, FnConst, FnCount, FnDebug, FnEnable, FnHold, FnInput, FnIsChangedValue, FnMax, FnMin, FnPiecewiseLineApprox, FnPointId, FnRecOpCycleMetric, FnTimer, FnTimerOffDelay, FnTimerOnDelay, FnToBool, FnToDouble, FnVar, PiecewiseLinear, SqlMetric, functions::{
            comp::{FnEq, FnGe, FnGt, FnLe, FnLt, FnNe}, conversion::{FnToInt, FnToReal, FnToString},
            edge_detection::{FnFallingEdge, FnRisingEdge}, export::{FnExport, FnPoint, FnToApiQueue},
            filter::{FnSelect, FnSmooth, FnThreshold}, functions::Functions, io::FnRetain,
            ops::{FnAdd, FnBitAnd, FnBitOr, FnBitXor, FnDiv, FnMul, FnNot, FnPow, FnSub}, plot::FnPlot,
        }, task_nodes::TaskNodes
    },
};
pub struct FnBuilder {}
impl FnBuilder {
    pub fn new(parent: &Name, tx_id: usize, conf: &mut FnConfKind, task_nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnOutRef, Error> {
        Self::function(parent, tx_id, "", conf, task_nodes, services)
    }
    fn get_input_config(txid: usize, parent: &Name, name: &str, conf: &mut FnConfig, task_nodes: &mut TaskNodes, services: &Arc<Services>) -> Result<Option<FnOutRef>, Error> {
        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
        Ok(match input_conf {
            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())?),
            None => None,
        })
    }
    fn function(parent: &Name, txid: usize, input_name: &str, conf: &mut FnConfKind, task_nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnOutRef, Error> {
        let dbg = Dbg::new(parent, "FnBuilder");
        let error = Error::new(&dbg, "function");
        match conf {
            FnConfKind::Fn(conf) => {
                log::trace!("{}.function | Fn {:?}: {:?}...", dbg, input_name, conf.name.clone());
                let c = conf.name.clone();
                let fn_name= c.clone();
                let fn_name = fn_name.as_str();
                drop(c);
                let fn_name = Functions::from_str(fn_name).unwrap();
                log::trace!("{}.function | Fn '{}' detected", dbg, fn_name.name());
                log::trace!("{}.function | fn_conf: {:?}: {:#?}", dbg, conf.name, conf);
                match fn_name {
                    Functions::Count => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnCount | Can't get 'initial'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnCount | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnCount::new(parent, initial, input),
                        )))
                    }
                    Functions::Add => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAdd | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnAdd::new(parent, inputs)
                        )))
                    }
                    Functions::Timer => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnTimer | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnTimer | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let conf = conf.inputs.get_mut(name).unwrap();
                        let input = Self::function(parent, txid, name, conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnTimer::new(parent, enable, initial, input, true)
                        )))
                    }
                    Functions::TimerOnDelay => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let delay = {
                            let name = "delay";
                            let param = conf.param(name).ok_or(error.err(format!("FnTimerOnDelay | Can't get '{name}'")))?;
                            let delay = param.as_param().conf;
                            let delay = delay.as_str()
                                .ok_or(error.err(format!("FnTimerOnDelay | Wrong conf in '{name}': '{:?}'", param)))?;
                            ConfDuration::from_str(delay)
                                .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Wrong conf in '{name}': {:?}", param), err))?
                        };
                        let name = "input";
                        let conf = conf.inputs.get_mut(name).unwrap();
                        let input = Self::function(parent, txid, name, conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnTimerOnDelay::new(parent, enable, delay, input)
                        )))
                    }
                    Functions::TimerOffDelay => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let delay = {
                            let name = "delay";
                            let param = conf.param(name).ok_or(error.err(format!("FnTimerOffDelay | Can't get '{name}'")))?;
                            let delay = param.as_param().conf;
                            let delay = delay.as_str()
                                .ok_or(error.err(format!("FnTimerOffDelay | Wrong conf in '{name}': '{:?}'", param)))?;
                            ConfDuration::from_str(delay)
                                .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Wrong conf in '{name}': {:?}", param), err))?
                        };
                        let name = "input";
                        let conf = conf.inputs.get_mut(name).unwrap();
                        let input = Self::function(parent, txid, name, conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnTimerOffDelay::new(parent, enable, delay, input)
                        )))
                    }
                    Functions::ToApiQueue => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes ,services.clone())
                            .map_err(|err| error.pass_with(format!("FnToApiQueue | Can't get '{name}'"), err))?;
                        let Some(queue_name) = conf.param("queue").map(|v| v.as_param()) else {
                            return Err(error.err(format!("FnToApiQueue | Parameter 'queue' is missed in '{}'", conf.name)));
                        };
                        let queue_name = queue_name.conf.as_str().unwrap();
                        let link_name = LinkName::from_str(queue_name).unwrap();
                        let send_queue = services.get_link(&link_name)
                            .map_err(|err| error.pass_with(format!("FnToApiQueue | Can't get link '{link_name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToApiQueue::new(parent, input, send_queue)
                        )))
                    }
                    Functions::Gt => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnGt | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnGt | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnGt::new(parent, input1, input2)
                        )))
                    }
                    Functions::Ge => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnGe | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnGe | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnGe::new(parent, input1, input2)
                        )))
                    }
                    Functions::Eq => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnEq | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnEq | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnEq::new(parent, input1, input2)
                        )))
                    }
                    Functions::Le => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnLe | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnLe | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnLe::new(parent, input1, input2)
                        )))
                    }
                    Functions::Lt => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnLt | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnLt | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnLt::new(parent, input1, input2)
                        )))
                    }
                    Functions::Ne => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnNe | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnNe | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnNe::new(parent, input1, input2)
                        )))
                    }
                    Functions::SqlMetric => {
                        Ok(Rc::new(RefCell::new(
                            SqlMetric::new(parent, conf, task_nodes, services)
                                .map_err(|err| error.pass_with(format!("Can't build SqlMetric"), err))?
                        )))
                    }
                    Functions::PointId => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnPointId | Can't get '{name}'"), err))?;
                        log::debug!("{}.functions | Functions::PointId | requesting points...", dbg);
                        let points = services.points(&parent.join())
                            .then(|points| points, |err| {
                                log::error!("{}.functions | Functions::PointId | Requesting points error: {:?}", dbg, err);
                                vec![]
                            });
                        Ok(Rc::new(RefCell::new(
                            FnPointId::new(parent, input, points)
                        )))
                    }
                    Functions::Debug => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnDebug | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnDebug::new(parent, inputs)
                        )))
                    }
                    Functions::Plot => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "x";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let x = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let mut inputs = IndexMap::new();
                        let mut conf_inputs: IndexMap<String, FnConfKind> = conf.inputs
                            .iter()
                            .filter(|(name, _)| ! ["enable", "x"].contains(&(name.as_str())))
                            .map(|(n, c)| (n.to_owned(), c.clone())).collect();
                        for (name, input_conf) in &mut conf_inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?;
                            inputs.insert(name.to_owned(), input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnPlot::new(parent, enable, x, inputs)
                        )))
                    }
                    Functions::ToBool => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToBool | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToBool::new(parent, input)
                        )))
                    }
                    Functions::ToInt => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToInt | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToInt::new(parent, input)
                        )))
                    }
                    Functions::ToReal => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToReal | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToReal::new(parent, input)
                        )))
                    }
                    Functions::ToDouble => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToDouble | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToDouble::new(parent, input)
                        )))
                    }
                    Functions::Export => {
                        let name = "input";
                        let input_conf = conf.input_conf(name)
                            .map_err(|err| error.pass_with(format!("FnExport | Can't get '{name}'"), err))?;
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnExport | Can't get '{name}'"), err))?;
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnExport | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "conf";
                        let point_conf = match conf.input_conf(name) {
                            Ok(FnConfKind::PointConf(p_conf)) => Some(p_conf.conf.clone()),
                            Ok(_) => return Err(error.err(format!("FnExport | Invalid Point config in '{name}'"))),
                            Err(_) => None,
                        };
                        let send_queue = match conf.param("send-to") {
                            Some(FnConfKind::Param(queue_name)) => {
                                let queue_name = queue_name.conf.as_str()
                                    .ok_or(error.err(format!("FnExport | Invalid conf in 'send-to', string expected")))?;
                                let link_name = LinkName::from_str(queue_name).unwrap();
                                services.get_link(&link_name).map_or(None, |send| Some(send))
                            }
                            Some(_) => return Err(error.err(format!("FnExport | Invalid conf in 'send-to'"))),
                            None => {
                                log::warn!("{}.function | FnExport | Parameter 'send-to' - missed in '{}'", dbg, conf.name);
                                None
                            },
                        };
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnExport::new(parent, point_conf, input, send_queue), task_nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnExport::new(parent, point_conf, input, send_queue))),
                        })
                    }
                    Functions::Select => {
                        let select = Self::get_input_config(txid, parent, "select", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnSelect | 'select' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'select'"), err))?;
                        let default = Self::get_input_config(txid, parent, "default", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'default'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnSelect | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnSelect::new(parent, default, input, select)
                        )))
                    }
                    Functions::RisingEdge => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnRisingEdge | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnRisingEdge | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnRisingEdge::new(parent, input)
                        )))
                    }
                    Functions::FallingEdge => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnFallingEdge | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnFallingEdge | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnFallingEdge::new(parent, input)
                        )))
                    }
                    Functions::Retain => {
                        let name = "default";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let default = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRetain | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let input = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRetain | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRetain | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "every-cycle";
                        let every_cycle = conf.param(name).map_or(false, |param| {
                            match param.as_param().conf.as_bool() {
                                Some(param) => param,
                                None => {
                                    log::warn!("{}.function | FnRetain | Illegal 'every_cycle' parameter value in '{:#?}'", dbg, conf);
                                    false
                                },
                            }
                        });
                        let Some(key) = conf.param("key").map(|v| v.as_param()) else {
                            return Err(error.err(format!("FnRetain | Parameter 'key' - missed in '{}'", conf.name)));
                        };
                        let key = key.conf.as_str().ok_or(error.err(format!("FnRetain | Parameter 'key' must be a string in '{}'", conf.name)))?;
                        let Some(retain_path) = services.retain().path else {
                            return Err(error.err(format!("FnRetain | Retain: path - missed in Application config")));
                        };
                        Ok(Rc::new(RefCell::new(
                            FnRetain::new(parent, retain_path, enable, every_cycle, key, default, input)
                        )))
                    }
                    Functions::Acc => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAcc | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnAcc | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnAcc::new(parent, initial, input),
                        )))
                    }
                    Functions::Mul => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnMul | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnMul | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnMul::new(parent, input1, input2)
                        )))
                    }
                    Functions::Div => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnDiv | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnDiv | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnDiv::new(parent, input1, input2)
                        )))
                    }
                    Functions::Sub => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnSub::new(parent, input1, input2)
                        )))
                    }
                    Functions::BitAnd => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitAnd | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitAnd::new(parent, inputs)
                        )))
                    }
                    Functions::BitOr => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitOr | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitOr::new(parent, inputs)
                        )))
                    }
                    Functions::BitXor => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitXor | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitXor::new(parent, inputs)
                        )))
                    }
                    Functions::Not => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnBitNot | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnNot::new(parent, input)
                        )))
                    }
                    Functions::Threshold => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'enable'"), err))?;
                        let threshold = Self::get_input_config(txid, parent, "threshold", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnThreshold | 'threshold' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'threshold'"), err))?;
                        let factor = Self::get_input_config(txid, parent, "factor", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'factor'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnThreshold | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(
                                FnThreshold::new(parent, threshold, factor, input),
                                task_nodes.enable_mode(),
                                enable
                            ))),
                            None => Rc::new(RefCell::new(FnThreshold::new(parent, threshold, factor, input))),
                        })
                    }
                    Functions::Smooth => {
                        let name = "factor";
                        let input_conf = conf.input_conf(name).unwrap();
                        let factor = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnSmooth | Can't get '{name}'"), err))?;
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnSmooth | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnSmooth::new(parent, factor, input)
                        )))
                    }
                    Functions::Average => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAverage | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "reset";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let reset = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnMax | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnAverage | Can't get '{name}'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnAverage::new(parent, reset, input), task_nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnAverage::new(parent, reset, input))),
                        })
                    }
                    Functions::Pow => {
                        let name = "input1";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input1 = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnPow | Can't get '{name}'"), err))?;
                        let name = "input2";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input2 = Self::function(parent, txid, name, input_conf, task_nodes, services)
                            .map_err(|err| error.pass_with(format!("FnPow | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnPow::new(parent, input1, input2)
                        )))
                    }
                    Functions::RecOpCycleMetric => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get 'reset'"), err))?;
                        let send_to = match conf.param("send-to") {
                            Some(queue_name) => {
                                let queue_name = match queue_name {
                                    FnConfKind::Param(queue_name) => queue_name.conf.as_str().unwrap(),
                                    _ => Err(error.err(format!("FnRecOpCycleMetric | Parameter 'send-to' - invalid type, string expected) '{:?}'", queue_name)))?,
                                };
                                let link_name = LinkName::from_str(queue_name).unwrap();
                                services.get_link(&link_name).map_or(None, |send| Some(send))
                            }
                            None => {
                                log::warn!("{}.function | FnRecOpCycleMetric | Parameter 'send-to' - missed in '{}'", dbg, conf.name);
                                None
                            },
                        };
                        let name = "op-cycle";
                        let op_cycle = Self::get_input_config(txid, parent, name, conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnRecOpCycleMetric | '{name}' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get '{name}'"), err))?;
                        let mut inputs = IndexMap::new();
                        let conf_inputs = conf.inputs.iter_mut().filter(|(name, _)| {
                            ! ["enable", "reset", "send-to", "conf", "op-cycle"].contains(&name.as_str())
                        });
                        for (name, input_conf) in conf_inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get '{name}'"), err))?;
                            inputs.insert(name.to_owned(), input);
                        }
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(
                                FnRecOpCycleMetric::new(parent, send_to, reset, op_cycle, inputs),
                                task_nodes.enable_mode(),
                                en,
                            ))),
                            None => Rc::new(RefCell::new(FnRecOpCycleMetric::new(parent, send_to, reset, op_cycle, inputs))),
                        })
                    }
                    Functions::Max => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnMax | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnMax::new(parent, reset, input), task_nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnMax::new(parent, reset, input))),
                        })
                    }
                    Functions::Min => {
                        let enable = Self::get_input_config(txid, parent, "enable", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(txid, parent, "reset", conf, task_nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnMin | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnMin::new(parent, reset, input), task_nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnMin::new(parent, reset, input))),
                        })
                    }
                    Functions::PiecewiseLineApprox => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnPiecewiseLineApprox | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnPiecewiseLineApprox | Can't get 'input'"), err))?;
                        log::trace!("{}.function | PiecewiseLineApprox | conf: {:#?}", dbg, conf);
                        let name = "piecewise";
                        let FnConfKind::Param(piecewise) = conf.param(name)
                            .ok_or(error.err(format!("FnPiecewiseLineApprox | Can't get '{name}'")))? else {
                                return Err(error.err(format!("FnPiecewiseLineApprox | Parameter 'piecewise' - has invalid type, expected map in '{}'", conf.name)));
                            };
                        let pieces = PiecewiseLinear::from_yaml(parent, &piecewise.conf)
                            .map_err(|err| error.pass_with(format!("FnPiecewiseLineApprox | Wrong conf in '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnPiecewiseLineApprox::new(parent, input, pieces)
                        )))
                    }
                    Functions::IsChangedValue => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &mut conf.inputs {
                            let input = Self::function(parent, txid, name, input_conf, task_nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnIsChangedValue | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnIsChangedValue::new(parent, inputs)
                        )))
                    }
                    Functions::Hold => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnHold | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnHold | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnHold::new(parent, input)
                        )))
                    }
                    Functions::ToString => {
                        let input = Self::get_input_config(txid, parent, "input", conf, task_nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnToString | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnToString | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToString::new(parent, input)
                        )))
                    }
                    _ => Err(error.err(format!("Unknown function name: {:?}", conf.name))),
                }
            }
            FnConfKind::Var(conf) => {
                let var_name = conf.name.clone();
                log::trace!("{}.function | Var: {:?}...", dbg, var_name);
                match conf.inputs.iter_mut().next() {
                    Some((input_conf_name, input_conf)) => {
                        let var = Self::fn_var(
                            var_name,
                            Self::function(parent, txid, input_conf_name, input_conf, task_nodes, services)
                                .map_err(|err| error.pass_with(format!("Var | Can't get '{input_conf_name}'"), err))?,
                        );
                        log::trace!("{}.function | Var: {:?}: {:?}", dbg, &conf.name, var.clone());
                        task_nodes.add_var(conf.name.clone(), var.clone())
                            .map_err(|err| error.pass_with(format!("Var | Can't add var '{}'", conf.name), err))?;
                        Ok(var)
                    }
                    None => {
                        let var = task_nodes.get_var(&var_name)
                            .ok_or(error.err(format!("Var {var_name} - is not declared")))?
                            .to_owned();
                        task_nodes.add_var_out(conf.name.clone())
                            .map_err(|err| error.pass_with(format!("Var | Can't add defined var '{}'", conf.name), err))?;
                        Ok(var)
                    }
                }
            }
            FnConfKind::Const(conf) => {
                let value = conf.name.trim().to_lowercase();
                let name = format!("const {:?} '{}'", conf.type_, value);
                log::trace!("{}.function | Const: {:?}...", dbg, name);
                let value = match conf.type_.clone() {
                    FnConfPointType::Bool => value.parse::<bool>().unwrap().to_point(txid, &name),
                    FnConfPointType::Int => value.parse::<i64>().unwrap().to_point(txid, &name),
                    FnConfPointType::Real => value.parse::<f32>().unwrap().to_point(txid, &name),
                    FnConfPointType::Double => value.parse::<f64>().unwrap().to_point(txid, &name),
                    FnConfPointType::String => value.to_point(txid, &name),
                    FnConfPointType::Any => Err(error.err(format!("Const '{name}': type 'any' - is not supported")))?,
                    FnConfPointType::Unknown => Err(error.err(format!("Const '{name}': type required")))?,
                };
                let fn_const = Self::fn_const(&name, value);
                log::trace!("{}.function | Const: {:?} - done", dbg, fn_const);
                Ok(fn_const)
            }
            FnConfKind::Point(conf) => {
                log::trace!("{}.function | Input (Point<{:?}>): {:?} ({:?})...", dbg, conf.type_, input_name, conf.name);
                let point_name = conf.name.clone();
                let input = task_nodes.add_input(
                    &point_name,
                    Rc::new(RefCell::new(
                        FnInput::new(&point_name, txid, conf, &task_nodes.cycle())
                    )),
                );
                log::trace!("{}.function | input (Point): {:?}", dbg, input);
                input
            }
            FnConfKind::PointConf(conf) => {
                let send_to = match conf.send_to.as_ref() {
                    Some(send_to) => {
                        let link_name = LinkName::from_str(send_to).unwrap();
                        Some(services.get_link(&link_name)
                            .map_err(|err| error.pass_with(format!("PointConf | Can't get link"), err))?)
                    }
                    None => None,
                };
                let enable = match conf.enable.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "enable", input_conf, task_nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'enable'"), err))?),
                    None => None,
                };
                let input = match conf.input.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "input", input_conf, task_nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'input'"), err))?),
                    None => None,
                };
                let changes_only = match conf.changes_only.as_mut() {
                    Some(input_conf) => Some(Self::function(parent, txid, "changes-only", input_conf, task_nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'input'"), err))?),
                    None => None,
                };
                Ok(Rc::new(RefCell::new(
                    FnPoint::new(parent, conf.conf.clone(), enable, changes_only, input, send_to),
                )))
            }
            FnConfKind::Param(conf) => {
                Err(error.err(format!("Param | Undefined variable or unknown custom parameters in the function conf: {:#?}", conf)))
            }
        }
    }
    fn fn_var(parent: impl Into<String>, input: FnOutRef,) -> FnOutRef {
        Rc::new(RefCell::new(
        FnVar::new(parent, input),
        ))
    }
    fn fn_const(parent: &str, value: Point) -> FnOutRef {
        Rc::new(RefCell::new(
        FnConst::new(parent, value)
        ))
    }
}
}
mod functions {
use std::str::FromStr;
#[derive(Debug)]
pub enum Functions {
    Input,
    Const,
    Var,
    Debug,
    Plot,
    Add,
    Count,
    Gt,
    Ge,
    Eq,
    Le,
    Lt,
    Ne,
    Timer,
    TimerOnDelay,
    TimerOffDelay,
    ToBool,
    ToInt,
    ToReal,
    ToDouble,
    ToString,
    ToApiQueue,
    ToMultiQueue,
    SqlMetric,
    PointId,
    Export,
    Select,
    RisingEdge,
    FallingEdge,
    Retain,
    Acc,
    Mul,
    Div,
    Sub,
    BitAnd,
    BitOr,
    BitXor,
    Not,
    Threshold,
    Smooth,
    Average,
    Pow,
    Max,
    Min,
    PiecewiseLineApprox,
    IsChangedValue,
    Hold,
    RecOpCycleMetric,
}
impl Functions {
    const INPUT                         : &'static str = "input";
    const CONST                         : &'static str = "const";
    const VAR                           : &'static str = "var";
    /// debuging functions
    const DEBUG                         : &'static str = "Debug";
    const PLOT                          : &'static str = "Plot";
    /// user defined functions
    const ADD                           : &'static str = "Add";
    const COUNT                         : &'static str = "Count";
    const GT                            : &'static str = "Gt";
    const GE                            : &'static str = "Ge";
    const EQ                            : &'static str = "Eq";
    const LE                            : &'static str = "Le";
    const LT                            : &'static str = "Lt";
    const NE                            : &'static str = "Ne";
    const TIMER                         : &'static str = "Timer";
    const TIMER_ON_DELAY                : &'static str = "TimerOnDelay";
    const TIMER_OFF_DELAY               : &'static str = "TimerOffDelay";
    const TO_API_QUEUE                  : &'static str = "ToApiQueue";
    const TO_MULTI_QUEUE                : &'static str = "ToMultiQueue";
    const SQL_METRIC                    : &'static str = "SqlMetric";
    const POINT_ID                      : &'static str = "PointId";
    const TO_BOOL                       : &'static str = "ToBool";
    const TO_INT                        : &'static str = "ToInt";
    const TO_REAL                       : &'static str = "ToReal";
    const TO_DOUBLE                     : &'static str = "ToDouble";
    const TO_STRING                     : &'static str = "ToString";
    const EXPORT                        : &'static str = "Export";
    const SELECT                        : &'static str = "Select";
    const RISING_EDGE                   : &'static str = "RisingEdge";
    const FALLING_EDGE                  : &'static str = "FallingEdge";
    const RETAIN                        : &'static str = "Retain";
    const ACC                           : &'static str = "Acc";
    const MUL                           : &'static str = "Mul";
    const DIV                           : &'static str = "Div";
    const SUB                           : &'static str = "Sub";
    const BIT_AND                       : &'static str = "BitAnd";
    const BIT_OR                        : &'static str = "BitOr";
    const BIT_XOR                       : &'static str = "BitXor";
    const NOT                           : &'static str = "Not";
    const THRESHOLD                     : &'static str = "Threshold";
    const SMOOTH                        : &'static str = "Smooth";
    const AVERAGE                       : &'static str = "Average";
    const POW                           : &'static str = "Pow";
    const MAX                           : &'static str = "Max";
    const MIN                           : &'static str = "Min";
    const PIECEWISE_LINE_APPROX         : &'static str = "PiecewiseLineApprox";
    const IS_CHANGED_VALUE              : &'static str = "IsChangedValue";
    const HOLD                          : &'static str = "Hold";
    const KEEP_VALID                    : &'static str = "KeepValid";
    const REC_OP_CYCLE_METRIC           : &'static str = "RecOpCycleMetric";
    ///
    /// Returns function name as string
    pub fn name(&self) -> &str {
        match self {
            Self::Add                   => Self::ADD,
            Self::Const                 => Self::CONST,
            Self::Count                 => Self::COUNT,
            Self::Gt                    => Self::GT,
            Self::Ge                    => Self::GE,
            Self::Eq                    => Self::EQ,
            Self::Le                    => Self::LE,
            Self::Lt                    => Self::LT,
            Self::Ne                    => Self::NE,
            Self::Input                 => Self::INPUT,
            Self::Timer                 => Self::TIMER,
            Self::TimerOnDelay          => Self::TIMER_ON_DELAY,
            Self::TimerOffDelay         => Self::TIMER_OFF_DELAY,
            Self::Var                   => Self::VAR,
            Self::ToApiQueue            => Self::TO_API_QUEUE,
            Self::ToMultiQueue          => Self::TO_MULTI_QUEUE,
            Self::SqlMetric             => Self::SQL_METRIC,
            Self::PointId               => Self::POINT_ID,
            Self::Debug                 => Self::DEBUG,
            Self::Plot                  => Self::PLOT,
            Self::ToBool                => Self::TO_BOOL,
            Self::ToInt                 => Self::TO_INT,
            Self::ToReal                => Self::TO_REAL,
            Self::ToDouble              => Self::TO_DOUBLE,
            Self::ToString              => Self::TO_STRING,
            Self::Export                => Self::EXPORT,
            Self::Select                => Self::SELECT,
            Self::RisingEdge            => Self::RISING_EDGE,
            Self::FallingEdge           => Self::FALLING_EDGE,
            Self::Retain                => Self::RETAIN,
            Self::Acc                   => Self::ACC,
            Self::Mul                   => Self::MUL,
            Self::Div                   => Self::DIV,
            Self::Sub                   => Self::SUB,
            Self::BitAnd                => Self::BIT_AND,
            Self::BitOr                 => Self::BIT_OR,
            Self::BitXor                => Self::BIT_XOR,
            Self::Not                   => Self::NOT,
            Self::Threshold             => Self::THRESHOLD,
            Self::Smooth                => Self::SMOOTH,
            Self::Average               => Self::AVERAGE,
            Self::Pow                   => Self::POW,
            Self::RecOpCycleMetric      => Self::REC_OP_CYCLE_METRIC,
            Self::Max                   => Self::MAX,
            Self::Min                   => Self::MIN,
            Self::PiecewiseLineApprox   => Self::PIECEWISE_LINE_APPROX,
            Self::IsChangedValue        => Self::IS_CHANGED_VALUE,
            Self::Hold             => Self::HOLD,
        }
    }
    ///
    /// Returns enum Function corresponding to the function name
    fn match_name(input: &str) -> Result<Functions, String> {
        match input {
            Self::ADD                       => Ok( Self::Add ),
            Self::CONST                     => Ok( Self::Const ),
            Self::COUNT                     => Ok( Self::Count ),
            Self::GT                        => Ok( Self::Gt ),
            Self::GE                        => Ok( Self::Ge ),
            Self::EQ                        => Ok( Self::Eq ),
            Self::LE                        => Ok( Self::Le ),
            Self::LT                        => Ok( Self::Lt ),
            Self::NE                        => Ok( Self::Ne ),
            Self::INPUT                     => Ok( Self::Input ),
            Self::TIMER                     => Ok( Self::Timer ),
            Self::TIMER_ON_DELAY            => Ok( Self::TimerOnDelay ),
            Self::TIMER_OFF_DELAY           => Ok( Self::TimerOffDelay ),
            Self::VAR                       => Ok( Self::Var ),
            Self::TO_API_QUEUE              => Ok( Self::ToApiQueue ),
            Self::TO_MULTI_QUEUE            => Ok( Self::ToMultiQueue ),
            Self::SQL_METRIC                => Ok( Self::SqlMetric ),
            Self::POINT_ID                  => Ok( Self::PointId ),
            Self::DEBUG                     => Ok( Self::Debug ),
            Self::PLOT                      => Ok( Self::Plot ),
            Self::TO_BOOL                   => Ok( Self::ToBool ),
            Self::TO_INT                    => Ok( Self::ToInt ),
            Self::TO_REAL                   => Ok( Self::ToReal ),
            Self::TO_DOUBLE                 => Ok( Self::ToDouble ),
            Self::TO_STRING                 => Ok( Self::ToString ),
            Self::EXPORT                    => Ok( Self::Export ),
            Self::SELECT                    => Ok( Self::Select ),
            Self::RISING_EDGE               => Ok( Self::RisingEdge ),
            Self::FALLING_EDGE              => Ok( Self::FallingEdge ),
            Self::RETAIN                    => Ok( Self::Retain ),
            Self::ACC                       => Ok( Self::Acc ),
            Self::MUL                       => Ok( Self::Mul ),
            Self::DIV                       => Ok( Self::Div ),
            Self::SUB                       => Ok( Self::Sub ),
            Self::BIT_AND                   => Ok( Self::BitAnd ),
            Self::BIT_OR                    => Ok( Self::BitOr ),
            Self::BIT_XOR                   => Ok( Self::BitXor ),
            Self::NOT                       => Ok( Self::Not ),
            Self::THRESHOLD                 => Ok( Self::Threshold ),
            Self::SMOOTH                    => Ok( Self::Smooth ),
            Self::AVERAGE                   => Ok( Self::Average ),
            Self::POW                       => Ok( Self::Pow ),
            Self::REC_OP_CYCLE_METRIC       => Ok( Self::RecOpCycleMetric ),
            Self::MAX                       => Ok( Self::Max ),
            Self::MIN                       => Ok( Self::Min ),
            Self::PIECEWISE_LINE_APPROX     => Ok( Self::PiecewiseLineApprox ),
            Self::IS_CHANGED_VALUE          => Ok( Self::IsChangedValue ),
            Self::HOLD | Self::KEEP_VALID   => Ok( Self::Hold ),
            _ => Err(format!("Functions.from_str | Unknown function name '{}'", &input)),
        }
    }
}
//
//
impl FromStr for Functions {
    type Err = String;
    fn from_str(input: &str) -> Result<Functions, String> {
        log::trace!("Functions.from_str | input: {}", input);
        Self::match_name(input)
    }
}
}
pub use application::*;
pub use common::*;
pub use comp::*;
pub use conversion::*;
pub use core::*;
pub use edge_detection::*;
pub use export::*;
pub use filter::*;
pub use import::*;
pub use io::*;
pub use ops::*;
pub use plot::*;
pub use sql::*;
pub use timers::*;
pub use fn_builder::*;
}
mod task_conf {
use indexmap::IndexMap;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::{Name, PointConf}, ConfSubscribe, task::functions::{FnConfKind, FnConfig}};
use std::{fs, time::Duration};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service Task operatingMetric:
///     cycle: 100 ms
///         in queue recv-queue:
///             max-length: 10000
///     metrics:
///         fn sqlUpdateMetric:
///             table: "TableName"
///             sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
///             initial: 123.456
///             inputs:
///                 input1:
///                     fn functionName:
///                         ...
///                 input2:
///                     fn SqlMetric:
///                         ...
#[derive(Debug, PartialEq, Clone)]
pub struct TaskConf {
    pub name: Name,
    pub cycle: Option<Duration>,
    pub rx: String,
    pub rx_max_length: i64,
    pub subscribe: ConfSubscribe,
    pub nodes: IndexMap<String, FnConfKind>,
    pub vars: Vec<String>,
}
//
//
impl TaskConf {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// task taskName:
    ///     cycle: 100 ms
    ///     in queue recv-queue:
    ///         max-length: 10000
    ///     fn sqlUpdateMetric:
    ///         table: "TableName"
    ///         sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
    ///         initial: 123.456
    ///         inputs:
    ///             input1:
    ///                 fn functionName:
    ///                     ...
    ///             input2:
    ///                 fn SqlMetric:
    ///                     ...
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> TaskConf {
        let mut vars = vec![];
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("TaskConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, self_name);
        let cycle = conf.get_duration("cycle").ok();
        log::trace!("{}.new | cycle: {:?}", dbg, cycle);
        let (rx, rx_max_length) = conf.get_in_queue().unwrap();
        log::trace!("{}.new | RX: {},\tmax-length: {:?}", dbg, rx, rx_max_length);
        let subscribe = conf.get("subscribe").unwrap_or_else(|| {
            log::warn!("{dbg}.new | 'subscribe' - not found, {self_name} will listen only default Receiver");
            serde_yaml::Value::Null
        });
        let subscribe = ConfSubscribe::new(subscribe);
        log::trace!("{}.new | subscribe: {:#?}", dbg, subscribe);
        let mut node_index = 0;
        let mut nodes = IndexMap::new();
        for key in conf.keys(&["wait-started", "cycle", "subscribe", format!("in queue {}", rx).as_str()]) {
            let node_conf = conf.get(key).unwrap();
            log::trace!("{}.new | nodeConf: {:?}", dbg, node_conf);
            node_index += 1;
            let node_conf = FnConfig::new(&self_name.join(), &self_name, &node_conf, &mut vars);
            nodes.insert(
                format!("{}-{}", node_conf.name(), node_index),
                node_conf,
            );
        }
        TaskConf {
            name: self_name,
            cycle,
            rx,
            rx_max_length,
            subscribe,
            nodes,
            vars,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> TaskConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("TaskConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> TaskConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        TaskConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("TaskConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("TaskConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConf> {
        self.nodes.iter().fold(vec![], |mut points, (_node_name,node_conf)| {
            points.extend(node_conf.points());
            points
        })
    }
}
}
mod task {
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{
    ConfSubscribe, Service, ServiceCycle, Services, SubscriptionCriteria, entity::{Name, Object, Point, PointConf, PointTxId}
}, sync::{Handles, Owner, channel::{self, Receiver, RecvTimeoutError, Sender}}, thread_pool::Scheduler};
use std::{
    collections::HashMap, fmt::Debug, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration
};
use concat_string::concat_string;
use crate::{
    domain::constants::constants::RECV_TIMEOUT, services::task::{task_conf::TaskConf, task_nodes::TaskNodes}, sync::SendWrapper,
};
///
/// Task implements entity, which provides cyclically (by event) executing calculations
///  - executed in the cycle mode (current impl)
///  - executed event mode (future impl..)
///  - has some number of functions / variables / metrics or additional entities
pub struct Task {
    name: Name,
    in_send: HashMap<String, Sender<Point>>,
    rx_recv: Owner<Receiver<Point>>,
    services: Arc<Services>,
    conf: TaskConf,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
//
//
impl Task {
    ///
    /// Creates new instance of [Task]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: TaskConf, services: Arc<Services>, scheduler: Scheduler) -> Task {
        let (send, recv) = channel::unbounded();
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Task {
            name: conf.name.clone(),
            in_send: HashMap::from([("in-send".to_owned(), send)]),
            rx_recv: Owner::new(recv),
            services,
            conf,
            scheduler,
            handles: Handles::new(&dbg),
            exit: Arc::new(ExitNotify::new(&dbg, None, None)),
            dbg,
        }
    }
    ///
    ///
    fn subscriptions_(&self, conf: &ConfSubscribe, services: &Arc<Services>) -> Option<(String, Vec<SubscriptionCriteria>)> {
        if conf.is_empty() {
            None
        } else {
            log::trace!("{}.subscriptions | requesting points...", self.dbg);
            let mut self_points = self.conf.points();
            let mut points = services.points(&self.dbg).then(
                |points| points,
                |err| {
                    log::error!("{}.subscriptions | Requesting Points error: {:?}", self.dbg, err);
                    vec![]
                },
            );
            points.append(&mut self_points);
            log::trace!("{}.subscriptions | rceived points: {:#?}", self.dbg, points.len());
            log::trace!(
                "{}.subscriptions | rceived points: {:#?}",
                self.dbg,
                points.iter().map(|p| concat_string!(p.id.to_string(), " | ", p.type_.to_string(), " | ", p.name)).collect::<Vec<String>>(),
            );
            // log::debug!("{}.subscriptions | conf.subscribe: {:#?}", self.dbg, conf);
            let subscriptions = conf.with(&points);
            if subscriptions.len() > 1 {
                log::error!("{}.subscriptions | Task does not supports multiple subscriptions for now: {:#?}.\n\tTry to use single subscription.", self.dbg, subscriptions);
                None
            } else {
                match subscriptions.clone().into_iter().next() {
                    Some((service_name, Some(points))) => {
                        log::debug!("{}.subscriptions | subscriptions: {:#?}", self.dbg, points.iter().map(|s| format!("{service_name}: {}", s.destination())).collect::<Vec<String>>());
                        Some((service_name, points))
                    }
                    Some((_, None)) => {
                        log::error!("{}.subscriptions | Task subscription configuration error / empty in: {:#?}", self.dbg, subscriptions);
                        None
                    }
                    None => {
                        log::error!("{}.subscriptions | Task subscription configuration error in: {:#?}", self.dbg, subscriptions);
                        None
                    }
                }
            }
        }
    }
    ///
    ///
    fn subscribe_(&self, subscriptions: &Option<(String, Vec<SubscriptionCriteria>)>, services: &Arc<Services>) -> Receiver<Point> {
        match subscriptions {
            Some((service_name, points)) => {
                let (_, rx_recv) = services.subscribe(
                    service_name,
                    &self.name.join(),
                    points,
                );
                rx_recv
            }
            None => self.rx_recv.take().unwrap(),
        }
    }
}
//
//
impl Object for Task {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for Task {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Task")
            .field("id", &self.dbg)
            .finish()
    }
}
impl Service for Task {
    fn get_link(&self, name: &str) -> Sender<Point> {
        match self.in_send.iter().next() {
            Some((_, send)) => send.clone(),
            None => {
                log::error!("{}.get_link | link '{}' - not found", self.dbg, name);
                panic!("{}.get_link | Error", self.dbg);
            }
        }
    }
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        log::trace!("{}.run | Self tx_id: {}", self.dbg, PointTxId::from_str(&self.name.join()));
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let conf_cycle = conf.cycle;
        let services = self.services.clone();
        let task_nodes = {
            let mut task_nodes = TaskNodes::new(&dbg);
            task_nodes.build_nodes(&self_name, &conf, services.clone())
                .map_err(|err| Error::new(&dbg, "run").pass(err))?;
            SendWrapper::wrap(task_nodes)
        };
        let subscriptions = self.subscriptions_(&conf.subscribe, &services);
        let rx_recv = self.subscribe_(&subscriptions, &services);
        let handle = self.scheduler.spawn({
            let dbg = dbg.clone();
            let (cyclic, cycle_interval, recv_timeout) = match conf_cycle {
                Some(interval) => (interval > Duration::ZERO, interval, interval),
                None => (false, Duration::ZERO, RECV_TIMEOUT),
            };
            move || {
            let mut cycle = ServiceCycle::new(&dbg, cycle_interval);
            let task_nodes = task_nodes.extract();
            log::trace!("{dbg}.run | task_nodes: {:#?}", task_nodes);
            'main: while !exit.get() {
                log::trace!("{dbg}.run | Calculation step...");
                if cyclic {
                    cycle.start();
                    match rx_recv.recv_timeout(recv_timeout) {
                        Ok(point) => {
                            // log::debug!("{dbg}.run | point: {:?}", &point);
                            log::debug!("{dbg}.run | Event '{}': {:?}  {:?}  {:?}", point.name(), point.value(), point.status(), point.cot());
                            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                task_nodes.eval(point);
                            }));
                            match result {
                                Ok(_) => log::debug!("{dbg}.run | Calculation step - done ({:?})", cycle.elapsed()),
                                Err(err) => log::error!("{dbg}.run | Calculation step - fails {:?}", err),
                            }
                            cycle.wait();
                        }
                        Err(err) => match err {
                            RecvTimeoutError::Timeout => {},
                            _ => {
                                log::trace!("{dbg}.run | Error receiving from queue: {:?}", err);
                                break 'main;
                            }
                        }
                    };
                } else {
                    match rx_recv.recv_timeout(RECV_TIMEOUT) {
                        Ok(point) => {
                            log::debug!("{dbg}.run | point: {:?}", &point);
                            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                task_nodes.eval(point);
                            }));
                            match result {
                                Ok(_) => log::debug!("{dbg}.run | Calculation step - done ({:?})", cycle.elapsed()),
                                Err(err) => log::error!("{dbg}.run | Calculation step - fails {:?}", err),
                            }
                        }
                        Err(err) => match err {
                            RecvTimeoutError::Timeout => {},
                            _ => {
                                log::error!("{dbg}.run | Error receiving from queue: {:?}", err);
                                break 'main;
                            }
                        }
                    };
                }
            }
            if let Some((service_name, points)) = subscriptions {
                if let Err(err) = services.unsubscribe(&service_name,&self_name.join(), &points) {
                    log::error!("{dbg}.run | Unsubscribe error: {:#?}", err);
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        }});
        match handle {
            Ok(handle) => {
                log::info!("{dbg}.run | Starting - ok");
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn points(&self) -> Vec<PointConf> {
        self.conf.points()
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.exit();
    }
}
}
mod task_nodes {
use std::{cell::{Cell, RefCell}, rc::Rc, sync::Arc};
use indexmap::IndexMap;
use sal_core::error::Error;
use sal_sync::services::{entity::{Name, Point, PointTxId}, Services, task::functions::FnConfKind};
use crate::{
    domain::{FnInOutRef, FnOutRef},
    services::task::{EvalCycle, FnEnableMode, FnEvalOnce, functions::{FnBuilder, FnKind}, task_conf::TaskConf},
};
use super::{task_node_vars::TaskNodeVars, task_eval_node::TaskEvalNode};
///
/// ### TaskNodes - holds the IndexMap<String, TaskNode> in the following structure:
///   ```
///   {
///       inputName1: TaskEvalNode {
///           input: FnInOutRef,
///           outs: [
///               var1
///               var2
///               var...
///               metric1
///               metric2
///               metric...
///           ]
///       },
///       inputName2: TaskEvalNode {
///           ...
///       },
///   }
///   ```
/// **every - Input wildcard**. Подстановка любого сигнала
/// - **Назначение**: `every` работает как wildcard (подстановочный знак) или глобальный триггер. Если узел подписан на `point type every` (например, `point any every` или `point int every`), то любое входящее событие (`Point`), переданное в `Task`, должно вызвать перерасчёт этого узла.
/// - **Типизация**: Дополнительный фильтр по типу (например, `point int every`) означает, что узел среагирует на любое событие, только если его значение имеет тип `int`. `point any every` реагирует вообще на всё.
/// - **Ограничение**: Все вычисления, зависящие от `every`, должны сводиться к одному корню (или одному выходному узлу).
#[derive(Debug)]
pub struct TaskNodes {
    dbg: String,
    nodes: IndexMap<String, Rc<RefCell<TaskEvalNode>>>,
    vars: IndexMap<String, FnOutRef>,
    new_node_vars: Option<TaskNodeVars>,
    /// Текущий номер вычислительного цикла, инкремнтируется с каждым входом в `self.eval`
    cycle: EvalCycle,
    /// Enable Strategy: Cold Standby / Warm Standby (TODO: read from config)
    enable_mode: FnEnableMode,
}
//
//
impl TaskNodes {
    ///
    /// Creates new empty instance
    pub fn new(parent: impl Into<String>) ->Self {
        Self {
            dbg: format!("{}/TaskNodes", parent.into()),
            nodes: IndexMap::new(),
            vars: IndexMap::new(),
            new_node_vars: None,
            cycle: Rc::new(Cell::new(0)),
            enable_mode: FnEnableMode::Cold,
        }
    }
    ///
    /// Returns Enable Strategy: Cold Standby / Warm Standby
    pub fn enable_mode(&self) -> FnEnableMode {
        self.enable_mode
    }
    ///
    /// ### Shared calculation cycle
    ///
    /// Возвращает ссылку на текущий номер вычислительного цикла
    pub fn cycle(&self) -> EvalCycle {
        self.cycle.clone()
    }
    ///
    /// Returns all configured inputs
    pub fn get_inputs(&self) -> Vec<String> {
        self.nodes.keys().map(|k| k.to_string()).collect()
    }
    ///
    /// Returns input by it's name
    pub fn get_eval_node(&self, name: &str) -> Option<Rc<RefCell<TaskEvalNode>>> {
        self.nodes.get(name).map(|node| node.clone())
    }
    pub fn get_var(&self, name: &str) -> Option<&FnOutRef> {
        log::trace!("{}.getVar | trying to find variable {:?} in {:?}", self.dbg, &name, self.vars);
        self.vars.get(name)
    }
    pub fn add_input(&mut self, name: impl Into<String>, input: FnInOutRef) -> Result<FnOutRef, Error> {
        let name = name.into();
        match self.new_node_vars {
            Some(_) => {
                match self.nodes.get_mut(&name) {
                    Some(node) => {
                        log::trace!("{}.add_input | input {:?}:{} - adding to the existing node if has different 'options hash'", self.dbg, name, input.borrow().hash());
                        Ok(node.borrow_mut().add_input(input))
                    }
                    None => {
                        log::trace!("{}.add_input | adding input {:?}:{}", self.dbg, name, input.borrow().hash());
                        log::trace!("{}.add_input | adding input {:?}:{}: {:?}", self.dbg, name, input.borrow().hash(), input);
                        self.nodes.insert(
                            name.clone(),
                            Rc::new(RefCell::new(TaskEvalNode::new(&self.dbg, name, vec![input.clone()]))),
                        );
                        Ok(input)
                    }
                }
            }
            None => Err(Error::new(&self.dbg, "add_input").err("Call begin_new_node first, then you can add inputs"))
        }
    }
    pub fn add_var(&mut self, name: impl Into<String>, var: FnOutRef) -> Result<(), Error> {
        let name = name.into();
        assert!(!name.is_empty(), "Variable name can't be emty");
        match self.new_node_vars.as_mut() {
            Some(new_node_vars) => {
                if self.vars.contains_key(&name) {
                    return Err(Error::new(&self.dbg, "add_var").err(format!("Dublicated variable name: {:?}", name)));
                } else {
                    log::trace!("{}.add_var | adding variable {:?}", self.dbg, &name);
                    log::trace!("{}.add_var | adding variable {:?}: {:?}", &name, self.dbg, &var);
                    self.vars.insert(
                        name.clone(),
                        var,
                    );
                }
                new_node_vars.add_var(name)
            }
            None => Err(Error::new(&self.dbg, "add_var").err(format!("Error: call beginNewNode first, then you can add inputs"))),
        }
    }
    pub fn add_var_out(&mut self, name: impl Into<String>) -> Result<(), Error> {
        let name = name.into();
        assert!(!name.is_empty(), "Variable name can't be emty");
        match self.new_node_vars.as_mut() {
            Some(new_node_vars) => {
                new_node_vars.add_var(name).map_err(|err| Error::new(&self.dbg, "add_var_out").pass(err))
            }
            None => Err(Error::new(&self.dbg, "add_var_out").err("Call beginNewNode first, then you can add inputs")),
        }
    }
    fn finish_new_node(&mut self, out: FnOutRef) -> Result<(), Error> {
        match self.new_node_vars.as_mut() {
            Some(new_node_vars) => {
                let mut vars: Vec<FnOutRef> = vec![];
                for var_name in new_node_vars.get_vars() {
                    match self.vars.get(&var_name) {
                        Some(var) => {
                            vars.push(
                                var.clone()
                            );
                        }
                        None => {
                            return Err(Error::new(&self.dbg, "finish_new_node").err(&format!("{}.finish_new_node | Variable {:?} - not found", self.dbg, var_name)))
                        }
                    };
                };
                let inputs = out.borrow().inputs();
                log::trace!("{}.finish_new_node | out {:#?} \n\tdipending on inputs:: {:#?}\n", self.dbg, &out, inputs);
                for input_name in inputs {
                    match self.nodes.get(&input_name) {
                        Some(eval_node) => {
                            log::trace!("{}.finish_new_node | updating input: {:?}", self.dbg, input_name);
                            let len = vars.len();
                            eval_node.borrow_mut().add_vars(&vars.clone());
                            if out.borrow().kind() != FnKind::Var {
                                eval_node.borrow_mut().add_out(out.clone());
                            }
                            log::trace!("{}.finish_new_node | evalNode '{}' appended: {:?}", self.dbg, eval_node.borrow().name(), len);
                        }
                        None => {
                            return Err(Error::new(&self.dbg, "finish_new_node").err(&format!("{}.finish_new_node | Input {:?} - not found", self.dbg, input_name)))
                        }
                    };
                };
                self.new_node_vars = None;
                log::trace!("\n{}.finish_new_node | self.inputs: {:?}\n", self.dbg, self.nodes);
            }
            None => {
                return Err(Error::new(&self.dbg, "finish_new_node").err(&format!("{}.finish_new_node | Call beginNewNode first, then you can add inputs & vars, then finish node", self.dbg)))
            }
        }
        Ok(())
    }
    pub fn build_nodes(&mut self, parent: &Name, conf: &TaskConf, services: Arc<Services>) -> Result<(), Error>{
        let error = Error::new(&self.dbg, "build_nodes");
        let tx_id = PointTxId::from_str(&parent.join());
        let conf_nodes = conf.nodes.clone();
        for (idx, (_node_name, mut node_conf)) in conf_nodes.into_iter().enumerate() {
            let node_name = node_conf.name();
            log::trace!("{}.build_nodes | node[{}]: {:?}", self.dbg, idx, node_name);
            self.new_node_vars = Some(TaskNodeVars::new());
            let out = match node_conf {
                FnConfKind::Fn(_) => {
                    Rc::new(RefCell::new(FnEvalOnce::new(parent, self.cycle.clone(),
                        FnBuilder::new(parent, tx_id, &mut node_conf, self, services.clone())
                            .map_err(|err| error.pass_with(format!("Can't build eval node '{node_name}': {:?}", conf), err))?,
                    )))
                }
                FnConfKind::Var(_) => {
                    Rc::new(RefCell::new(FnEvalOnce::new(parent, self.cycle.clone(),
                    FnBuilder::new(parent, tx_id, &mut node_conf, self, services.clone())
                            .map_err(|err| error.pass_with(format!("Can't build eval node '{node_name}': {:?}", conf), err))?,
                    )))
                }
                FnConfKind::Const(conf) => {
                    return Err(error.err(format!("Const is not supported in the root of the Task, config: {:?}: {:?}", node_name, conf)));
                }
                FnConfKind::Point(conf) => {
                    return Err(error.err(format!("Point is not supported in the root of the Task, config: {:?}: {:?}", node_name, conf)));
                }
                FnConfKind::PointConf(conf) => {
                    return Err(error.err(format!("PointConf is not supported in the root of the Task, config: {:?}: {:?}", node_name, conf)));
                }
                FnConfKind::Param(conf) => {
                    return Err(error.err(format!("Param (custom parameter) is not supported in the root of the Task, config: {:?}: {:?} - ", node_name, conf)));
                }
            };
            self.finish_new_node(out)
                .map_err(|err| error.pass_with(format!("Can't finish node {node_name}"), err))?;
        }
        Ok(())
    }
    pub fn eval(&self, point: Point) {
        let dbg = self.dbg.clone();
        self.cycle.update(|c| c + 1);
        let point_name = point.name();
        let node_every = self.get_eval_node("every").map(|eval_node_every| {
            log::trace!("{dbg}.eval | evalNode '{}' - adding point...", &eval_node_every.borrow().name());
            eval_node_every.borrow().add(&point);
            eval_node_every
        });
        let node_spec = self.get_eval_node(&point_name).map(|eval_node| {
            log::trace!("{dbg}.eval | evalNode '{}' - adding point...", eval_node.borrow().name());
            eval_node.borrow().add(&point);
            eval_node
        });
        if let Some(node) = node_every {
            log::trace!("{dbg}.eval | evalNode '{}' - evaluating...", node.borrow().name());
            node.borrow_mut().eval();
        }
        if let Some(node) = node_spec {
            log::trace!("{dbg}.eval | evalNode '{}' - evaluating...", node.borrow().name());
            node.borrow_mut().eval();
        }
    }
}}
mod task_node_vars {
use sal_core::error::Error;
#[derive(Debug)]
pub struct TaskNodeVars {
    vars: Vec<String>,
}
impl TaskNodeVars {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
        }
    }
    pub fn add_var(&mut self, name: impl Into<String> + Clone) -> Result<(), Error> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::new("TaskNodeVars", "add_var").err("Variable name can't be emty"));
        }
        log::trace!("TaskNodeStuff.addVar | adding variable {:?}", name);
        self.vars.push(name);
        Ok(())
    }
    pub fn get_vars(&self) -> Vec<String> {
        self.vars.clone()
    }
}
}
mod task_eval_node {
use sal_core::dbg::Dbg;
use sal_sync::services::entity::Point;
use crate::{domain::{FnInOutRef, FnOutRef}, services::task::FnResult};
#[derive(Debug)]
pub struct TaskEvalNode {
    name: String,
    input: Vec<FnInOutRef>,
    vars: Vec<FnOutRef>,
    outs: Vec<FnOutRef>,
    dbg: Dbg,
}
impl TaskEvalNode {
    pub fn new(parent: impl Into<String>, name: impl Into<String>, input: Vec<FnInOutRef>) -> Self {
        let name = name.into();
        let dbg = Dbg::new(parent, &name);
        TaskEvalNode {
            name,
            input,
            vars:  vec![],
            outs: vec![],
            dbg,
        }
    }
    pub fn add_input(&mut self, input: FnInOutRef) -> FnInOutRef {
        let input_hash = input.borrow().hash();
        for input in &self.input {
            let hash = input.borrow().hash();
            if input_hash == hash {
                return input.clone();
            }
        }
        self.input.push(input.clone());
        log::trace!("TaskEvalNode.add_input | eval_node '{}' - input '{}' added", self.dbg, input.borrow().hash());
        input
    }
    fn contains_var(&self, var: &FnOutRef) -> bool {
        let var_id = var.borrow().id();
        for self_var in &self.vars {
            if self_var.borrow().id() == var_id {
                return true;
            }
        }
        false
    }
    fn contains_out(&self, out: &FnOutRef) -> bool {
        let out_id = out.borrow().id();
        for self_out in &self.outs {
            if self_out.borrow().id() == out_id {
                return true;
            }
        }
        false
    }
    pub fn add_vars(&mut self, vars: &Vec<FnOutRef>) {
        for var in vars {
            if !self.contains_var(var) {
                self.vars.push(var.clone());
            }
        }
    }
    pub fn add_out(&mut self, out: FnOutRef) {
        if !self.contains_out(&out) {
            self.outs.push(out);
        }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    #[allow(unused)]
    pub fn get_vars(&self) -> &Vec<FnOutRef> {
        &self.vars
    }
    pub fn get_outs(&self) -> &Vec<FnOutRef> {
        &self.outs
    }
    pub fn add(&self, point: &Point) {
        for input in &self.input {
            input.borrow_mut().add(point);
        }
    }
    pub fn eval(&mut self) {
        for eval_node_out in &self.outs {
            log::trace!("TaskEvalNode.eval | node '{}' out...", self.dbg);
            match eval_node_out.borrow_mut().out() {
                Ok(Some(_)) => {
                }
                Ok(None) => {
                    log::warn!("TaskEvalNode.eval | node '{}' out: 'None'", self.dbg);
                }
                Err(err) => {
                    log::warn!("TaskEvalNode.eval | node '{}' out: {}", self.dbg, err);
                }
            }
        };
    }
}
}
use std::{cell::Cell, rc::Rc};
pub(super) use fn_eval_once::*;
pub use functions::*;
pub use task_conf::*;
pub use task::*;
pub use task_nodes::*;
pub use task_node_vars::*;
pub use task_eval_node::*;
pub use task_test_receiver::*;
pub use task_test_producer::*;
pub(crate) type EvalCycle = Rc<Cell<usize>>;
