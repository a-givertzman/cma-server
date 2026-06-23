mod eval_cycle {
use std::{cell::Cell, rc::Rc};
pub(crate) type EvalCycleRef = Rc<EvalCycle>;
#[derive(Debug)]
pub(crate) struct EvalCycle {
    val: Cell<CycleIndex>,
}
impl EvalCycle {
    pub fn new() -> Self {
        Self {
            val: Cell::new(
                CycleIndex::restart()
            )
        }
    }
    pub fn increment(&self) {
        self.val.update(|mut v| {
            if v.0 >= u64::MAX { return CycleIndex::restart() }
            v.0 = v.0 + 1;
            v
        })
    }
    pub fn get(&self) -> CycleIndex {
        self.val.get()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CycleIndex(u64);
impl CycleIndex {
    pub(crate) fn new() -> Self {
        Self(0)
    }
    pub(crate) fn update(&mut self, cycle: &Self) -> bool {
        if self.0 != cycle.0 {
            self.0 = cycle.0;
            return true;
        }
        false
    }
    fn restart() -> Self {
        Self(1)
    }
}
}
mod fn_eval_once {
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{CycleIndex, EvalCycleRef, FnFlow, FnKind, FnOut, FnResult},
};
#[derive(Debug)]
pub struct FnEvalOnce {
    id: String,
    cycle: CycleIndex,
    eval_cycle: EvalCycleRef,
    input: FnOutRef,
    state: FnResult<FnFlow, String>,
}
impl FnEvalOnce {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, eval_cycle: EvalCycleRef, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnEvalOnce{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            cycle: CycleIndex::new(),
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
        if !self.cycle.update(&self.eval_cycle.get()) {
            return self.state.clone();
        }
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
mod application {
mod cma_recorder {
mod fn_rec_op_cycle_metric {
use indexmap::IndexMap;
use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::{Point, PointType}, sync::channel::Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::{Edge, EdgeDetector, FnOutRef}, services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult}};
#[derive(Debug)]
pub struct FnRecOpCycleMetric {
    id: String,
    kind: FnKind,
    send_to: Option<Sender<Point>>,
    reset: Option<FnChange>,
    op_cycle: FnChange,
    inputs: FxIndexMap<String, FnChange>,
    values: FxIndexMap<String, Point>,
    state: State,
    reset_edge: EdgeDetector,
}
impl FnRecOpCycleMetric {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, send_to: Option<Sender<Point>>, reset: Option<FnOutRef>, op_cycle: FnOutRef, inputs: impl IntoIterator<Item = (String, FnOutRef)>) -> Self {
        let id = format!("{}/FnRecOpCycleMetric{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            kind: FnKind::Fn,
            send_to,
            reset: reset.map(FnChange::new),
            op_cycle: FnChange::new(op_cycle),
            inputs: inputs.into_iter().map(|(k, inp)| (k, FnChange::new(inp))).collect(),
            values: FxIndexMap::default(),
            state: State::new(&id),
            reset_edge: EdgeDetector::new(),
            id,
        }
    }
}
impl FnOut for FnRecOpCycleMetric {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        let mut inputs = self.op_cycle.inputs();
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.inputs());
        }
        for (_, input) in &self.inputs {
            inputs.append(&mut input.inputs());
        }
        inputs
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let reset = self.reset.as_mut().map(|f| f.out());
        let op_cycle = self.op_cycle.out();
        let inputs: IndexMap<&String, FnResult<FnFlow, String>> = self.inputs.iter_mut()
            .map(|(key, input)| (key, input.out())).collect();
        if let Some(reset) = reset {
            if let Some(reset) = reset? {
                if let Some(Edge::Rising) = self.reset_edge.add(reset.into_value().to_bool().as_bool().value.0) {
                    self.state.reset();
                }
            }
        }
        let Some(op_cycle_point) = flow.map(op_cycle)? else { return Ok(None) };
        let op_cycle = match op_cycle_point.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => op_cycle_point.to_bool().as_bool().value.0,
            _ => return Err(format!("{}.out | Invalid op_cycle type '{:?}', expected bool or number", self.id, op_cycle_point.typ())),
        };
        match self.state.add(op_cycle) {
            Cycle::None => {}
            Cycle::Started => {
                log::trace!("{}.out | Operating Cycle - Active", self.id);
                for (input_name, input) in inputs {
                    if let Some(val_flow) = input? {
                        let value = val_flow.into_value();
                        if value.typ() == PointType::String {
                            self.values.insert(input_name.to_owned(), value);
                        } else {
                            log::warn!("{}.out | Input '{}': unexpected type {:?}, string sql requared", self.id, input_name, value.typ());
                        }
                    }
                }
            }
            Cycle::Finished => {
                log::debug!("{}.out | Operating Cycle - SENDING {} values...", self.id, self.values.len());
                let log_values: Vec<String> = self.values.iter().map(|(key, point)| {
                    format!("'{}': '{}'", key, point.value().to_string())
                }).collect();
                log::debug!("{}.out | Operating Cycle - values ({}): {:#?}", self.id, self.values.len(), log_values);
                if let Some(tx) = &self.send_to {
                    for (_, value) in self.values.drain(..) {
                        if let Err(err) = tx.send(value) {
                            log::error!("{}.out | Send error: {:#?}", self.id, err);
                        }
                    }
                } else {
                    log::warn!("{}.out | Point can't be sent - 'send-to' is not specified", self.id);
                    self.values.clear();
                }
            }
        }
        flow.wrap(op_cycle_point)
    }
    fn reset(&mut self) {
        self.state.reset();
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.reset_edge.reset();
        self.op_cycle.reset();
        for (_, input) in &mut self.inputs {
            input.reset();
        }
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
#[derive(Debug, Clone, Copy, PartialEq)]
enum Cycle {
    None,
    Started,
    Finished,
}
#[derive(Debug)]
struct State {
    state: Cycle,
    dbg: Dbg,
}
impl State {
    pub fn new(parent: impl Into<String>) -> Self {
        Self {
            state: Cycle::None,
            dbg: Dbg::new(parent, "State")
        }
    }
    pub fn add(&mut self, op_cycle: bool) -> Cycle {
        match self.state {
            Cycle::None => match op_cycle {
                true => {
                    log::debug!("{}.add | Operating Cycle - STARTED", self.dbg);
                    self.state = Cycle::Started;
                    self.state
                }
                false => Cycle::None,
            },
            Cycle::Started => match op_cycle {
                true => Cycle::Started,
                false => {
                    log::debug!("{}.add | Operating Cycle - FINISHED", self.dbg);
                    self.state = Cycle::None;
                    Cycle::Finished
                }
            },
            Cycle::Finished => unreachable!(),
        }
    }
    #[allow(unused)]
    pub fn reset(&mut self) {
        self.state = Cycle::None;
    }
}
}
pub use fn_rec_op_cycle_metric::*;
}
mod va {
mod fft_buff {
use std::f64::consts::PI;
use rustfft::{num_complex::Complex, num_traits::Zero};
pub struct FftBuf {
    size: usize,
    amp_factor: f64,
    time_i: usize,
    unit_complex: Vec<Complex<f64>>,
    index: usize,
    index_last: usize,
    complex: Vec<Complex<f64>>,
}
impl FftBuf {
    pub fn new(size: usize) -> Self {
        let unit_complex: Vec<Complex<f64>> = (0..size).into_iter().map(|i| {
            let angle = PI * 2.0 * (i as f64) / (size as f64);
            Complex {
                re: angle.cos(),
                im: angle.sin()
            }
        }).collect();
        log::trace!("FftBuf.new | unit_complex: {:?}", unit_complex);
        Self {
            size,
            amp_factor: 2.0 / (size as f64),
            time_i: 0,
            unit_complex,
            index: 0,
            index_last: size - 1,
            complex: vec![Complex::zero(); size],
        }
    }
    pub fn amp_factor(&self) -> f64 {
        self.amp_factor
    }
    pub fn add(&mut self, value: f64) -> Option<&mut [Complex<f64>]> {
        if self.index == 0 {
            self.complex = vec![Complex::zero(); self.size];
        }
        self.complex[self.index].re = value * self.unit_complex[self.index].re;
        self.complex[self.index].im = value * self.unit_complex[self.index].im;
        log::trace!("FftBuf.add | index: {}", self.index);
        if self.index < self.index_last {
            self.index = (self.index  + 1) % self.size;
            self.time_i += 1;
            None
        } else {
            self.index = (self.index  + 1) % self.size;
            self.time_i += 1;
            Some(&mut self.complex)
        }
    }
    pub fn reset(&mut self) {
        self.index = 0;
        self.time_i = 0;
    }
    pub fn freq_of(&self, sampl_freq: usize, index: usize) -> f64 {
        let freq_factor = (sampl_freq as f64) / (self.size as f64);
        (index as f64) * freq_factor
    }
    #[allow(unused)]
    pub fn time(sampl_freq: usize, index: usize) -> f64 {
        let sampling_period = 1.0 / (sampl_freq as f64);
        (index as f64) * sampling_period
    }
}
}
mod fn_va_fft {
use chrono::Utc;
use derivative::Derivative;
use indexmap::IndexMap;
use rustfft::{num_complex::ComplexFloat, Fft, FftPlanner};
use sal_core::error::Error;
use sal_sync::{collections::FxHashMap, services::{
    entity::{
        Cot, Name,
        Point, PointConf, PointConfFilter, PointType, PointHlr, PointTxId,
        Status, ToPoint,
    }, task::functions::{FnConfKind, FnConfOptions, FnConfPointType, FnConfig}, types::Bool, LinkName, Services
}, sync::channel::Sender};
use std::{cell::RefCell, rc::Rc, str::FromStr, sync::{atomic::{AtomicUsize, Ordering}, Arc}};
use crate::{
    domain::{FnInOutRef, FnOutRef, filter::{filter::{Filter, FilterEmpty}, filter_threshold::FilterThreshold}, format::FormatPoint},
    services::task::{
        FlowContext, FnConst, FnFlow, FnInput, FnKind, FnOut, FnResult, FnRetain
    }
};
use super::fft_buff::FftBuf;
#[derive(Derivative)]
#[derivative(Debug)]
pub struct FnVaFft {
    txid: usize,
    id: String,
    kind: FnKind,
    point_conf: PointConf,
    fft_size: usize,
    input: FnOutRef,
    #[derivative(Debug="ignore")]
    fft: Arc<dyn Fft<f64>>,
    fft_freqs: Vec<String>,
    amp_factor: f64,
    #[derivative(Debug="ignore")]
    fft_buf: FftBuf,
    sampl_freq: Option<usize>,
    retain: FxHashMap<String, (FnInOutRef, FnRetain)>,
    filters: Vec<(String, Box<dyn Filter<Item = f64>>)>,
    tx_send: Option<Sender<Point>>,
    format: Option<FormatPoint>,
    format_key: String,
}
impl FnVaFft {
    #[allow(unused)]
    pub fn new(parent: impl Into<String>, input: FnOutRef, conf: FnConfig, services: Arc<Services>) -> Result<Self, Error> {
        let parent = parent.into();
        let name = Name::new(&parent, format!("FnVaFft-{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let dbg = name.join();
        return Err(Error::new(&dbg, "new").err("Isn't implemented yet"));
    }
    fn retain_input(parent: impl Into<String>, txid: usize, freq_name: &str) -> FnInOutRef {
        Rc::new(RefCell::new(
            FnInput::new(
                parent,
                txid,
                &mut FnConfig {
                    name: freq_name.to_string(),
                    inputs: IndexMap::new(),
                    type_: FnConfPointType::Double,
                    options: FnConfOptions::default(),
                },
                todo!("Pass an calculation cycle if realy required !"),
            )
        ))
    }
    fn build_filter(conf: Option<PointConfFilter>, initial: Option<f64>) -> Box<dyn Filter<Item = f64>> {
        match conf {
            Some(conf) => {
                Box::new(
                    FilterThreshold::<f64>::new(initial, conf.threshold, conf.factor.unwrap_or(0.0))
                )
            }
            None => Box::new(FilterEmpty::<f64>::new(None)),
        }
    }
    fn parse_point_conf(parent: impl Into<String>, self_id: &str, conf: &FnConfig) -> PointConf {
        match conf.clone().input_conf("conf") {
            Ok(conf) => match conf {
                FnConfKind::PointConf(conf) => match conf.conf.type_ {
                    PointType::Int | PointType::Real | PointType::Double => conf.conf.clone(),
                    _ => panic!("{}.new | Invalid Point type: '{:?}' in {:#?}", self_id, conf.conf.type_, conf.conf),
                }
                _ => panic!("{}.new | Invalid Point config in: {:?}", self_id, conf.name()),
            }
            Err(_) => PointConf::from_yaml(&Name::new(parent, ""), &serde_yaml::from_str(r#"
                conf point FFT:
                    type: 'Real'
            "#).unwrap()),
        }
    }
    fn parse_threshold_conf(self_id: &str, conf: &FnConfig) -> Option<PointConfFilter> {
        match conf.param("filter") {
            Some(threshold) => match threshold {
                FnConfKind::Param(threshold) => match serde_yaml::from_value(threshold.conf.clone()) {
                    Ok(threshold) => {
                        let threshold: PointConfFilter = threshold;
                        Some(threshold)
                    }
                    Err(err) => {
                        log::warn!("{}.new | Invalid Threshold filter config in: {:?}, \n\t error: {:#?}", self_id, conf, err);
                        None
                    }
                }
                _ => {
                    log::warn!("{}.new | Invalid Threshold filter config in: {:?}", self_id, conf);
                    None
                }
            }
            None => {
                log::warn!("{}.new | Threshold filter config missed in: {:?}", self_id, conf);
                None
            },
        }
    }
    fn parse_send_to(self_id: &str, conf: &FnConfig, services: &Arc<Services>) -> Option<Sender<Point>> {
        match conf.param("send-to") {
            Some(send_to) => {
                match send_to {
                    FnConfKind::Param(send_to) => {
                        let send_to = LinkName::from_str(send_to.conf.as_str().unwrap()).unwrap();
                        log::debug!("{}.new | send-to: {:?}", self_id, send_to.name());
                        services.get_link(&send_to).map_or(None, |send| Some(send))
                    }
                    _ => {
                        log::warn!("{}.new | Parameter 'send-to' - invalid type (string expected): {:#?}", self_id, send_to);
                        None
                    }
                }
            }
            None => {
                log::warn!("{}.new | Parameter 'send-to' - missed in {:#?}", self_id, conf);
                None
            },
        }
    }
    fn parse_format(dbg: &str, conf: &FnConfig) -> (Option<FormatPoint>, String) {
        match conf.param("format") {
            Some(conf) => {
                let conf = conf.as_param().conf.as_str().unwrap().to_owned();
                log::debug!("{dbg}.new | format: {conf}");
                let Ok(format) = FormatPoint::new(&conf) else {
                    log::warn!("{dbg}.new | Wrong format config: '{:?}'", conf);
                    return (None, String::new());
                };
                let format_key = format.markers().into_iter().enumerate().fold(String::new(), |prev, (i, (_, (name, _)))| {
                    if (i > 0) & (prev != name) {
                        panic!("{dbg}.new | format '{conf}' has diferent inputs: '{prev}' and '{name}', but must have single");
                    }
                    name
                });
                (Some(format), format_key)
            }
            None => (None, String::new())
        }
    }
    fn send(self_id: &str, tx_send: &Option<Sender<Point>>, point: Point) {
        if let Some(tx_send) = tx_send {
            match tx_send.send(point) {
                Ok(_) => {
                }
                Err(err) => {
                    log::error!("{}.out | Send error: {:#?}", self_id, err);
                }
            };
        }
    }
    fn fft_process(&mut self, input: &Point) {
        let value = match input {
            Point::Int(point) => point.to_double().value,
            Point::Real(point) => point.to_double().value,
            Point::Double(point) => point.value,
            _ => {
                log::error!("{}.out | Invalid input type '{:?}' Point: {}", self.id, input.typ(), input.name());
                0.0
            }
        };
        match self.fft_buf.add(value) {
            Some(buf) => {
                log::debug!("{}.out | fft.process buf {:?}...", self.id, buf.len());
                self.fft.process(buf);
                for (index, amplitude) in buf.iter().take(self.fft_size / 2).skip(1).enumerate() {
                    match self.filters.get_mut(index) {
                        Some((freq_name, filter)) => {
                            if let Some(value) = filter.add(amplitude.abs() * self.amp_factor) {
                                let point = Point::Double(PointHlr::new(
                                    self.txid,
                                    freq_name.as_str(),
                                    value,
                                    input.status(),
                                    input.cot(),
                                    input.ts(),
                                ));
                                if let Some((retain_input, retain)) = self.retain.get_mut(freq_name) {
                                    retain_input.borrow_mut().add(&point);
                                }
                                let point = match &mut self.format {
                                    Some(format) => {
                                        for (key, _) in format.markers() {
                                            format.insert(&key, point.clone());
                                        }
                                        Point::String(PointHlr::new(
                                            self.txid,
                                            freq_name.as_str(),
                                            format.out(),
                                            input.status(),
                                            input.cot(),
                                            input.ts(),
                                        ))
                                    }
                                    None => {
                                        Point::Double(PointHlr::new(
                                            self.txid,
                                            freq_name.as_str(),
                                            value,
                                            input.status(),
                                            input.cot(),
                                            input.ts(),
                                        ))
                                    },
                                };
                                log::trace!("{}.out | point: {:#?}", self.id, point);
                                Self::send(&self.id, &self.tx_send, point);
                            }
                        }
                        None => log::error!("{}.out | Fft filter index {} out of size {}", self.id, index, self.filters.len()),
                    }
                }
            }
            None => {},
        };
    }
}
impl FnOut for FnVaFft {
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
        let mut flow = FlowContext::new();
        let input = flow.map(self.input.borrow_mut().out());
        log::trace!("{}.out | input: {:#?}", self.id, input);
        match &input {
            Ok(Some(input)) => self.fft_process(input),
            Ok(None) => {},
            Err(err) => log::trace!("{}.out | Input error: {:#?}", self.id, err),
        }
        Ok(None)
    }
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
        self.fft_buf.reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
use rustfft::num_complex::Complex;
#[derive(Debug)]
pub struct UnitCircle {
    pub freq: usize,
    pi2f: f64,
}
impl UnitCircle {
    pub const PI2: f64 = std::f64::consts::PI * 2.0;
    pub fn new(freq: usize) -> Self {
        Self {
            freq,
            pi2f: Self::PI2 * freq as f64,
        }
    }
    pub fn angle(&self, t: f64) -> f64 {
        self.pi2f * t
    }
    pub fn complex(&self, t: f64) -> Complex<f64> {
        let angle = self.angle(t);
        Complex::new(angle.cos(), angle.sin())
    }
    pub fn at(&self, t: f64) -> (f64, Complex<f64>) {
        let angle = self.angle(t);
        (angle, Complex::new(angle.cos(), angle.sin()))
    }
    pub fn at_with(&self, t: f64, amp: f64) -> (f64, Complex<f64>) {
        let angle = self.angle(t);
        self.at_angle_with(angle, amp)
    }
    pub fn at_angle(&self, angle: f64) -> (f64, Complex<f64>) {
        (angle, Complex::new(angle.cos(), angle.sin()))
    }
    pub fn at_angle_with(&self, angle: f64, amp: f64) -> (f64, Complex<f64>) {
        (angle, Complex::new(amp * angle.cos(), amp * angle.sin()))
    }
}
}
pub use fft_buff::*;
pub use fn_va_fft::*;
}
pub use cma_recorder::*;
pub use va::*;}
mod common {
mod fn_acc {
use sal_sync::services::entity::{Point, PointHlr, PointType};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::domain::FnOutRef;
use crate::services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult};
#[derive(Debug)]
pub struct FnAcc {
    id: String,
    kind: FnKind,
    initial: Option<FnChange>,
    input: FnChange,
    acc: Option<Point>,
}
impl FnAcc {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, initial: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnAcc{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            initial: initial.map(|f| FnChange::new(f)),
            input: FnChange::new(input),
            acc: None,
        }
    }
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
}
impl FnOut for FnAcc {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.inputs());
        }
        inputs.append(&mut self.input.inputs());
        inputs
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = self.input.out();
        let initial = self.initial.as_mut().map(|f| f.out());
        let Some(input) = flow.map(input)? else { return Ok(None) };
        let acc = match self.acc.as_ref() {
            Some(acc) => acc.clone(),
            None => {
                let acc = if let Some(initial) = initial {
                    let Some(initial) = initial? else { return Ok(None) };
                    initial.into_value()
                } else {
                    match input.typ() {
                        PointType::Bool | PointType::Int => Point::Int(Self::point_with(&input, input.name(), 0)),
                        PointType::Real => Point::Real(Self::point_with(&input, input.name(), 0.0)),
                        PointType::Double => Point::Double(Self::point_with(&input, input.name(), 0.0)),
                        _ => return Err(format!("{}.out | Invalid input type '{:?}', expected number", self.id, input.typ())),
                    }
                };
                self.acc = Some(acc.clone());
                acc
            }
        };
        if !flow.is_new() {
            return flow.wrap(acc);
        };
        let acc = match &input {
            Point::Bool(_) => acc + input.to_int(),
            _ => acc + input,
        };
        log::trace!("{}.out | out: {:?}", self.id, acc);
        self.acc = Some(acc.clone());
        flow.wrap(acc)
    }
    fn reset(&mut self) {
        if let Some(initial) = &mut self.initial {
            initial.reset();
        }
        self.acc = None;
        self.input.reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_average {
use sal_sync::services::entity::{Point, PointHlr, PointType};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::domain::{Edge, EdgeDetector, FnOutRef};
use crate::services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult};
#[derive(Debug)]
pub struct FnAverage {
    id: String,
    kind: FnKind,
    reset: Option<FnChange>,
    input: FnChange,
    count: i64,
    sum: f64,
    average: Option<Point>,
    reset_edge: EdgeDetector,
}
impl FnAverage {
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnAverage{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            reset: reset.map(FnChange::new),
            input: FnChange::new(input),
            count: 0,
            sum: 0.0,
            average: None,
            reset_edge: EdgeDetector::new(),
        }
    }
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
}
impl FnOut for FnAverage {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        let mut inputs = self.input.inputs();
        if let Some(reset) = self.reset.as_ref() {
            inputs.append(&mut reset.inputs());
        }
        inputs
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let mut force_recalc = false;
        let input = self.input.out();
        let reset = self.reset.as_mut().map(|f| f.out());
        if let Some(reset) = reset {
            if let Some(reset) = reset? {
                if let Some(Edge::Rising) = self.reset_edge.add(reset.into_value().to_bool().as_bool().value.0) {
                    self.count = 0;
                    self.sum = 0.0;
                    self.average = None;
                    force_recalc = true;
                }
            }
        }
        let Some(input) = flow.map(input)? else { return Ok(None) };
        if !flow.is_new() && !force_recalc {
            let Some(average) = self.average.as_ref() else { return Ok(None) };
            return flow.wrap_old(average.clone());
        }
        let value = match input.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        self.sum += value;
        self.count += 1;
        let average = if self.count != 0 {
            self.sum / (self.count as f64)
        } else {
            0.0
        };
        log::trace!("{}.out | sum: {:?}", self.id, self.sum);
        log::trace!("{}.out | count: {:?}", self.id, self.count);
        log::trace!("{}.out | average: {:?}", self.id, average);
        let average = match input.typ() {
            PointType::Int => Point::Int(Self::point_with(&input, &self.id, average.round() as i64)),
            PointType::Real => Point::Real(Self::point_with(&input, &self.id, average as f32)),
            PointType::Double => Point::Double(Self::point_with(&input, &self.id, average)),
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        self.average = Some(average.clone());
        flow.wrap_new(average)
    }
    fn reset(&mut self) {
        self.count = 0;
        self.sum = 0.0;
        self.average = None;
        self.reset_edge.reset();
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.input.reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_count {
use std::sync::atomic::{AtomicUsize, Ordering};
use sal_sync::services::entity::{Point, PointHlr};
use crate::domain::{EdgeDetector, FnOutRef};
use crate::services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult};
#[derive(Debug)]
pub struct FnCount {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    edge: EdgeDetector,
    count: Option<i64>,
    initial: Option<FnOutRef>,
}
impl FnCount {
    pub fn new(parent: impl Into<String>, initial: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnCount{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
            edge: EdgeDetector::new(),
            count: None,
            initial,
        }
    }
    fn point_with(p: &Point, name: impl Into<String>, value: i64) -> Point {
        Point::Int(PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.timestamp()))
    }
}
impl FnOut for FnCount {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        inputs.append(&mut self.input.borrow().inputs());
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.borrow().inputs());
        }
        inputs
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        let mut count = match self.count {
            Some(count) => count,
            None => {
                let val = if let Some(initial) = &self.initial {
                    let Some(initial) = initial.borrow_mut().out()? else { return Ok(None) };
                    initial.into_value().to_int().as_int().value
                } else {
                    0
                };
                self.count = Some(val);
                val
            }
        };
        if !flow.is_new() {
            return flow.wrap(Self::point_with(&input, &self.id, count));
        };
        let val = input.to_bool().as_bool().value.0;
        let Some(edge) = self.edge.add(val) else {
            return flow.wrap_old(Self::point_with(&input, &self.id,count));
        };
        if edge.is_rising() {
            count += 1;
            self.count = Some(count);
            log::trace!("{}.out | value: {:?}", self.id, count);
            flow.wrap(Self::point_with(&input, &self.id, count))
        } else {
            log::trace!("{}.out | value: {:?}", self.id, count);
            flow.wrap_old(Self::point_with(&input, &self.id, count))
        }
    }
    fn reset(&mut self) {
        self.count = None;
        self.edge.reset();
        if let Some(initial) = &self.initial { initial.borrow_mut().reset(); };
        self.input.borrow_mut().reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_debug {
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
#[derive(Debug)]
pub struct FnDebug {
    id: String,
    kind: FnKind,
    inputs: Vec<(String, FnOutRef)>,
}
impl FnDebug {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: impl IntoIterator<Item = (String, FnOutRef)>) -> Self {
        Self {
            id: format!("{}/FnDebug{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            inputs: inputs.into_iter().collect(),
        }
    }
}
impl FnOut for FnDebug {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        self.inputs.iter()
            .flat_map(|(_, input)| input.borrow().inputs())
            .collect()
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let flow = FlowContext::new();
        for (name, input) in &self.inputs {
            match flow.ignore(input.borrow_mut().out()) {
                Ok(Some(v)) => {
                    log::debug!(
                        "{}.out | Value {} | {}:{}\n  └─ Val: {:?} | {:?} | {:?} | {}",
                        self.id, flow, v.txid(), v.name(), v.value(), v.status(), v.cot(), v.ts().format("%H:%M:%S%.3f")
                    );
                }
                Ok(None) => log::error!("{}.out | None on input '{}'", self.id, name),
                Err(err) => log::error!("{}.out | Error on input '{}': {:?}", self.id, name, err),
            }
        }
        Ok(None)
    }
    fn reset(&mut self) {
        for (_, input) in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_is_changed_value {
use sal_sync::{
    collections::FxHashMap,
    services::{entity::{Point, PointHlr, PointTxId},
    types::Bool,
}};
use testing::entities::test_value::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::{FnOutRef, PointMeta}, services::task::{FlowContext, FnFlow}};
use crate::services::task::{FnOut, FnKind, FnResult};
#[derive(Debug)]
pub struct FnIsChangedValue {
    id: String,
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    state: FxHashMap<String, Value>,
    prev: Option<bool>,
}
impl FnIsChangedValue {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Self {
        let id = format!("{}/FnIsChangedValue{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let txid = PointTxId::from_str(&id);
        Self {
            id,
            txid,
            kind: FnKind::Fn,
            inputs,
            state: FxHashMap::default(),
            prev: None,
        }
    }
}
impl FnOut for FnIsChangedValue {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let mut fb_meta = PointMeta::default();
        let mut meta = None::<PointMeta>;
        let mut val = false;
        let mut has_active = false;
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter()
            .map(|input| input.borrow_mut().out()).collect();
        for input in inputs {
            if let Some(point) = flow.map(input)? {
                has_active = true;
                fb_meta = fb_meta.update_latest(&point);
                let key = point.name();
                log::trace!("{}.out | input '{}': {:?}", self.id, key, point);
                if let Some(state) = self.state.get_mut(&key) {
                    let value = point.value();
                    if value != *state {
                        log::trace!("{}.out | changed: {}  |  state '{:?}', value: {:?}", self.id, key, state, value);
                        meta = Some(meta.unwrap_or_default().update_latest(&point));
                        *state = value;
                        val = true;
                    }
                } else {
                    meta = Some(meta.unwrap_or_default().update_latest(&point));
                    self.state.insert(key, point.value());
                    val = true;
                }
            }
        }
        if !has_active {
            self.prev = None;
            return Ok(None);
        }
        let meta = meta.unwrap_or(fb_meta);
        let value = Point::Bool(PointHlr::new(
            self.txid,
            &self.id,
            Bool(val),
            meta.status,
            meta.cot,
            meta.ts,
        ));
        let is_changed = val || self.prev == Some(true);
        self.prev = Some(val);
        if is_changed {
            log::trace!("{}.out | FlowNew | value {:?} | {:?}", self.id, flow, value);
            return flow.wrap_new(value);
        }
        log::trace!("{}.out | FlowOld | value {:?} | {:?}", self.id, flow, value);
        flow.wrap_old(value)
    }
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
        self.state.clear();
        self.prev = None;
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_hold {
use sal_sync::services::entity::Point;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::services::task::FnChange;
use crate::{
    domain::{FnOutRef},
    services::task::{FlowContext, FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
#[derive(Debug)]
pub struct FnHold {
    id: String,
    kind: FnKind,
    input: FnChange,
    state: Option<Point>,
}
impl FnHold {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnHold{}", parent.into(), COUNT.fetch_add(1, Ordering::AcqRel)),
            kind: FnKind::Fn,
            input: FnChange::new(input),
            state: None,
        }
    }
}
impl FnOut for FnHold {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        self.input.inputs()
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = flow.map(self.input.out())?;
        log::trace!("{}.out | input: {:?}", self.id, input);
        if let Some(point) = input {
            self.state = Some(point.clone());
            return flow.wrap(point);
        }
        if let Some(point) = &self.state {
            return flow.wrap_old(point.clone());
        }
        Ok(None)
    }
    fn reset(&mut self) {
        self.state = None;
        self.input.reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_max {
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use sal_sync::services::entity::{Point, PointHlr, PointType};
use sal_sync::services::types::Bool;
use crate::domain::{Edge, EdgeDetector, FnOutRef};
use crate::services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult};
#[derive(Debug)]
pub struct FnMax {
    id: String,
    kind: FnKind,
    reset: Option<FnChange>,
    input: FnChange,
    max: Option<f64>,
    reset_edge: EdgeDetector,
}
impl FnMax {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnMax{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            reset: reset.map(FnChange::new),
            input: FnChange::new(input),
            max: None,
            reset_edge: EdgeDetector::new(),
        }
    }
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
    #[inline]
    fn point(id: &str, input: &Point, val: f64) -> Result<Point, String> {
        match input.typ() {
            PointType::Bool => Ok(Point::Bool(Self::point_with(input, id, Bool(val != 0.0)))),
            PointType::Int => Ok(Point::Int(Self::point_with(input, id, val.round() as i64))),
            PointType::Real => Ok(Point::Real(Self::point_with(input, id, val as f32))),
            PointType::Double => Ok(Point::Double(Self::point_with(input, id, val))),
            _ => Err(concat_string!(id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        }
    }
}
impl FnOut for FnMax {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        let mut inputs = self.input.inputs();
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.inputs());
        }
        inputs
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let mut force_recalc = false;
        let input = self.input.out();
        let reset = self.reset.as_mut().map(|f| f.out());
        if let Some(reset) = reset {
            if let Some(reset) = reset? {
                if let Some(Edge::Rising) = self.reset_edge.add(reset.into_value().to_bool().as_bool().value.0) {
                    self.max = None;
                    force_recalc = true;
                }
            }
        }
        let Some(input) = flow.map(input)? else { return Ok(None) };
        if !flow.is_new() && !force_recalc {
            let Some(max) = self.max else { return Ok(None) };
            return flow.wrap_old(Self::point(&self.id, &input, max)?);
        }
        let value = match input.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        let was_none = self.max.is_none();
        let max = *self.max.get_or_insert(value);
        if value > max {
            self.max = Some(value);
            log::trace!("{}.out | max: {:?}", self.id, self.max);
            let max = Self::point(&self.id, &input, value)?;
            flow.wrap_new(max)
        } else if was_none || force_recalc {
            log::trace!("{}.out | max: {:?}", self.id, self.max);
            let max = Self::point(&self.id, &input, max)?;
            flow.wrap_new(max)
        } else {
            log::trace!("{}.out | max: {:?}", self.id, self.max);
            let max = Self::point(&self.id, &input, max)?;
            flow.wrap_old(max)
        }
    }
    fn reset(&mut self) {
        self.max = None;
        self.input.reset();
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.reset_edge.reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_min {
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use sal_sync::services::entity::{Point, PointHlr, PointType};
use sal_sync::services::types::Bool;
use crate::domain::{Edge, EdgeDetector, FnOutRef};
use crate::services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult};
#[derive(Debug)]
pub struct FnMin {
    id: String,
    kind: FnKind,
    reset: Option<FnChange>,
    input: FnChange,
    min: Option<f64>,
    reset_edge: EdgeDetector,
}
impl FnMin {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnMin{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            reset: reset.map(FnChange::new),
            input: FnChange::new(input),
            min: None,
            reset_edge: EdgeDetector::new(),
        }
    }
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
    #[inline]
    fn point(id: &str, input: &Point, val: f64) -> Result<Point, String> {
        match input.typ() {
            PointType::Bool => Ok(Point::Bool(Self::point_with(input, id, Bool(val != 0.0)))),
            PointType::Int => Ok(Point::Int(Self::point_with(input, id, val.round() as i64))),
            PointType::Real => Ok(Point::Real(Self::point_with(input, id, val as f32))),
            PointType::Double => Ok(Point::Double(Self::point_with(input, id, val))),
            _ => Err(concat_string!(id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        }
    }
}
impl FnOut for FnMin {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn kind(&self) -> FnKind {
        self.kind
    }
    fn inputs(&self) -> Vec<String> {
        let mut inputs = self.input.inputs();
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.inputs());
        }
        inputs
    }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let mut force_recalc = false;
        let input = self.input.out();
        let reset = self.reset.as_mut().map(|f| f.out());
        if let Some(reset) = reset {
            if let Some(reset) = reset? {
                if let Some(Edge::Rising) = self.reset_edge.add(reset.into_value().to_bool().as_bool().value.0) {
                    self.min = None;
                    force_recalc = true;
                }
            }
        }
        let Some(input) = flow.map(input)? else { return Ok(None) };
        if !flow.is_new() && !force_recalc {
            let Some(min) = self.min else { return Ok(None) };
            return flow.wrap_old(Self::point(&self.id, &input, min)?);
        }
        let value = match input.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        let was_none = self.min.is_none();
        let min = *self.min.get_or_insert(value);
        if value < min {
            self.min = Some(value);
            log::trace!("{}.out | min: {:?}", self.id, self.min);
            let min = Self::point(&self.id, &input, value)?;
            flow.wrap_new(min)
        } else if was_none || force_recalc {
            log::trace!("{}.out | min: {:?}", self.id, self.min);
            let min = Self::point(&self.id, &input, min)?;
            flow.wrap_new(min)
        } else {
            log::trace!("{}.out | min: {:?}", self.id, self.min);
            let min = Self::point(&self.id, &input, min)?;
            flow.wrap_old(min)
        }
    }
    fn reset(&mut self) {
        self.min = None;
        self.input.reset();
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.reset_edge.reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_piecewise_line_approx {
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{entity::{Point, PointHlr, PointType}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, functions::{
        FnKind, FnOut, FnResult
    }}
};
#[derive(Debug)]
pub struct FnPiecewiseLineApprox {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    piecewise: PiecewiseLinear,
}
impl FnPiecewiseLineApprox {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef, piecewise: PiecewiseLinear) -> Self {
        let self_id = format!("{}/FnPiecewiseLineApprox{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        Self {
            id: self_id,
            kind: FnKind::Fn,
            input,
            piecewise,
        }
    }
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
    #[inline]
    fn point(id: &str, input: &Point, val: f64) -> Result<Point, String> {
        match input.typ() {
            PointType::Bool => Ok(Point::Bool(Self::point_with(input, id, Bool(val != 0.0)))),
            PointType::Int => Ok(Point::Int(Self::point_with(input, id, val.round() as i64))),
            PointType::Real => Ok(Point::Real(Self::point_with(input, id, val as f32))),
            PointType::Double => Ok(Point::Double(Self::point_with(input, id, val))),
            PointType::String => Ok(Point::String(Self::point_with(input, id, val.to_string()))),
            _ => Err(concat_string!(id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        }
    }
}
impl FnOut for FnPiecewiseLineApprox {
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
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:?}", self.id, input);
        let value: f64 = match &input {
            Point::Bool(_) | Point::Int(_) | Point::Real(_) | Point::Double(_) => input.to_double().as_double().value,
            Point::String(val) => {
                val.value.parse()
                    .map_err(|_| concat_string!(self.id, ".out | Invalid input '", val.value, "'"))?
            }
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        if value.is_nan() {
            return Err(concat_string!(self.id, ".out | Math error: Received NaN value instead of valid number"));
        }
        let val = match self.piecewise.eval(value) {
            Some(v) => v,
            None => return Ok(None),
        };
        let out = Self::point(&self.id, &input, val)?;
        log::trace!("{}.out | out: {:?}", self.id, &out);
        flow.wrap(out)
    }
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(1);
#[derive(Debug, Copy, Clone)]
struct LinePoint {
    x: f64, y: f64
}
impl LinePoint {
    #[inline]
    fn new(x: f64, y: f64) -> Self {
        Self {x, y}
    }
}
#[derive(Debug)]
struct LinearApprox {
    left: LinePoint,
    right: LinePoint,
    k: f64,
    b: f64,
}
impl LinearApprox {
    fn try_new(p1: &LinePoint, p2: &LinePoint) -> Result<Self, Error> {
        if (p2.x - p1.x).abs() <= f64::MIN_POSITIVE {
            return Err(Error::new("LinearApprox", "try_new")
                .err(format!("Can't create linear approximation: vertical line (x1 {} and x2 {} coordinates are equal)", p1.x, p2.x)));
        }
        let k = (p2.y - p1.y) / (p2.x - p1.x);
        if k.is_nan() || k.is_infinite() {
            return Err(Error::new("LinearApprox", "try_new")
                .err(format!("Can't create linear approximation, invalid factor k: {k}")));
        }
        let b = p1.y - p1.x * k;
        if b.is_nan() || b.is_infinite() {
            return Err(Error::new("LinearApprox", "try_new")
                .err(format!("Can't create linear approximation, invalid number b: {b}")));
        }
        Ok(Self {
            left: *p1,
            right: *p2,
            k,
            b,
        })
    }
    #[inline]
    fn eval(&self, x: f64) -> f64 {
        self.k * x + self.b
    }
    #[inline]
    fn is_out_of_left(&self, x: f64) -> bool {
        x < self.left.x
    }
    #[inline]
    fn is_out_of_right(&self, x: f64) -> bool {
        x > self.right.x
    }
}
#[derive(Debug)]
pub struct PiecewiseLinear {
    id: Dbg,
    lines: Vec<LinearApprox>,
}
impl PiecewiseLinear {
    const IS_EMPTY_MSG: &'static str = "Piecewise function must contains at least two points";
    ///
    /// Creates new instance of the [Linears]
    /// - `parent` - Parent entity identifier
    /// - `pairs` - Points of piecewise-line function
    pub fn try_new(parent: impl Into<String>, mut pairs: Vec<LinePoint>) -> Result<Self, Error> {
        let dbg = Dbg::new(parent, "Linears");
        let error = Error::new(&dbg, "try_new");
        if pairs.len() < 2 {
            return Err(error.err(Self::IS_EMPTY_MSG));
        }
        for p in &pairs {
            if p.x.is_nan() || p.y.is_nan() {
                return Err(error.err("Piecewise points contains NAN instead of numbers"));
            }
        }
        pairs.sort_by(|a, b| a.x.total_cmp(&b.x));
        let has_duplicates = pairs.windows(2).any(|w| (w[0].x - w[1].x).abs() < f64::EPSILON);
        if has_duplicates {
            return Err(error.err(format!("Piecewise points contains duplicates in `x`")));
        }
        let mut lines = Vec::with_capacity(pairs.len() - 1);
        for pp in pairs.windows(2) {
            lines.push(
                LinearApprox::try_new(&pp[0], &pp[1])
                    .map_err(|err| error.pass(err))?
            );
        }
        Ok(Self { id: dbg, lines })
    }
    ///
    /// ### Returns `Linears` parsed from YAML
    ///
    /// Expected following YAML format:
    /// ```yaml
    /// x1: y1
    /// x2: y2
    /// x3: y3
    /// ...
    /// ```
    pub fn from_yaml(parent: impl Into<String>, points: &serde_yaml::Value) -> Result<Self, Error> {
        let parent = parent.into();
        let dbg = Dbg::new(&parent, "Linears");
        let error = Error::new(&dbg, "from_yaml");
        let points: serde_yaml::Mapping = serde_yaml::from_value(points.clone())
            .map_err(|err| error.pass_with(format!("Can't parse points from {:?}", points), err.to_string()))?;
        // let points: Vec<(serde_yaml::Value, serde_yaml::Value)> = points.into_iter().collect();
        if points.len() < 2 {
            return Err(error.err(Self::IS_EMPTY_MSG));
        }
        let mut pairs = Vec::with_capacity(points.len());
        for (x, y) in points {
            let x = x.as_f64()
                .ok_or(error.err(format!("Can't parse {:?} as f64", x)))?;
            let y = y.as_f64()
                .ok_or(error.err(format!("Can't parse {:?} as f64", y)))?;
        pairs.push(LinePoint::new(x, y));
        }
        Self::try_new(parent, pairs)
    }
    ///
    /// ### Evaluates the piecewise linear function at point `x`.
    ///
    /// If `x` is less than the first point's `x`, returns the first point's `y`.
    /// If `x` is greater than the last point's `x`, returns the last point's `y`.
    /// Otherwise, finds the segment containing `x` and performs linear interpolation.
    ///
    /// **Examples**
    /// ```ignore
    /// let points = vec![LinePoint::new(0.0, 0.0), LinePoint::new(1.0, 1.0)];
    /// let func = PiecewiseLinear::try_new("test", points).unwrap();
    /// assert_eq!(func.eval(0.5), Some(0.5));
    /// assert_eq!(func.eval(-1.0), Some(0.0)); // extrapolation
    /// ```
    pub fn eval(&self, x: f64) -> Option<f64> {
        log::trace!("{}.line_approx | value: {:?}", self.id, x);
        // Проверка выхода за левую границу (экстраполяция/const)
        if let Some(first) = self.lines.first() {
            if first.is_out_of_left(x) {
                log::trace!("{}.line_approx | less then first: {:#?}", self.id, first);
                return Some(first.left.y);
            }
        }
        // Проверка выхода за правую границу (экстраполяция/const)
        if let Some(last) = self.lines.last() {
            if last.is_out_of_right(x) {
                log::trace!("{}.line_approx | Greater then last: {:#?}", self.id, last);
                return Some(last.right.y);
            }
        }
        self.lines.iter().find_map(|line| {
            if x >= line.left.x && x <= line.right.x {
                return Some(line.eval(x))
            }
            None
        })
        // let res = self.lines.binary_search_by(|probe| {
        //     if x < probe.left.x {
        //         std::cmp::Ordering::Greater
        //     } else if x >= probe.right.x {
        //         std::cmp::Ordering::Less
        //     } else {
        //         std::cmp::Ordering::Equal
        //     }
        // });
        // match res {
        //     Ok(i) => Some(self.lines[i].eval(x)),
        //     Err(_) => None
        // }
    }
}
///
/// `PiecewiseLinear` Basic Tests
}
mod fn_point_id {
use sal_sync::{collections::FxIndexMap, services::entity::{Point, PointConf, PointHlr, PointTxId}};
use std::{sync::atomic::{AtomicUsize, Ordering}};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, functions::{FnKind, FnOut, FnResult}},
};
///
/// ### Function | `FnPointId`
///
/// Возвращает ID входного сигнала по его имени из `RetainPointId`
/// - Является прозрачным узлом: сохраняет все метаданные входа (`Cot`, `Status`, `Timestamp`)
/// - Передает состояние потока (New/Old) без изменений.
///
/// **Example**
/// ```yaml
/// fn PointId:
///     input: point int /App/PointName
/// ```
#[derive(Debug)]
pub struct FnPointId {
    txid: usize,
    kind: FnKind,
    input: FnOutRef,
    points: FxIndexMap<String, usize>,
    id: String,
}
//
impl FnPointId {
    ///
    /// Returns `FnPointId` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `input`: Входной сигнал (например `point any every`)
    /// - `points`: Список сигналов из `RetainPointId`
    pub fn new(parent: impl Into<String>, input: FnOutRef, points: Vec<PointConf>) -> Self {
        let id = format!("{}/FnPointId{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            input,
            points: points.into_iter().map(|p| (p.name, p.id)).collect(),
            id,
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, name: impl Into<String>, p: &Point, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, p.status(), p.cot(), p.timestamp())
    }
}
//
impl FnOut for FnPointId {
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
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:?}", self.id, input);
        let Some(id) = self.points.get(&input.name()) else {
            return Err(concat_string!(self.id, ".out | Point '", input.name(), "' - not found in configured points"));
        };
        // log::debug!("{}.out | ID: {:?}", self.id, id);
        flow.wrap(Point::Int(Self::point_with(self.txid, &self.id, &input, *id as i64)))
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
pub use fn_acc::*;
pub use fn_average::*;
pub use fn_count::*;
pub use fn_debug::*;
pub use fn_is_changed_value::*;
pub use fn_hold::*;
pub use fn_max::*;
pub use fn_min::*;
pub use fn_piecewise_line_approx::*;
pub use fn_point_id::*;
}
mod comp {
//! Comparison functions compare two or more variables returning a bool Point containing TRUE or FALSE.
//!
//!  Function | Operator | Description
//! :-------:|:--------:|-------------
//!   Gt     |    >     | Greater than
//!   Ge     |    >=    | Greater than or equal to
//!   Eq     |    =     | Equal
//!   Le     |    <=    | Less than or equal to
//!   Lt     |    <     | Less than
//!   Ne     |    <>    | Not equal to
//!
//! Example
//!
//! `Point.A >= 0.5`
//!
//! ```yaml
//! fn Ge:
//!     input1: point real /App/Service/Point.A
//!     input2: const real 0.5
//! ```
mod fn_gt {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnGt`
///
/// Выполняет операцию логического сравнения GT (Greater Than) `v1 > v2`
/// Динамически приводит типы данных.
///
/// **Example**
/// ```yaml
/// fn Gt:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Gt:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnGt {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
impl FnGt {
    ///
    /// Returns `FnGt` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnGt{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnGt {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let value = v1 > v2;
        flow.wrap(Point::Bool(Self::point_with(self.txid, &meta, &self.id, Bool(value))))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnGt instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_ge {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnGe`
///
/// Выполняет операцию логического сравнения GE (Greater or Equal) `v1 >= v2`
/// Динамически приводит типы данных.
///
/// **Example**
/// ```yaml
/// fn Ge:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Ge:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnGe {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
impl FnGe {
    ///
    /// Returns `FnGe` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnGe{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnGe {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let value = v1 >= v2;
        flow.wrap(Point::Bool(Self::point_with(self.txid, &meta, &self.id, Bool(value))))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnGe instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_eq {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnEq`
///
/// Выполняет операцию логического сравнения EQ (Equal) `v1 == v2`
/// Динамически приводит типы данных.
///
/// **Example**
/// ```yaml
/// fn Eq:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Eq:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnEq {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
impl FnEq {
    ///
    /// Returns `FnEq` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnEq{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnEq {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let value = v1 == v2;
        flow.wrap(Point::Bool(Self::point_with(self.txid, &meta, &self.id, Bool(value))))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnEq instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_le {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnLe`
///
/// Выполняет операцию логического сравнения LE (Less or Equal) `v1 <= v2`
/// Динамически приводит типы данных.
///
/// **Example**
/// ```yaml
/// fn Le:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Le:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnLe {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
impl FnLe {
    ///
    /// Returns `FnLe` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnLe{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnLe {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let value = v1 <= v2;
        flow.wrap(Point::Bool(Self::point_with(self.txid, &meta, &self.id, Bool(value))))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnLe instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_lt {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnLt`
///
/// Выполняет операцию логического сравнения LT (Less Than) `v1 < v2`
/// Динамически приводит типы данных.
///
/// **Example**
/// ```yaml
/// fn Lt:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Lt:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnLt {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
impl FnLt {
    ///
    /// Returns `FnLt` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnLt{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnLt {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let value = v1 < v2;
        flow.wrap(Point::Bool(Self::point_with(self.txid, &meta, &self.id, Bool(value))))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnLt instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_ne {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnNe`
///
/// Выполняет операцию логического сравнения NE (Not Equal) `v1 != v2`
/// Динамически приводит типы данных.
///
/// **Example**
/// ```yaml
/// fn Ne:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Ne:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnNe {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
impl FnNe {
    ///
    /// Returns `FnNe` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnNe{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnNe {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let value = v1 != v2;
        flow.wrap(Point::Bool(Self::point_with(self.txid, &meta, &self.id, Bool(value))))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnNe instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
pub use fn_gt::*;
pub use fn_ge::*;
pub use fn_eq::*;
pub use fn_le::*;
pub use fn_lt::*;
pub use fn_ne::*;}
mod conversion {
//!
//! `Task` Service functions intended for the type conversions
//!
mod fn_to_int {
use sal_sync::services::entity::{Point, PointHlr};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// ### Function | FnToInt
///
/// Converts input to Int
///  - bool: true -> 1, false -> 0
///  - real: 0.1 -> 0 | 0.5 -> 1 | 0.9 -> 1 | 1.1 -> 1
///  - string: try to parse int
#[derive(Debug)]
pub struct FnToInt {
    id: String,
    kind: FnKind,
    input: FnOutRef,
}
//
impl FnToInt {
    ///
    /// Creates new instance of the `FnToInt`
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnToInt{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }
}
//
impl FnOut for FnToInt {
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
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:#?}", self.id, input);
        let value: i64 = match &input {
            Point::Bool(_) | Point::Int(_) | Point::Real(_) | Point::Double(_) => input.to_int().as_int().value,
            Point::String(val) => {
                val.value.parse()
                    .map_err(|_| concat_string!(self.id, ".out | Invalid input '", val.value, "'"))?
            }
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        flow.wrap(Point::Int(PointHlr::new(
            input.txid(),
            &self.id,
            value,
            input.status(),
            input.cot(),
            input.ts(),
        )))
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToInt instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_to_real {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr}, types::DebugTypeOf};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// Function | Converts input to Real
///  - bool: true -> 1.0, false -> 0.0
///  - string: try to parse Real
#[derive(Debug)]
pub struct FnToReal {
    kind: FnKind,
    input: FnOutRef,
    id: String,
}
//
impl FnToReal {
    ///
    /// Creates new instance of the FnToReal
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnToReal{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            kind: FnKind::Fn,
            input,
            id,
        })
    }
}
//
impl FnOut for FnToReal {
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
        unimplemented!();
        // let mut flow = FlowContext::new();
        // let input = self.input.borrow_mut().out();
        // log::trace!("{}.out | input: {:?}", self.id, input);
        // match input {
        //     FnResult::Ok(input) => {
        //         let out = match &input {
        //             Point::Bool(value) => {
        //                 if value.value.0 {1.0f32} else {0.0f32}
        //             }
        //             Point::Int(value) => {
        //                 value.value as f32
        //             }
        //             Point::Real(value) => {
        //                 value.value
        //             }
        //             Point::Double(value) => {
        //                 value.value as f32
        //             }
        //             _ => panic!("{}.out | {:?} type is not supported: {:?}", self.id, input.print_type_of(), input),
        //         };
        //         log::trace!("{}.out | out: {:?}", self.id, &out);
        //         FnResult::Ok(Point::Real(
        //             PointHlr::new(
        //                 input.txid(),
        //                 &concat_string!(self.id, ".out"),
        //                 out,
        //                 input.status(),
        //                 input.cot(),
        //                 input.timestamp(),
        //             )
        //         ))
        //     }
        //     FnResult::None => FnResult::None,
        //     FnResult::Err(err) => FnResult::Err(err),
        // }
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToReal instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_to_double {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr}, types::DebugTypeOf};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// Function | Converts input to Double
///  - bool: true -> 1.0, false -> 0.0
///  - string: try to parse double
#[derive(Debug)]
pub struct FnToDouble {
    kind: FnKind,
    input: FnOutRef,
    id: String,
}
//
impl FnToDouble {
    ///
    /// Creates new instance of the FnToDouble
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnToDouble{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            kind: FnKind::Fn,
            input,
            id,
        })
    }
}
//
//
impl FnOut for FnToDouble {
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
        unimplemented!();
        // let mut flow = FlowContext::new();
        // let input = self.input.borrow_mut().out();
        // log::trace!("{}.out | input: {:?}", self.id, input);
        // match input {
        //     FnResult::Ok(input) => {
        //         let out = match &input {
        //             Point::Bool(value) => {
        //                 if value.value.0 {1.0f64} else {0.0f64}
        //             }
        //             Point::Int(value) => {
        //                 value.value as f64
        //             }
        //             Point::Real(value) => {
        //                 value.value as f64
        //             }
        //             Point::Double(value) => {
        //                 value.value
        //             }
        //             _ => panic!("{}.out | {:?} type is not supported: {:?}", self.id, input.print_type_of(), input),
        //         };
        //         log::trace!("{}.out | out: {:?}", self.id, &out);
        //         FnResult::Ok(Point::Double(
        //             PointHlr::new(
        //                 input.txid(),
        //                 &concat_string!(self.id, ".out"),
        //                 out,
        //                 input.status(),
        //                 input.cot(),
        //                 input.timestamp(),
        //             )
        //         ))
        //     }
        //     FnResult::None => FnResult::None,
        //     FnResult::Err(err) => FnResult::Err(err),
        // }
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToDouble instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_to_bool {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr}, types::{Bool, DebugTypeOf}};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// Function | Converts input to Bool
///  - bool: true -> 1, false -> 0
///  - real: 0.1 -> 0 | 0.5 -> 1 | 0.9 -> 1 | 1.1 -> 1
///  - string: try to parse bool
#[derive(Debug)]
pub struct FnToBool {
    kind: FnKind,
    input: FnOutRef,
    id: String,
}
//
impl FnToBool {
    ///
    /// Creates new instance of the FnToBool
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnToBool{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            kind: FnKind::Fn,
            input,
            id,
        })
    }
}
//
impl FnOut for FnToBool {
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
        unimplemented!();
        // let mut flow = FlowContext::new();
        // let input = self.input.borrow_mut().out();
        // match input {
        //     FnResult::Ok(input) => {
        //         log::trace!("{}.out | input: {:?}", self.id, input);
        //         let out = match &input {
        //             Point::Bool(value) => {
        //                 value.value.0
        //             }
        //             Point::Int(value) => {
        //                 value.value > 0
        //             }
        //             Point::Real(value) => {
        //                 value.value > 0.0
        //             }
        //             Point::Double(value) => {
        //                 value.value > 0.0
        //             }
        //             Point::Bytes(value) => {
        //                 if value.value.len() > 0 {
        //                     value.value[0] != 0
        //                 } else {
        //                     false
        //                 }
        //             }
        //             _ => panic!("{}.out | {:?} type is not supported: {:?}", self.id, input.print_type_of(), input),
        //         };
        //         log::trace!("{}.out | out: {:?}", self.id, &out);
        //         FnResult::Ok(Point::Bool(
        //             PointHlr::new(
        //                 input.txid(),
        //                 &concat_string!(self.id, ".out"),
        //                 Bool(out),
        //                 input.status(),
        //                 input.cot(),
        //                 input.timestamp(),
        //             )
        //         ))
        //     }
        //     FnResult::None => FnResult::None,
        //     FnResult::Err(err) => FnResult::Err(err),
        // }
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToBool instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_to_string {
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// Function | Converts input to String
#[derive(Debug)]
pub struct FnToString {
    id: String,
    kind: FnKind,
    input: FnOutRef,
}
//
impl FnToString {
    ///
    /// Creates new instance of the FnToString
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnToString{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            kind: FnKind::Fn,
            input,
            id,
        })
    }
}
//
impl FnOut for FnToString {
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
        unimplemented!();
        // let mut flow = FlowContext::new();
        // let input = self.input.borrow_mut().out();
        // log::trace!("{}.out | input: {:?}", self.id, input);
        // match input {
        //     FnResult::Ok(input) => {
        //         let out = match &input {
        //             Point::Bool(value) => &value.value.0.to_string(),
        //             Point::Int(value) => &value.value.to_string(),
        //             Point::Real(value) => &value.value.to_string(),
        //             Point::Double(value) => &value.value.to_string(),
        //             Point::String(value) => &value.value,
        //             Point::Bytes(value) => &value.to_string().value,
        //         };
        //         log::trace!("{}.out | out: {:?}", self.id, &out);
        //         FnResult::Ok(Point::String(
        //             PointHlr::new(
        //                 input.txid(),
        //                 &concat_string!(self.id, ".out"),
        //                 out.to_owned(),
        //                 input.status(),
        //                 input.cot(),
        //                 input.timestamp(),
        //             )
        //         ))
        //     }
        //     FnResult::None => FnResult::None,
        //     FnResult::Err(err) => FnResult::Err(err),
        // }
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToString instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
pub use fn_to_int::*;
pub use fn_to_real::*;
pub use fn_to_double::*;
pub use fn_to_bool::*;
pub use fn_to_string::*;}
mod core {
//!
//! `Task` Service | Functions common interfaces
//!
mod fn_change {
use crate::{domain::FnOutRef, services::task::{FnFlow, FnKind, FnOut, FnResult}};
use sal_sync::services::entity::Point;
use std::fmt::Debug;
///
/// ### Function | FnChange Decorator
///
/// Декоратор для контроля изменений потока данных.
/// Пропускает через себя вычисления внутреннего узла, но понижает статус `FnFlow::New` до `FnFlow::Old`,
/// если фактическое значение и статус качества точки остались неизменными.
#[derive(Debug)]
pub struct FnChange {
    input: FnOutRef,
    last_val: Option<Point>,
}
impl FnChange {
    /// Создает новый экземпляр декоратора FnChange.
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
            // Перехватываем всё, что содержит значение (и New, и Old)
            Some(FnFlow::New(point)) | Some(FnFlow::Old(point)) => {
                let is_changed = match &self.last_val {
                    Some(last) => {
                        // Смена статуса (например, Ok -> Invalid) так же важна, как и смена значения
                        last.value() != point.value() || last.status() != point.status()
                    },
                    None => true,
                };
                if is_changed {
                    // Значение изменилось! Принудительно генерируем New,
                    // даже если источник ошибочно или лениво прислал Old.
                    self.last_val = Some(point.clone());
                    Ok(Some(FnFlow::New(point)))
                } else {
                    // Значение старое. Принудительно гасим в Old,
                    // даже если источник спамит New.
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
///
/// Basic tests
}
mod fn_const {
use std::sync::atomic::{Ordering, AtomicUsize};
use sal_sync::services::entity::Point;
use crate::services::task::FnFlow;
use super::{FnOut, FnKind, FnResult};
///
/// Function | Constant value
#[derive(Debug, Clone)]
pub struct FnConst {
    id: String,
    kind: FnKind,
    point: Point,
}
//
//
impl FnConst {
    ///
    /// Creates new instance of function [Const] value
    ///     - [parent] - name of the parent object
    ///     - [value] - PointType, contains point with constant value
    pub fn new(parent: &str, value: Point) -> Self {
        Self {
            id: format!("{}/FnConst{}", parent, COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Input,
            point: value
        }
    }
}
//
//
impl FnOut for FnConst {
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
        vec![]
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        log::trace!("{}.out | value: {:?}", self.id, &self.point);
        Ok(Some(FnFlow::Old(self.point.clone())))
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnConst instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_enable {
use sal_sync::services::entity::Point;
use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow}};
use super::{FnOut, FnKind, FnResult};
///
/// ### Function | Enable Decorator
///
/// Декоратор для управления активностью вычислительного узла.
///
/// Оборачивает любой узел, реализующий `FnOut`, добавляя логику включения/выключения
/// на основе управляющего сигнала `enable`.
///
/// Режимы работы:
/// - `Cold`: При выключении внутренний узел сбрасывается (`reset()`). При включении начинает работу с чистого листа.
/// - `Warm`: Внутренний узел вычисляется всегда (поддерживая актуальное состояние), но наружу значение передается только при активном сигнале.
/// - **`Cold`**: При выключении (`enable = false`) внутренний узел сбрасывается (`reset()`), при включении начинает работу с чистого листа.
/// - **`Warm`**: Внутренний узел вычисляется всегда (поддерживая актуальное состояние), но наружу значение передается только при `enable = true`.
///
/// > **Если enable не привязан или молчит (`None`), по умолчанию используем `fals` или предыдущее значение**
#[derive(Debug, Clone)]
pub struct FnEnable<T: FnOut> {
    origin: T,
    mode: FnEnableMode,
    enable: FnOutRef,
    prev_en: bool,
    last_val: Option<Point>,
}
//
//
impl<T: FnOut> FnEnable<T> {
    ///
    /// Creates a new instance of the FnEnable decorator
    /// - `origin` - Оборачиваемая вычислительная функция (`FnXyz`).
    /// - `mode` - Режим работы при отключении сигнала (Cold / Warm).
    /// - `en` - Ссылка на `enable`, выдающий логический сигнал активности.
    pub fn new(origin: T, mode: FnEnableMode, en: FnOutRef) -> Self {
        Self {
            origin,
            mode,
            enable: en,
            prev_en: false,
            last_val: None,
        }
    }
}
//
impl<T: FnOut> FnOut for FnEnable<T> {
    //
    fn id(&self) -> String {
        self.origin.id()
    }
    //
    fn kind(&self) -> FnKind {
        self.origin.kind()
    }
    //
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        inputs.append(&mut self.enable.borrow().inputs());
        inputs.append(&mut self.origin.inputs());
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let en = match self.enable.borrow_mut().out()? {
            Some(en) => en.into_value().to_bool().as_bool().value.0,
            None => self.prev_en, // Если enable не приходит (None), то используем прежнее значение
        };
        // Детектируем передний фронт (false -> true)
        let rising_edge = !self.prev_en && en;
        // Детектируем задний фронт (true -> false)
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
                        self.last_val = None; // Очищаем стейт для чистоты
                    }
                    // Cold означает полное отсутствие сигнала при выключении
                    Ok(None)
                }
            }
            FnEnableMode::Warm => {
                let mut flow = FlowContext::new();
                // Всегда дергаем оригинал, чтобы кэш/математика внутри оставались актуальными
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
    //
    fn reset(&mut self) {
        self.prev_en = false;
        self.last_val = None;
        self.enable.borrow_mut().reset();
        self.origin.reset();
    }
}
///
/// ### Enable Strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnEnableMode {
    /// **Cold Standby**: При enable = false внутреннее состояние полностью уничтожается.
    Cold,
    /// **Warm Standby**: Математика работает всегда, но enable работает как заслонка.
    Warm,
}
///
/// Basic tests
}
mod fn_flow {
use std::fmt::{Debug, Display};
use sal_sync::services::entity::Point;
use crate::services::task::FnResult;
///
/// ### Контракт потока данных
///
/// - Оборачивает `Point` в вычислениях `Task`.
/// - Определяет статус входных и выходных значений.
///     - `New` - Точка была обновлена в текущем цикле вычислений (пришел эвент)
///     - `Old` - Точка взята из кэша, в этом цикле эвента для нее не было
#[derive(Debug, Clone)]
pub enum FnFlow {
    /// Точка была обновлена в текущем цикле вычислений (пришел эвент)
    New(Point),
    /// Точка взята из кэша, в этом цикле эвента для нее не было
    Old(Point),
}
impl FnFlow {
    ///
    /// Возвращает ссылку на `Point`
    pub fn value(&self) -> &Point {
        match self {
            FnFlow::New(p) => p,
            FnFlow::Old(p) => p,
        }
    }
    ///
    /// Возвращает `Point`
    pub fn into_value(self) -> Point {
        match self {
            FnFlow::New(p) => p,
            FnFlow::Old(p) => p,
        }
    }
    ///
    /// Возвращает `true` если `FnFlow::New`
    pub fn is_new(&self) -> bool {
        match self {
            FnFlow::New(_) => true,
            FnFlow::Old(_) => false,
        }
    }
}
///
///
/// ### Контекст вычисления узла.
/// Аккумулирует признак активности данных (is_new) при обходе входов,
/// позволяя финальному узлу корректно транслировать статус обновления.
pub struct FlowContext {
    is_new: bool,
}
//
impl FlowContext {
    ///
    /// Returns `FlowContext` new instance
    pub fn new() -> Self {
        Self {
            is_new: false,
        }
    }
    ///
    /// Пропускает через себя управляющие сигналы `FnResult<FnFlow>`, ни как не помечая контекст
    /// - Извлекает чистый `Point` для дальнейшей бизнес-логики.
    /// - Используется для входов вроде `reset` или `limits`.
    pub fn ignore(&self, v: FnResult<FnFlow, String>) -> FnResult<Point, String> {
        let Some(flow) = v? else { return Ok(None) };
        Ok(Some(flow.into_value()))
    }
    ///
    /// ### Пропускает через себя `FnResult<FnFlow>`
    /// - Фиксирует `FnFlow::New`
    /// - Извлекает чистый `Point` для дальнейшей бизнес-логики.
    pub fn map(&mut self, v: FnResult<FnFlow, String>) -> FnResult<Point, String> {
        let Some(flow) = v? else { return Ok(None) };
        self.is_new |= flow.is_new();
        Ok(Some(flow.into_value()))
    }
    ///
    /// ### Оборачивает итоговый `Point` обратно в `FnFlow`,
    /// - Учитывая историю опроса всех входов в текущем контексте.
    /// - Возвращает `Ok(Some(FnFlow(p)))`
    pub fn wrap(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(match self.is_new {
            true => FnFlow::New(p),
            false => FnFlow::Old(p)
        }))
    }
    ///
    /// ### Принудительно оборачивает итоговый `Point` в `FnFlow::New`,
    /// - Учитывая историю опроса всех входов в текущем контексте.
    /// - Возвращает `Ok(Some(FnFlow(p)))`
    pub fn wrap_new(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(FnFlow::New(p)))
    }
    ///
    /// ### Принудительно оборачивает итоговый `Point` в `FnFlow::Old`,
    /// - Возвращает `Ok(Some(FnFlow(p)))`
    pub fn wrap_old(&self, p: Point) -> FnResult<FnFlow, String> {
        Ok(Some(FnFlow::Old(p)))
    }
    ///
    /// ### Возвращает `true` если `FnFlow::New` зарегистрирован
    /// - Это означае, что один из входов вернул новое значение
    pub fn is_new(&self) -> bool {
        self.is_new
    }
    ///
    /// ### Возвращает `true` если `FnFlow::New` не было
    /// - Все входы вернули устаревшее значение
    pub fn is_old(&self) -> bool {
        !self.is_new
    }
}
//
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
///
/// Input side interface for nested function
/// Used for generic access to the different kinde of functions
/// for adding new value on input side
pub trait FnIn: std::fmt::Debug {
    ///
    /// Adds new value into Input
    fn add(&mut self, point: &Point);
    ///
    /// Returns 'Options hash' to identify unique set of options of the Input
    fn hash(&self) -> String;
}
///
/// Out side interface for the function
/// Used for generic access to the different kinde of functions
/// - to get the calculated value on out side
/// - to reset the state to the initial
pub trait FnOut: std::fmt::Debug {
    ///
    /// Retirns it unique idetificator
    fn id(&self) -> String;
    ///
    /// Returns enum kind of the FnOut
    fn kind(&self) -> FnKind;
    ///
    /// Returns names of inputs it depending on
    fn inputs(&self) -> Vec<String>;
    ///
    /// - Evaluate calculations
    /// - Returns calculated value
    /// - Returns error if:
    ///   - Calculations fails
    ///   - Input not initialized
    /// - Returns None if:
    ///   - Point filtered by any kind of filtering function
    fn out(&mut self) -> FnResult<FnFlow, String>;
    ///
    /// resets self state to the initial, calls reset method of all inputs
    fn reset(&mut self);
}
///
/// Interface for nested function
/// Used for generic access to the different kinde of functions in the nested tree
pub trait FnInOut: FnIn + FnOut {}}
mod fn_input {
use sal_sync::services::{entity::{Point, PointHlr, Status, ToPoint}, task::functions::{FnConfPointType, FnConfig}, types::Bool};
use std::{fmt::Debug, sync::atomic::{AtomicUsize, Ordering}};
use crate::services::task::{CycleIndex, EvalCycleRef, FnFlow, FnInOut};
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
    eval_cycle: EvalCycleRef,
    /// Локальное значение вычислительного цикла в котором было оновление
    cycle: CycleIndex,
}
//
//
impl FnInput {
    // pub fn new(parent: &str, name: impl Into<String>, initial: Option<PointType>, type_: FnConfPointType) -> Self {
    pub fn new(parent: impl Into<String>, txid: usize, conf: &FnConfig, cycle: &EvalCycleRef) -> Self {
        let dbg = format!("{}/FnInput{}", parent.into(), COUNT.fetch_add(1, Ordering::AcqRel));
        let (typ, initial) = match conf.type_.clone() {
            FnConfPointType::Bool => (PointType_::Bool, conf.options.default.as_ref().map_or(None, |d| match d.parse::<bool>() {
                Ok(d) => Some(d.to_point(txid, &conf.name)),
                Err(_) => panic!("{}.function | Error parsing Point default as Bool in: {:?}", dbg, conf),
            })),
            FnConfPointType::Int => (PointType_::Int, conf.options.default.as_ref().map_or(None, |d| match d.parse::<i64>() {
                Ok(d) => Some(d.to_point(txid, &conf.name)),
                Err(_) => panic!("{}.function | Error parsing Point default as Int in: {:?}", dbg, conf),
            })),
            FnConfPointType::Real => (PointType_::Real, conf.options.default.as_ref().map_or(None, |d| match d.parse::<f32>() {
                Ok(d) => Some(d.to_point(txid, &conf.name)),
                Err(_) => panic!("{}.function | Error parsing Point default as Real in: {:?}", dbg, conf),
            })),
            FnConfPointType::Double => (PointType_::Double, conf.options.default.as_ref().map_or(None, |d| match d.parse::<f64>() {
                Ok(d) => Some(d.to_point(txid, &conf.name)),
                Err(_) => panic!("{}.function | Error parsing Point default as Double in: {:?}", dbg, conf),
            })),
            FnConfPointType::String => (PointType_::String, conf.options.default.as_ref().map(|d| d.to_point(txid, &conf.name))),
            FnConfPointType::Any => (PointType_::Any, Some(false.to_point(txid, &conf.name))),
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
            eval_cycle: cycle.clone(),
            cycle: CycleIndex::new(),
        }
    }
    ///
    /// ### Returns true if has new value
    fn is_new(&self) -> bool {
        self.cycle == self.eval_cycle.get()
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
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.typ(), self.typ);
                        return;
                    }
                }
            }
            PointType_::Int => {
                match point {
                    Point::Bool(p) => Point::Int(PointHlr::new(p.txid, &p.name, if p.value.0 {1} else {0}, p.status, p.cot, p.timestamp)),
                    Point::Int(p) => Point::Int(PointHlr::new(p.txid, &p.name, p.value, p.status, p.cot, p.timestamp)),
                    Point::Real(_) | Point::Double(_) | Point::String(_) | Point::Bytes(_) => {
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.typ(), self.typ);
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
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.typ(), self.typ);
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
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.typ(), self.typ);
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
                        log::error!("{}.add | Error. Incompatible Type '{:?}', '{:?}' expected", self.dbg, point.typ(), self.typ);
                        return;
                    }
                }
            }
            PointType_::Any => point.clone(),
        };
        self.cycle = self.eval_cycle.get();
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
                Ok(Some(FnFlow::New(point.clone())))
            } else {
                Ok(Some(FnFlow::Old(point.clone())))
            },
            None => Ok(None),   //FnResult::Err(concat_string!(self.dbg, ".out | Not initialized")),
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
}
mod fn_kind {
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnKind {
    Input,
    Var,
    Fn,
}}
mod fn_result {
///
/// Result returning from the Task FnOut
pub type FnResult<T, E> = Result<Option<T>, E>;
}
mod fn_var {
use std::sync::atomic::{Ordering, AtomicUsize};
use crate::{domain::FnOutRef, services::task::FnFlow};
use super::{FnOut, FnKind, FnResult};
///
/// ### Variable | Specific kinde of function
/// - has reference to calculations corresponding to the variable name
#[derive(Debug, Clone)]
pub struct FnVar {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    // value: Option<FnResult<Point, String>>,
}
//
//
impl FnVar {
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnVar{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Var,
            input,
            // value: None,
        }
    }
}
//
//
// impl FnIn for FnVar {}
//
//
impl FnOut for FnVar {
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
    ///
    /// - Evaluate calculations
    /// - Returns calculated value
    /// - Returns error if:
    ///   - Calculations fails
    ///   - Input not initialized
    /// - Returns None if:
    ///   - Point filtered by any kind of filtering function
    fn out(&mut self) -> FnResult<FnFlow, String> {
        log::trace!("{}.eval | evaluating...", self.id);
        let value = self.input.borrow_mut().out();
        log::trace!("{}.out | value: {:?}", self.id, value);
        value
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
//
//
// impl FnInOut for FnVar {}
///
/// Global static counter of FnVar instances
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
mod edge_detection {
//!
//! `Task` Service Functions intend for signal edge dectection, used in the Task service
//!
mod fn_falling_edge {
use concat_string::concat_string;
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, Status}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{Edge, EdgeDetector, FnOutRef, TryTo},
    services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnFallingEdge`
///
/// Детектор отрицательного (заднего) фронта
///
/// - `input`: Последовательность `true -> false` - активирует выход на один такт
#[derive(Debug)]
pub struct FnFallingEdge {
    id: String,
    kind: FnKind,
    input: FnChange,
    edge: EdgeDetector,
    prev: Option<(bool, Status)>,
}
//
impl FnFallingEdge {
    ///
    /// Returns `FnFallingEdge` new instance
    /// - `input`: Последовательность `true -> false` - активирует выход на один такт
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnFallingEdge{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input: FnChange::new(input),
            edge: EdgeDetector::new(),
            prev: None,
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
}
//
//
impl FnOut for FnFallingEdge {
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
        self.input.inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let input = self.input.out();
        let flow = FlowContext::new();
        let Some(input) = flow.ignore(input)? else {
            self.edge.reset();
            self.prev = None;
            return Ok(None);
        };
        let val: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let value = match self.edge.add(val) {
            Some(Edge::Falling) => true,
            _ => false,
        };
        let status = input.status();
        let is_changed = self.prev.map_or(
            true,
            |(prev_value, prev_status)| prev_value != value || prev_status != status
        );
        self.prev = Some((value, status));
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value)));
        // log::trace!("{}.out | value: {:#?}", self.id, point);
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.edge.reset();
        self.prev = None;
        self.input.reset();
    }
}
///
/// Global static counter of FnFallingEdge instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_rising_edge {
use concat_string::concat_string;
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, Status}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{Edge, EdgeDetector, FnOutRef, TryTo},
    services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnRisingEdge`
///
/// Детектор положительного (переднего) фронта
///
/// - `input`: Последовательность `false -> true` - активирует выход на один такт
#[derive(Debug)]
pub struct FnRisingEdge {
    id: String,
    kind: FnKind,
    input: FnChange,
    edge: EdgeDetector,
    prev: Option<(bool, Status)>,
}
//
impl FnRisingEdge {
    ///
    /// Returns `FnRisingEdge` new instance
    /// - `input`: Последовательность `false -> true` - активирует выход на один такт
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnRisingEdge{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input: FnChange::new(input),
            edge: EdgeDetector::new(),
            prev: None,
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
}
//
impl FnOut for FnRisingEdge {
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
        self.input.inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let input = self.input.out();
        let flow = FlowContext::new();
        let Some(input) = flow.ignore(input)? else {
            self.edge.reset();
            self.prev = None;
            return Ok(None);
        };
        let status = input.status();
        let val: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let value = match self.edge.add(val) {
            Some(Edge::Rising) => true,
            _ => false,
        };
        let is_changed = self.prev.map_or(
            true,
            |(prev_value, prev_status)| prev_value != value || prev_status != status
        );
        self.prev = Some((value, status));
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value)));
        // log::trace!("{}.out | value: {:#?}", self.id, point);
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.edge.reset();
        self.prev = None;
        self.input.reset();
    }
}
///
/// Global static counter of FnRisingEdge instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Test
}
pub use fn_falling_edge::*;
pub use fn_rising_edge::*;}
mod export {
//!
//! `Task` Service Functions intend for exporting Point's to another services or send back to the Task
//!
mod fn_to_api_queue {
use sal_sync::{services::entity::{Point, PointHlr}, sync::channel::Sender};
use std::sync::{atomic::{AtomicUsize, Ordering}};
use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}};
///
/// ### Function | `FnToApiQueue`
///
/// Экспортирует данные (SQL-запросы) из вычислительного графа в очередь API.
/// - Отправляет данные только при наличии статуса `New` в потоке.
/// - Автоматически экранирует кавычки и удаляет мусорные символы перед отправкой.
/// - Игнорирует пустые строки (не отправляет их в базу данных).
///
/// **Example**
/// ```yaml
/// fn ToApiQueue:
///     queue: /App/ApiClient.in-queue
///     input fn Sql:
///         sql: "insert into public.event (pid,value,status,timestamp) values ({input2.value},{input1.value},{input1.status},'{input1.timestamp}');"
///         input1 fn ToInt:
///             input: point any every      # point: every point of any type
///         input2 fn PointId:
///             input: point any every
/// ```
#[derive(Debug)]
pub struct FnToApiQueue {
    txid: usize,
    id: String,
    kind: FnKind,
    input: FnOutRef,
    tx: Sender<Point>,
}
//
impl FnToApiQueue {
    ///
    /// Returns `FnToApiQueue` new instance
    /// - `parent`: Имя родительского узла.
    /// - `txid`: Идентификатор сервиса-отправителя (родительский `Task`).
    /// - `input`: Входящий поток данных (SQL-строки).
    /// - `send`: Канал связи с целевым сервисом (например, `ApiClient`).
    pub fn new(parent: impl Into<String>, txid: usize, input: FnOutRef, send: Sender<Point>) -> Self {
        Self {
            txid,
            kind: FnKind::Fn,
            input,
            tx: send,
            id: format!("{}/FnToApiQueue{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
        }
    }
    ///
    /// Возвращает `Point` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with(&self, p: &Point, value: String) -> Point {
        Point::String(match p {
            Point::String(hlr) => hlr.clone().with_value(value).with_txid(self.txid),
            _ => PointHlr::new(self.txid, p.name(), value, p.status(), p.cot(), p.ts()),
        })
    }
    ///
    /// Выполняет отправку точки в канал целевого сервиса
    fn send(&self, point: Point) {
        if log::max_level() >= log::LevelFilter::Trace {
            match self.tx.send(point.clone()) {
                Ok(_) => log::trace!("{}.out | Point sent: {:#?}", self.id, point),
                Err(err) => log::error!("{}.out | Send error: {:?}", self.id, err),
            };
        } else {
            if let Err(err) = self.tx.send(point) {
                log::error!("{}.out | Send error: {:#?}", self.id, err);
            }
        }
    }
}
//
impl FnOut for FnToApiQueue {
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
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:?}", self.id, input);
        if flow.is_new() {
            let sql = prepare_for_sql(&(&input).to_string().as_string().value);
            if !sql.is_empty() {
                self.send(self.point_with(&input, sql));
            }
        }
        flow.wrap(input)
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToApiQueue instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Подготавливает сырую строку для безопасной вставки в SQL-запрос.
/// - Удаляет пробелы по краям
/// - Вырезает нулевые байты (\0)
/// - Экранирует одинарные кавычки
pub fn prepare_for_sql(input: &str) -> String {
    let trimmed = input.trim();
    // +8 байт — запас под несколько кавычек
    let mut result = String::with_capacity(trimmed.len() + 8);
    for c in trimmed.chars() {
        match c {
            '\0' => continue,
            '\'' => result.push_str("''"),
            _ => result.push(c),
        }
    }
    result
}
///
/// Basic Tests
}
mod fn_export {
use function_name::named;
use sal_core::error::Error;
use sal_sync::{services::{entity::{Point, PointConf, PointType, PointHlr}, types::Bool}, sync::channel::Sender};
use std::sync::{atomic::{AtomicUsize, Ordering}};
use crate::{
    domain::{FnOutRef, TryTo}, err, err_pass, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | FnExport
///
/// Узел экспорта сигнала `Point` другому сервису
///
/// - Сигнал отправляется в очередь только при выполнении условий:
///     - Входящий поток имеет статус `New` (изменение значения).
///     - Разрешающий сигнал `enable` активен (через `FnEnable`).
///     - Указан маршрут `send-to`.
/// - Если указан `conf`, то сигнал будет конвертирован в указанный тип и отправлен с указанным именем
///
/// - Функция абсолютно прозрачна, не разрывает граф, всегда возвращает оригинал со входа.
///
/// **Example**
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
    txid: usize,
    kind: FnKind,
    conf: Option<PointConf>,
    input: FnOutRef,
    tx: Option<Sender<Point>>,
    id: String,
}
//
impl FnExport {
    ///
    /// Creates new instance of FnExport
    /// - `parent` - Name of the parent node
    /// - `txid`: Идентификатор сервиса отправителя (родительский `Task` в данном случае)
    /// - `conf` - Configuration of the Point to be produced. If None, input Point is sent
    /// - `input`: Incoming input events
    /// - `send`: Link of the destination service
    pub fn new(parent: impl Into<String>, txid: usize, conf: Option<PointConf>, input: FnOutRef, send: Option<Sender<Point>>) -> Self {
        let id = format!("{}/FnExport{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            txid,
            kind: FnKind::Fn,
            conf,
            input,
            tx: send,
            id,
        }
    }
    ///
    /// Конвертирует оригинальный сигнал в целевой тип с обновлением txid и имени
    #[named]
    fn convert_to(&self, name: &str, p: &Point, target: &PointType) -> Result<Point, Error> {
        match target {
            PointType::Bool => Ok(Point::Bool(PointHlr::new(self.txid, name, Bool(p.try_to()?), p.status(), p.cot(), p.ts()))),
            PointType::Int => Ok(Point::Int(PointHlr::new(self.txid, name, p.try_to()?, p.status(), p.cot(), p.ts()))),
            PointType::Real => Ok(Point::Real(PointHlr::new(self.txid, name, p.try_to()?, p.status(), p.cot(), p.ts()))),
            PointType::Double => Ok(Point::Double(PointHlr::new(self.txid, name, p.try_to()?, p.status(), p.cot(), p.ts()))),
            PointType::String => Ok(p.to_string().with_txid(self.txid).with_name(name)),
            PointType::Bytes => match p {
                Point::Bytes(hlr) => Ok(Point::Bytes(hlr.clone()).with_txid(self.txid).with_name(name)),
                _ => Err(err!(self.id, "Invalid input type {:?} for converting into 'Bytes'", p.typ())),
            },
            PointType::Json => match p {
                Point::Bytes(hlr) => Ok(Point::Bytes(hlr.clone()).with_txid(self.txid).with_name(name)),
                _ => Err(err!(self.id, "Invalid input type {:?} for converting into 'Json'", p.typ())),
            },
        }
    }
    ///
    /// Выполняет отправку точки в канал целевого сервиса
    fn send(&self, point: Point) {
        if let Some(tx) = &self.tx {
            if log::max_level() >= log::LevelFilter::Trace {
                match tx.send(point.clone()) {
                    Ok(_) => log::trace!("{}.out | Point sent: {:#?}", self.id, point),
                    Err(err) => log::error!("{}.out | Send error: {:?}", self.id, err),
                };
            } else {
                if let Err(err) = tx.send(point) {
                    log::error!("{}.out | Send error: {:#?}", self.id, err);
                }
            }
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
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::debug!("{}.out | input: {:?}", self.id, input);
        if flow.is_new() {
            let point = match &self.conf {
                Some(conf) => self.convert_to(&conf.name, &input, &conf.type_)
                    .map_err(|err| err_pass!(self.id, err).to_string())?,
                None => input.clone().with_txid(self.txid),
            };
            self.send(point);
        }
        flow.wrap(input)
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnExport instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_point {
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
///
/// Global static counter of FnPoint instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
pub use fn_to_api_queue::*;
pub use fn_export::*;
pub use fn_point::*;
}
mod filter {
mod fn_select {
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use sal_sync::services::entity::PointType;
use crate::{
    domain::FnOutRef,
    services::task::{FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `Select`
///
/// Классический мультиплексор (чистая функция).
/// - Возвращает значение `input`, если `select = true` или `> 0`.
/// - Возвращает значение `default`, если `select = false` или `<= 0`.
/// - Если `default` не задан, а `select = false`, возвращает `Ok(None)` (обрыв потока).
#[derive(Debug)]
pub struct FnSelect {
    id: String,
    kind: FnKind,
    default: Option<FnOutRef>,
    input: FnOutRef,
    select: FnOutRef,
}
//
//
impl FnSelect {
    ///
    /// ### Creates new instance of the FnSelect
    /// * `parent` - Идентификатор родительского узла.
    /// * `default` - Входной сигнал (select = 0). Опционален.
    /// * `input` - Входной сигнал (select = 1).
    /// * `select` - Управляющий логический или числовой сигнал.
    pub fn new(parent: impl Into<String>, default: Option<FnOutRef>, input: FnOutRef, select: FnOutRef) -> Self {
        let self_id = format!("{}/FnSelect{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            id: self_id,
            kind: FnKind::Fn,
            default,
            input,
            select,
        }
    }
}
//
//
impl FnOut for FnSelect {
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
        inputs.append(&mut self.select.borrow().inputs());
        inputs.append(&mut self.input.borrow().inputs());
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let input = self.input.borrow_mut().out();
        let default = self.default.as_mut().map(|f| f.borrow_mut().out());
        let select = self.select.borrow_mut().out();
        let Some(select) = select? else { return Ok(None) };
        let select = select.into_value();
        log::trace!("{}.out | select: {:?}", self.id, select);
        let is_selected = match select.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => select.to_bool().as_bool().value.0,
            _ => return Err(concat_string!(self.id, ".out | Invalid select type '", select.typ().to_string(), "'")),
        };
        if is_selected {
            let Some(input) = input? else { return Ok(None) };
            log::trace!("{}.out | input value: {:?}", self.id, input);
            Ok(Some(input))
        } else {
            if let Some(default) = default {
                let Some(default) = default? else { return Ok(None) };
                log::trace!("{}.out | default value: {:?}", self.id, default);
                Ok(Some(default))
            } else {
                Ok(None)
            }
        }
    }
    //
    fn reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
        self.select.borrow_mut().reset();
    }
}
///
/// Global static counter of FnSelect instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_threshold {
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use sal_sync::services::{entity::{Point, PointHlr, PointType}, types::Bool};
use crate::{
    domain::{FnOutRef, filter::{filter::Filter, filter_threshold::FilterThreshold}}, services::task::{
        FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult
    }
};
///
/// ### Function | `FnThreshold`
///
/// Фильтр сигнала по порогу (абсолютному или интегральному).
///
/// Особенности работы:
/// - **enable**: (Через декоратор FnEnable) Управление жизненным циклом.
///   Режим Cold полностью сбросит накопленную дельту и вернет None.
/// - Если `factor` не задан: работает как детектор абсолютного скачка.
///   Пропускает значение только если `|текущее - предыдущее| >= threshold`.
/// - Если `factor` задан: работает как интегратор.
///   Каждый такт накапливает дельту: `sum += |текущее - предыдущее| * factor`.
///   Пропускает значение, когда сумма достигает порога.
///
/// **Example**
///
/// ```yaml
/// fn Threshold:
///     enable: const bool true     # optional, default true
///     threshold: const real 0.5   # absolute threshold if [factor] is not specified
///     factor: const real 0.1      # optional, use for integral threshold
///     input: point real '/App/Service/Point.Name'
/// ```
#[derive(Debug)]
pub struct FnThreshold {
    id: String,
    kind: FnKind,
    threshold: FnChange,
    factor: Option<FnChange>,
    input: FnChange,
    filter: FilterThreshold<f64>,
}
//
impl FnThreshold {
    ///
    /// ### Creates `FnThreshold` new instance
    /// * `parent` - Идентификатор родительского узла.
    /// * `threshold` - Узел, задающий порог срабатывания.
    /// * `factor` - Узел весового коэффициента (опционально, для интегрального режима).
    /// * `input` - Входной сигнал для фильтрации.
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, threshold: FnOutRef, factor: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnThreshold{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            threshold: FnChange::new(threshold),
            factor: factor.map(FnChange::new),
            input: FnChange::new(input),
            filter: FilterThreshold::new(None, 0.0, 0.0),
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
    ///
    /// Возвращает `Point` с обновленными `name` и `value` сохраняя тип
    #[inline]
    fn point(id: &str, input: &Point, val: f64) -> Result<Point, String> {
        match input.typ() {
            PointType::Bool => Ok(Point::Bool(Self::point_with(input, id, Bool(val != 0.0)))),
            PointType::Int => Ok(Point::Int(Self::point_with(input, id, val.round() as i64))),
            PointType::Real => Ok(Point::Real(Self::point_with(input, id, val as f32))),
            PointType::Double => Ok(Point::Double(Self::point_with(input, id, val))),
            _ => Err(concat_string!(id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        }
    }
}
//
impl FnOut for FnThreshold {
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
        let mut inputs = self.input.inputs();
        inputs.append(&mut self.threshold.inputs());
        if let Some(factor) = &self.factor {
            inputs.append(&mut factor.inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let input = self.input.out();
        let threshold = self.threshold.out();
        let factor = self.factor.as_mut().map(|f| f.out());
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(input)? else { return Ok(None) };
        let Some(threshold) = flow.ignore(threshold)? else { return Ok(None) };
        log::trace!("{}.out | threshold: {:?}", self.id, threshold);
        let threshold = match threshold.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => threshold.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid threshold type '", threshold.typ().to_string(), "'")),
        };
        self.filter = self.filter.with_threshold(threshold);
        if let Some(factor) = factor {
            let Some(factor) = flow.ignore(factor)? else { return Ok(None) };
            log::trace!("{}.out | factor: {:?}", self.id, factor);
            let factor = match factor.typ() {
                PointType::Bool | PointType::Int | PointType::Real | PointType::Double => factor.to_double().as_double().value,
                _ => return Err(concat_string!(self.id, ".out | Invalid factor type '", factor.typ().to_string(), "'")),
            };
            self.filter = self.filter.with_factor(factor);
        }
        if flow.is_old() {
            let value = match self.filter.last() {
                Some(val) => val,
                None => match input.typ() {
                    PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
                    _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
                },
            };
            let value = Self::point(&self.id, &input, value)?;
            return flow.wrap_old(value);
        }
        let value = match input.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        log::trace!("{}.out | input: {:?}", self.id, value);
        match self.filter.add(value) {
            Some(new_val) => {
                let point = Self::point(&self.id, &input, new_val)?;
                log::trace!("{}.out | Threshold new value: {:?}", self.id, point);
                flow.wrap(point)
            },
            None => {
                let point = Self::point(&self.id, &input, self.filter.last().unwrap_or(value))?;
                log::trace!("{}.out | Threshold old value: {:?}", self.id, point);
                flow.wrap_old(point)
            }
        }
    }
    //
    fn reset(&mut self) {
        self.threshold.reset();
        if let Some(factor) = &mut self.factor {
            factor.reset();
        }
        self.input.reset();
        self.filter.reset();
    }
}
///
/// Global static counter of FnThreshold instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_smooth {
use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointType};
use crate::{
    domain::FnOutRef, services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    }
};
///
/// Function | EMA (Exponential Moving Average)
/// - Returns smoothed input:
/// - out = out + (input - prev) * factor
#[derive(Debug)]
pub struct FnSmooth {
    kind: FnKind,
    factor: FnOutRef,
    input: FnOutRef,
    value: Point,
    id: String,
}
//
//
impl FnSmooth {
    ///
    /// Creates new instance of the FnSmooth
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, factor: FnOutRef, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnSmooth{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            kind: FnKind::Fn,
            factor,
            input,
            value: Point::new(0, "", 0.0),
            id,
        })
    }
}
//
//
impl FnOut for FnSmooth {
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
        inputs.append(&mut self.factor.borrow().inputs());
        inputs.append(&mut self.input.borrow().inputs());
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        unimplemented!();
        // let mut flow = FlowContext::new();
        // let factor = self.factor.borrow_mut().out();
        // log::trace!("{}.out | factor: {:?}", self.id, factor);
        // let factor = match factor {
        //     FnResult::Ok(factor) => factor.to_double().as_double(),
        //     FnResult::None => return FnResult::None,
        //     FnResult::Err(err) => return FnResult::Err(err),
        // };
        // let input = self.input.borrow_mut().out();
        // log::trace!("{}.out | input: {:?}", self.id, input);
        // match input {
        //     FnResult::Ok(input) => {
        //         let input_type = input.typ();
        //         log::trace!("{}.out | factor: {:?}", self.id, factor);
        //         let delta = input.to_double().as_double() - self.value.to_double().as_double();
        //         log::trace!("{}.out | delta: {:?}", self.id, delta);
        //         let value = self.value.to_double().as_double() + delta * factor;
        //         log::trace!("{}.out | value: {:?}", self.id, value);
        //         let value = Point::Double(value);
        //         self.value = match input_type {
        //             PointType::Int => value.to_int(),
        //             PointType::Real => value.to_real(),
        //             PointType::Double => value.to_double(),
        //             _ => panic!("{}.out | Illegal type of input {:?}", self.id, input_type),
        //         };
        //         log::trace!("{}.out | value: {:?}", self.id, self.value);
        //         FnResult::Ok(self.value.clone())
        //     }
        //     FnResult::None => FnResult::None,
        //     FnResult::Err(err) => FnResult::Err(err),
        // }
    }
    //
    //
    fn reset(&mut self) {
        self.factor.borrow_mut().reset();
        self.input.borrow_mut().reset();
        self.value = Point::new(0, "", 0.0);
    }
}
///
/// Global static counter of FnSmooth instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
}
pub use fn_select::*;
pub use fn_threshold::*;
pub use fn_smooth::*;
}
mod import {
}
mod io {
mod fn_retain {
use function_name::named;
use sal_core::error::Error;
use sal_sync::services::{Services, entity::Name , task::functions::FnConfig};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use crate::{domain::{FnOutRef}, err, err_pass, services::task::{FnBuilder, FnEnable, TaskNodes, functions::{FnRetainRead, FnRetainWrite}}};
///
/// ### Builder | `FnRetain`
///
/// Билдер для создания узлов `FnRetainRead` или `FnRetainWrite`.
///
/// - **`FnRetainRead`** (Если `input` отсутствует)
///     - Чтение значений с диска (по умолчанию читает с диска только в первый цикл, дальше возвращает закэшированное значение).
///     - `default` на случай, когда значений еще не записано.
///     - `every-cycle` - значение будет читаться на каждом цикле вычислений (учитывай нагрузку на диск)
/// - **`FnRetainWrite`** (Если `input` задан)
///     - Запись на диск при изменении значения, статуса или метки времени.
///     - Для графа прозрачна, пропускает `FnFlow` сквозь себя без модификаций.
#[derive(Debug)]
pub struct FnRetain {}
//
impl FnRetain {
    ///
    /// ### Returns `FnRetainRead`  or `FnRetainWrite` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `conf`: Конфигурация узла
    #[named]
    pub fn new(parent: &Name, conf: &FnConfig, nodes: &mut TaskNodes, services: &Arc<Services>) -> Result<FnOutRef, Error> {
        let self_id = format!("{parent}/FnRetain");
        let enable = FnBuilder::get_input_config(parent, "enable", conf, nodes, services)
            .map_err(|err| err_pass!(self_id, err, "Can't get 'enable'"))?;
        let default = FnBuilder::get_input_config(parent, "default", conf, nodes, services)
            .map_err(|err| err_pass!(self_id, err, "Can't get 'default'"))?;
        let input = FnBuilder::get_input_config(parent, "input", conf, nodes, services)
            .map_err(|err| err_pass!(self_id, err, "Can't get 'input'"))?;
        let every_cycle = conf.param("every-cycle").map_or(Ok(false), |param| {
            param.as_param().conf.as_bool().ok_or_else(|| err!(self_id, "'every-cycle' - wrong config"))
        })?;
        let Some(key) = conf.param("key").map(|v| v.as_param()) else {
            return Err(err!(self_id, "Parameter 'key' - missed in '{}'", conf.name));
        };
        let key = key.conf.as_str()
            .ok_or_else(|| err!(self_id, "Parameter 'key' must be a string in '{}'", conf.name))?;
        Ok(match input {
            None => {
                let read = FnRetainRead::new(parent, nodes.retain(), key, every_cycle, default).map_err(|err| err_pass!(self_id, err))?;
                match enable {
                    Some(en) => Rc::new(RefCell::new(FnEnable::new(read, nodes.enable_mode(), en))),
                    None => Rc::new(RefCell::new(read)),
                }
            }
            Some(input) => {
                let write = FnRetainWrite::new(parent, nodes.retain().link(), key, default, input).map_err(|err| err_pass!(self_id, err))?;
                match enable {
                    Some(en) => Rc::new(RefCell::new(FnEnable::new(write, nodes.enable_mode(), en))),
                    None => Rc::new(RefCell::new(write)),
                }
            }
        })
    }
}
}
pub use fn_retain::*;
mod fn_retain_read {
use sal_core::error::Error;
use sal_sync::services::entity::{Name, Point, PointTxId};
use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}};
use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult, TaskRetain}};
///
/// ### Function | `FnRetainRead`
///
/// Чтение значения `Point` из журнала на диске.
/// По умолчанию читает данные только в первый цикл вычислений,
/// далее возвращает закэшированное значение.
///
/// **Особенности работы:**
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `default`: Запасной источник данных, если файл на диске отсутствует.
/// - `every-cycle`: При `true` читает актуальные данные из журнала на каждом такте вычислений.
///
/// **Пример:**
/// ```yaml
/// fn Retain:
///     key: 'OperatingCycleId'
///     input fn Acc:
///         initial fn Retain:      # Тут происходит чтение
///             default: const int 0
///             key: 'OperatingCycleId'
///         input: opCycleIsDone
/// ```
#[derive(Debug)]
pub struct FnRetainRead {
    txid: usize,
    kind: FnKind,
    key: String,
    retain: Arc<TaskRetain>,
    every_cycle: bool,
    default: Option<FnOutRef>,
    cache: Option<Point>,
    id: String,
}
//
impl FnRetainRead {
    ///
    /// ### Returns `FnRetainRead` new instance
    /// - `parent`: Идентификатор родительского узла.
    /// - `retain`: `TaskRetain`, разделяемый контекст доступа к дисковому журналу.
    /// - `key`: Уникальный ключ переменной в журнале.
    /// - `every_cycle`: Если true, чтение будет выполняться непрерывно, иначе — единоразово при старте.
    /// - `default`: Резервный узел. Будет использован, если ключа в журнале нет.
    pub fn new(parent: &Name, retain: Arc<TaskRetain>, key: impl Into<String>, every_cycle: bool, default: Option<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnRetainRead{}", parent.join(), COUNT.fetch_add(1, Ordering::Relaxed));
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            key: key.into(),
            retain,
            every_cycle,
            default,
            cache: None,
            id,
        })
    }
}
//
impl FnOut for FnRetainRead {
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
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        if !self.every_cycle {
            if let Some(val) = self.cache.as_ref() {
                return flow.wrap_old(val.clone());
            }
        }
        if let Some(val) = self.retain.get(&self.key) {
            if self.every_cycle {
                if let Some(cached) = &self.cache {
                    if cached.value() == val.value() && cached.status() == val.status() {
                        return flow.wrap_old(val);
                    }
                }
            }
            self.cache = Some(val.clone());
            return flow.wrap_new(val);
        } else {
            if let Some(default) = self.default.as_ref() {
                if self.every_cycle {
                    let Some(val) = flow.map(default.borrow_mut().out())? else { return Ok(None) };
                    return flow.wrap(val);
                } else {
                    let Some(val) = flow.ignore(default.borrow_mut().out())? else { return Ok(None) };
                    self.cache = Some(val.clone());
                    return flow.wrap_new(val);
                }
            }
            Ok(None)
        }
    }
    //
    fn reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().reset();
        }
        self.cache = None;
    }
}
///
/// Global static counter of FnRetainRead instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
pub(super) use fn_retain_read::*;
mod fn_retain_write {
use sal_core::error::Error;
use sal_sync::services::entity::{Name, Point};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::{FnOutRef, Sender}, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult, RetainEvent}};
///
/// ### Function | `FnRetainWrite`
///
/// Запись значения `Point` на диск через фоновый процесс `TaskRetain`.
/// Пишет строго по изменению значения, статуса или метки времени.
/// Для вычислений прозрачна, не вносит изменений в поток.
///
/// **Особенности работы:**
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `default`: Запасной источник данных, если вход молчит (`Ok(None)`),
///    то узел попытается вернуть и записать на диск `default`.
///
/// **Пример:**
/// ```yaml
/// fn Retain:            # Тут происходит запись
///     key: 'OperatingCycleId'
///     input fn Acc:
///         initial fn Retain:
///             default: const int 0
///             key: 'OperatingCycleId'
///         input: opCycleIsDone
/// ```
#[derive(Debug)]
pub struct FnRetainWrite {
    kind: FnKind,
    key: String,
    default: Option<FnOutRef>,
    input: FnOutRef,
    cache: Option<Point>,
    send: Sender<RetainEvent>,
    id: String,
}
//
impl FnRetainWrite {
    ///
    /// ### Returns `FnRetainWrite` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `send`: Канал для отправки событий записи в TaskRetain
    /// - `key`: Уникальный ключ переменной для сохранения
    /// - `default`: Запасной источник данных
    /// - `input`: Входной сигнал
    pub fn new(parent: &Name, send: Sender<RetainEvent>, key: impl Into<String>, default: Option<FnOutRef>, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnRetainWrite{}", parent.join(), COUNT.fetch_add(1, Ordering::Relaxed));
        Ok(Self {
            kind: FnKind::Fn,
            key: key.into(),
            default,
            input,
            cache: None,
            send,
            id,
        })
    }
}
//
impl FnOut for FnRetainWrite {
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
        let mut inputs = self.input.borrow().inputs();
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let input = self.input.borrow_mut().out();
        let default = self.default.as_ref().map(|d| d.borrow_mut().out());
        let mut flow = FlowContext::new();
        let point = if let Some(input) = flow.map(input)? {
            Some(input)
        } else if let Some(default) = default {
            flow.ignore(default)?
        } else {
            None
        };
        let Some(point) = point else {
            return Ok(None);
        };
        let is_changed = match self.cache.as_ref() {
            Some(cache) => cache.value() != point.value() || cache.status() != point.status() || cache.ts() != point.ts(),
            None => true,
        };
        if is_changed {
            self.cache = Some(point.clone());
            if let Err(err) = self.send.send(RetainEvent::new(self.key.clone(), point.clone())) {
                log::error!("{}.out | Can't store value '{}': {:?}", self.id, point.name(), err);
            }
        }
        flow.wrap(point)
    }
    //
    fn reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
        self.cache = None;
    }
}
///
/// Global static counter of FnRetainWrite instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Baisic Tests
}
pub(super) use fn_retain_write::*;}
mod ops {
mod fn_bit_and {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnBitAnd`
///
/// Побитовое логическое умножение всех входящих сигналов.
///
/// **Example**
///
/// ```yaml
/// fn BitAnd:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn BitAnd:
///     in1: point bool '/App/Service/Point.Name1'
///     in2: point bool '/App/Service/Point.Name2'
///     in3: point bool '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnBitAnd {
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
//
impl FnBitAnd {
    ///
    /// Returns `FnBitAnd` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `inputs`: Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnBitAnd{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        if inputs.len() < 1 {
            return Err(Error::new(&id, "new").err("At least one input must be specified"));
        }
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `Point` с обновленными `txid`, `name`, `meta` и `value`
    /// - `txid`: Текущий идентификатор отправителя.
    /// - `meta`: Объединенные метаданные всех задействованных входов.
    /// - `name`: Имя формируемого сигнала.
    /// - `value`: Значение формируемого сигнала.
    #[inline]
    fn point_with(txid: usize, meta: &PointMeta, name: impl Into<String>, value: NumValue) -> Point {
        match value {
            NumValue::Bool(value) => Point::Bool(PointHlr::new(txid, name, Bool(value), meta.status, meta.cot, meta.ts)),
            NumValue::Int(value) => Point::Int(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            NumValue::Real(value) => Point::Real(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            NumValue::Double(value) => Point::Double(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
        }
    }
}
//
impl FnOut for FnBitAnd {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        unimplemented!();
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut flow = FlowContext::new();
        let mut value = NumValue::Bool(true);
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            let val: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
            value = NumValue::Bool(value .and(&val).map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?);
        }
        flow.wrap(Self::point_with(self.txid, &meta, &self.id, value))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnBitAnd instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_bit_or {
use sal_core::error::Error;
use sal_sync::services::{
    entity::{Cot, {Point, PointHlr, PointTxId}, Status},
    types::Bool,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::Utc;
use crate::{
    domain::FnOutRef,
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// Function | `FnBitOr`
///
/// Example
///
/// ```yaml
/// fn BitOr:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn BitOr:
///     input1: point bool '/App/Service/Point.Name1'
///     input2: point bool '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnBitOr {
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
//
//
impl FnBitOr {
    ///
    /// Creates new instance of the FnBitOr
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnBitOr{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            kind:FnKind::Fn,
            inputs,
            id,
        })
    }
}
//
//
impl FnOut for FnBitOr {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        unimplemented!();
        let mut inputs = self.inputs.iter();
        // let mut value: Point;
        // match inputs.next() {
        //     Some(first) => {
        //         value = match first.borrow_mut().out() {
        //             FnResult::Ok(first) => first,
        //             FnResult::None => return FnResult::None,
        //             FnResult::Err(err) => return FnResult::Err(err),
        //         };
        //         while let Some(input) = inputs.next() {
        //             let input = input.borrow_mut().out();
        //             match input {
        //                 FnResult::Ok(input) => {
        //                     log::trace!("{}.out | input '{}': {:?}", self.id, input.name(), input.value());
        //                     value = match &value {
        //                         Point::Bool(val) => {
        //                             let input_val = input.try_as_bool().unwrap_or_else(|_| panic!("{}.out | Incopatable types, expected '{:?}', but input '{}' has type '{:?}'", self.id, value.typ(), input.name(), input.typ()));
        //                             Point::Bool(
        //                                 PointHlr::new(
        //                                     tx_id,
        //                                     &format!("{}.out", self.id),
        //                                     Bool(val.value.0 | input_val.value.0),
        //                                     Status::Ok,
        //                                     Cot::Inf,
        //                                     Utc::now(),
        //                                 )
        //                             )
        //                         }
        //                         Point::Int(val) => {
        //                             let input_val = input.try_as_int().unwrap_or_else(|_| panic!("{}.out | Incopatable types, expected '{:?}', but input '{}' has type '{:?}'", self.id, value.typ(), input.name(), input.typ()));
        //                             Point::Int(
        //                                 PointHlr::new(
        //                                     tx_id,
        //                                     &format!("{}.out", self.id),
        //                                     val.value | input_val.value,
        //                                     Status::Ok,
        //                                     Cot::Inf,
        //                                     Utc::now(),
        //                                 )
        //                             )
        //                         }
        //                         Point::Real(_) => {
        //                             panic!("{}.out | Not implemented for Real", self.id);
        //                         }
        //                         Point::Double(_) => {
        //                             panic!("{}.out | Not implemented for Double", self.id);
        //                         }
        //                         Point::String(_) => {
        //                             panic!("{}.out | Not implemented for String", self.id);
        //                         }
        //                         Point::Bytes(_) => {
        //                             panic!("{}.out | Not implemented for Bytes", self.id);
        //                         }
        //                     };
        //                 }
        //                 FnResult::None => return FnResult::None,
        //                 FnResult::Err(err) => return FnResult::Err(err),
        //             }
        //         }
        //     },
        //     None => panic!("{}.out | At least one input must be specified", self.id),
        // };
        // // trace!("{}.out | value: {:#?}", self.id, value);
        // FnResult::Ok(value)
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnBitOr instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_bit_xor {
use sal_core::error::Error;
use sal_sync::services::{
    entity::{Cot, {Point, PointHlr, PointTxId}, Status},
    types::Bool,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::Utc;
use crate::{
    domain::FnOutRef,
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// Function | Returns bitwise XOR of all inputs
///
/// Example
///
/// ```yaml
/// fn BitXor:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn BitXor:
///     input1: point bool '/App/Service/Point.Name1'
///     input2: point bool '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnBitXor {
    id: String,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
}
//
//
impl FnBitXor {
    ///
    /// Creates new instance of the FnBitXor
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnBitXor{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
        Ok(Self {
            id: format!("{}/FnBitXor{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            inputs,
        })
    }
}
//
//
impl FnOut for FnBitXor {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        unimplemented!();
        // let txid = PointTxId::from_str(&self.id);
        // let mut inputs = self.inputs.iter();
        // let mut value: Point;
        // match inputs.next() {
        //     Some(first) => {
        //         value = match first.borrow_mut().out() {
        //             FnResult::Ok(first) => first,
        //             FnResult::None => return FnResult::None,
        //             FnResult::Err(err) => return FnResult::Err(err),
        //         };
        //         while let Some(input) = inputs.next() {
        //             let input = input.borrow_mut().out();
        //             match input {
        //                 FnResult::Ok(input) => {
        //                     log::debug!("{}.out | input '{}': {:?}", self.id, input.name(), input.value());
        //                     value = match &value {
        //                         Point::Bool(val) => {
        //                             let input_val = input.try_as_bool().unwrap_or_else(|_| panic!("{}.out | Incopatable types, expected '{:?}', but input '{}' has type '{:?}'", self.id, value.typ(), input.name(), input.typ()));
        //                             Point::Bool(
        //                                 PointHlr::new(
        //                                     txid,
        //                                     &format!("{}.out", self.id),
        //                                     Bool(val.value.0 ^ input_val.value.0),
        //                                     Status::Ok,
        //                                     Cot::Inf,
        //                                     Utc::now(),
        //                                 )
        //                             )
        //                         }
        //                         Point::Int(val) => {
        //                             let input_val = input.try_as_int().unwrap_or_else(|_| panic!("{}.out | Incopatable types, expected '{:?}', but input '{}' has type '{:?}'", self.id, value.typ(), input.name(), input.typ()));
        //                             Point::Int(
        //                                 PointHlr::new(
        //                                     txid,
        //                                     &format!("{}.out", self.id),
        //                                     val.value ^ input_val.value,
        //                                     Status::Ok,
        //                                     Cot::Inf,
        //                                     Utc::now(),
        //                                 )
        //                             )
        //                         }
        //                         Point::Real(_) => {
        //                             panic!("{}.out | Not implemented for Real", self.id);
        //                         }
        //                         Point::Double(_) => {
        //                             panic!("{}.out | Not implemented for Double", self.id);
        //                         }
        //                         Point::String(_) => {
        //                             panic!("{}.out | Not implemented for String", self.id);
        //                         }
        //                         Point::Bytes(_) => {
        //                             panic!("{}.out | Not implemented for Bytes", self.id);
        //                         }
        //                     };
        //                 }
        //                 FnResult::None => return FnResult::None,
        //                 FnResult::Err(err) => return FnResult::Err(err),
        //             }
        //         }
        //     },
        //     None => panic!("{}.out | At least one input must be specified", self.id),
        // };
        // // trace!("{}.out | value: {:#?}", self.id, value);
        // FnResult::Ok(value)
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnBitXor instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
}
mod fn_not {
use concat_string::concat_string;
use sal_sync::services::entity::{Point, PointHlr, PointType};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | Not
///
/// Логическое отрицание. Инвертирует входное значение.
/// Если на входе `true` (или число > 0), на выходе `false`.
/// Если на входе `false` (или число 0), на выходе `true`.
///
/// - Возвращает: `Bool`
/// - Наследует `status`, `timestamp` и `txid` от входной точки.
///
/// **Example**
/// ```yaml
/// fn Not:
///     input: point int '/App/Service/Point.Name1'
/// fn Not:
///     input: point bool '/App/Service/Point.Name1'
/// ```
#[derive(Debug)]
pub struct FnNot {
    id: String,
    kind: FnKind,
    input: FnOutRef,
}
//
impl FnNot {
    ///
    /// Creates new instance of `FnNot`
    /// * `parent` - Идентификатор родительского узла графа.
    /// * `input` - Входной сигнал для инверсии.
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnNot{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
        }
    }
}
//
impl FnOut for FnNot {
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
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:#?}", self.id, input);
        let value = match input.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_bool().as_bool().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        flow.wrap(Point::Bool(PointHlr::new(
            input.txid(),
            &self.id,
            !value,
            input.status(),
            input.cot(),
            input.ts(),
        )))
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnNot instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic tests
}
mod fn_or {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnOr`
///
/// Логическое сложение всех входящих сигналов.
///
/// **Example**
///
/// ```yaml
/// fn Or:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Or:
///     in1: point bool '/App/Service/Point.Name1'
///     in2: point bool '/App/Service/Point.Name2'
///     in3: point bool '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnOr {
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
//
impl FnOr {
    ///
    /// Returns `FnOr` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `inputs`: Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnOr{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        if inputs.len() < 1 {
            return Err(Error::new(&id, "new").err("At least one input must be specified"));
        }
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `Point` с обновленными `txid`, `name`, `meta` и `value`
    #[inline]
    fn point_with(txid: usize, meta: &PointMeta, name: impl Into<String>, value: NumValue) -> Point {
        match value {
            NumValue::Bool(value) => Point::Bool(PointHlr::new(txid, name, Bool(value), meta.status, meta.cot, meta.ts)),
            NumValue::Int(value) => Point::Int(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            NumValue::Real(value) => Point::Real(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            NumValue::Double(value) => Point::Double(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
        }
    }
}
//
impl FnOut for FnOr {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut flow = FlowContext::new();
        let mut value = NumValue::Bool(false);
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            let val: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
            value = NumValue::Bool(value.or(&val).map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?);
        }
        flow.wrap(Self::point_with(self.txid, &meta, &self.id, value))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnOr instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_and {
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnAnd`
///
/// Логическое умножение всех входящих сигналов.
///
/// **Example**
///
/// ```yaml
/// fn And:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn And:
///     in1: point bool '/App/Service/Point.Name1'
///     in2: point bool '/App/Service/Point.Name2'
///     in3: point bool '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnAnd {
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
//
impl FnAnd {
    ///
    /// Returns `FnAnd` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `inputs`: Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnAnd{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        if inputs.len() < 1 {
            return Err(Error::new(&id, "new").err("At least one input must be specified"));
        }
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `Point` с обновленными `txid`, `name`, `meta` и `value`
    /// - `txid`: Текущий идентификатор отправителя.
    /// - `meta`: Объединенные метаданные всех задействованных входов.
    /// - `name`: Имя формируемого сигнала.
    /// - `value`: Значение формируемого сигнала.
    #[inline]
    fn point_with(txid: usize, meta: &PointMeta, name: impl Into<String>, value: NumValue) -> Point {
        match value {
            NumValue::Bool(value) => Point::Bool(PointHlr::new(txid, name, Bool(value), meta.status, meta.cot, meta.ts)),
            NumValue::Int(value) => Point::Int(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            NumValue::Real(value) => Point::Real(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            NumValue::Double(value) => Point::Double(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
        }
    }
}
//
impl FnOut for FnAnd {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut flow = FlowContext::new();
        let mut value = NumValue::Bool(true);
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            let val: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
            value = NumValue::Bool(value.and(&val).map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?);
        }
        flow.wrap(Self::point_with(self.txid, &meta, &self.id, value))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnAnd instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_add {
use function_name::named;
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta}, err, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | `FnAdd`
///
/// Возвращает сумму всех входов,
/// динамически повышая тип данных до наиболее точного.
///
/// **Example**
/// ```yaml
/// fn Add:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Add:
///     in1: point real '/App/Service/Point.Name1'
///     in2: point real '/App/Service/Point.Name2'
///     in3: point real '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnAdd {
    txid: usize,
    id: String,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
}
//
//
impl FnAdd {
    ///
    /// Returns `FnAdd` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnAdd{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        if inputs.len() < 1 {
            return Err(Error::new(&id, "new").err("At least one input must be specified"));
        }
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind:FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnAdd {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut flow = FlowContext::new();
        let mut has_real = false;
        let mut has_double = false;
        let mut i64_value = 0;
        let mut f32_value = 0.0;
        let mut f64_value = 0.0;
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            match input {
                Point::Int(p) => i64_value = i64::checked_add(i64_value, p.value)
                    .ok_or_else(|| err!(self.id, "Overflow: `{:?} + {:?}`", i64_value, p.value).to_string())?,
                Point::Real(p) => {
                    if p.value.is_nan() {
                        return Err(format!("{}.out | Invalid input '{}': NAN, expected bool or number", self.id, p.name));
                    }
                    has_real = true;
                    f32_value += p.value;
                }
                Point::Double(p) => {
                    if p.value.is_nan() {
                        return Err(format!("{}.out | Invalid input '{}': NAN, expected bool or number", self.id, p.name));
                    }
                    has_double = true;
                    f64_value += p.value;
                }
                Point::Bool(_) | Point::String(_) | Point::Bytes(_) => return Err(format!("{}.out | Invalid input type '{:?}', expected number", self.id, input.typ())),
            }
        }
        if has_double {
            flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, i64_value as f64 + f32_value as f64 + f64_value)))
        } else if has_real {
            flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, i64_value as f32 + f32_value)))
        } else {
            flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, i64_value)))
        }
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnAdd instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic tests
}
mod fn_sub {
use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// ### Function | `FnSub`
///
/// Возвращает разность `input1 - input2`,
/// динамически повышая тип данных до наиболее точного.
///
/// **Example**
/// ```yaml
/// fn Sub:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Sub:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnSub {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
//
impl FnSub {
    ///
    /// Creates `FnSub` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnSub{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnSub {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        // TODO Add overflow check
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1 = match input {
            Point::Bool(p) => NumValue::Bool(p.value.0),
            Point::Int(p) => NumValue::Int(p.value),
            Point::Real(p) => NumValue::Real(p.value),
            Point::Double(p) => NumValue::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2 = match input {
            Point::Bool(p) => NumValue::Bool(p.value.0),
            Point::Int(p) => NumValue::Int(p.value),
            Point::Real(p) => NumValue::Real(p.value),
            Point::Double(p) => NumValue::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let value = (v1 - v2).map_err(|_| format!("{}.out | Can't sub {:?} - {:?}", self.id, v1, v2))?;
        match value {
            NumValue::Bool(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value as i64))),
            NumValue::Int(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value))),
            NumValue::Real(value) => flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, value))),
            NumValue::Double(value) => flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, value))),
        }
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnSub instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_mul {
use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use crate::{
    domain::{FnOutRef, PointMeta},
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// ### Function | `FnMul`
///
/// Возвращает произведение всех входов,
/// динамически повышая тип данных до наиболее точного.
///
/// **Example**
/// ```yaml
/// fn Mul:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Mul:
///     in1: point bool '/App/Service/Point.Name1'
///     in2: point bool '/App/Service/Point.Name2'
///     in3: point bool '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnMul {
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
//
impl FnMul {
    ///
    /// Returns `FnMul` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnMul{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        if inputs.len() < 1 {
            return Err(Error::new(&id, "new").err("At least one input must be specified"));
        }
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnMul {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut flow = FlowContext::new();
        let mut has_real = false;
        let mut has_double = false;
        let mut i64_value = 1;
        let mut f32_value = 1.0;
        let mut f64_value = 1.0;
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            match input {
                Point::Bool(p) => i64_value *= p.value.0 as i64,
                Point::Int(p) => i64_value *= p.value,
                Point::Real(p) => {
                    if p.value.is_nan() {
                        return Err(format!("{}.out | Invalid input '{}': NAN, expected bool or number", self.id, p.name));
                    }
                    has_real = true;
                    f32_value *= p.value;
                }
                Point::Double(p) => {
                    if p.value.is_nan() {
                        return Err(format!("{}.out | Invalid input '{}': NAN, expected bool or number", self.id, p.name));
                    }
                    has_double = true;
                    f64_value *= p.value;
                }
                Point::String(_) | Point::Bytes(_) => return Err(format!("{}.out | Invalid input type '{:?}', expected bool or number", self.id, input.typ())),
            }
        }
        if has_double {
            flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, i64_value as f64 * f32_value as f64 * f64_value)))
        } else if has_real {
            flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, i64_value as f32 * f32_value)))
        } else {
            flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, i64_value)))
        }
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnMul instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_div {
use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnDiv`
///
/// Возвращает результат деления `input1 / input2`,
/// динамически повышая тип данных до наиболее точного.
///
/// **Внимание (Целочисленное деление):** Если оба входа имеют тип `Int` или `Bool`,
/// деление выполняется с отсечением дробной части (например, `5 / 2 = 2`).
/// Для получения результата с плавающей точкой (например, `2.5`), хотя бы один
/// из источников должен быть приведен к типу `Real` или `Double`.
///
/// **Example**
/// ```yaml
/// fn Div:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Div:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
#[derive(Debug)]
pub struct FnDiv {
    txid: usize,
    kind: FnKind,
    input1: FnOutRef,
    input2: FnOutRef,
    inputs: [FnOutRef; 2],
    id: String,
}
//
//
impl FnDiv {
    ///
    /// Creates `FnDiv` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnDiv{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            input1: inputs[0].clone(),
            input2: inputs[1].clone(),
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
//
impl FnOut for FnDiv {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        // TODO Div overflow check
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1 = match input {
            Point::Bool(p) => NumValue::Bool(p.value.0),
            Point::Int(p) => NumValue::Int(p.value),
            Point::Real(p) => NumValue::Real(p.value),
            Point::Double(p) => NumValue::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2 = match input {
            Point::Bool(p) => NumValue::Bool(p.value.0),
            Point::Int(p) => NumValue::Int(p.value),
            Point::Real(p) => NumValue::Real(p.value),
            Point::Double(p) => NumValue::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let value = (v1 / v2).map_err(|_| format!("{}.out | Can't div {:?} / {:?}", self.id, v1, v2))?;
        match value {
            NumValue::Bool(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value as i64))),
            NumValue::Int(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value))),
            NumValue::Real(value) => flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, value))),
            NumValue::Double(value) => flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, value))),
        }
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnDiv instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Test
}
mod fn_pow {
use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// ### Function | `FnPow`
///
/// Выполняет операцию вычисления математической степени `v1 ^ v2` над входящими точками данных.
/// Динамически повышает тип данных до наиболее точного.
///
/// **Example**
/// ```yaml
/// fn Pow:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Pow:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
#[derive(Debug)]
pub struct FnPow {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
impl FnPow {
    ///
    /// Returns `FnPow` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnPow{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnPow {
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1 = match input {
            Point::Bool(p) => NumValue::Bool(p.value.0),
            Point::Int(p) => NumValue::Int(p.value),
            Point::Real(p) => NumValue::Real(p.value),
            Point::Double(p) => NumValue::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2 = match input {
            Point::Bool(p) => NumValue::Bool(p.value.0),
            Point::Int(p) => NumValue::Int(p.value),
            Point::Real(p) => NumValue::Real(p.value),
            Point::Double(p) => NumValue::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let value = v1.pow(v2).map_err(|_| format!("{}.out | Can't pow {:?} ^ {:?}", self.id, v1, v2))?;
        match value {
            NumValue::Bool(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value as i64))),
            NumValue::Int(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value))),
            NumValue::Real(value) => {
                if value.is_nan() || value.is_infinite() {
                    return Err(format!("{}.out | Math error: NaN or Infinite result", self.id));
                }
                flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, value)))
            }
            NumValue::Double(value) => {
                if value.is_nan() || value.is_infinite() {
                    return Err(format!("{}.out | Math error: NaN or Infinite result", self.id));
                }
                flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, value)))
            }
        }
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnPow instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
pub use fn_bit_and::*;
pub use fn_bit_or::*;
pub use fn_bit_xor::*;
pub use fn_not::*;
pub use fn_or::*;
pub use fn_and::*;
pub use fn_add::*;
pub use fn_sub::*;
pub use fn_mul::*;
pub use fn_div::*;
pub use fn_pow::*;
}
mod plot {
//!
//! Display input points on the various types of diagrams.
//!
//! **Note !** To activate fn Plot use:
//! - `cargo test --features=plot` or
//! - `cargo run --features=plot`
mod fn_plot {
use sal_sync::sync::channel::Sender;
use std::{sync::{atomic::{AtomicUsize, Ordering}}, thread};
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
use lazy_static::lazy_static;
///
/// Function | Displaying values of the inputs on the diagram
/// - 'x' - input of the x-values, default current time
/// - 'any input' - y-values, name of input displayed in the legend
/// - 'legend' - legend wil be displayed if true
/// - 'enable' - enables functionality
/// - Returns Ok(None)
///
/// **Note !** To activate fn Plot use:
/// - `cargo test --features=plot` or
/// - `cargo run --features=plot`
///
#[derive(Debug)]
pub struct FnPlot {
    kind: FnKind,
    x: Option<FnOutRef>,
    inputs: Vec<(String, FnOutRef)>,
    plot_send: Sender<(String, egui::accesskit::Point)>,
    id: String,
}
//
//
impl FnPlot {
    ///
    /// Creates new instance of the FnPlot
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, x: Option<FnOutRef>, inputs: impl IntoIterator<Item = (String, FnOutRef)>) -> Self {
        let id = format!("{}/FnPlot{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            kind: FnKind::Fn,
            x,
            inputs: inputs.into_iter().collect(),
            plot_send: UI_PLOT.clone(),
            id,
        }
    }
}
//
//
impl FnOut for FnPlot {
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
        if let Some(x) = &self.x {
            inputs.append(&mut x.borrow().inputs());
        }
        for (_, input) in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let flow = FlowContext::new();
        let inputs: Vec<(&String, Result<Option<FnFlow>, String>)> = self.inputs.iter()
            .map(|(k, f)| (k, f.borrow_mut().out()))
            .collect();
        for (name, input) in inputs {
            match flow.ignore(input) {
                Ok(Some(value)) => {
                    log::trace!("{}.out | value: {:?}", self.id, value);
                    let d = value.ts();
                    let secs = d.timestamp() as f64 ;
                    let nanos = (d.timestamp_subsec_nanos() as f64) / 1_000_000_000.0;
                    let x = secs + nanos;
                    let send = (name.to_owned(), egui::accesskit::Point::new(x, value.to_double().as_double().value));
                    if let Err(err) = self.plot_send.send(send) {
                        log::error!("{}.out | Send error: {:#?}", self.id, err);
                    }
                }
                Ok(None) => log::error!("{}.out | None on input '{}'", self.id, name),
                Err(err) => log::error!("{}.out | Error on input '{}': {:?}", self.id, name, err),
            }
        }
        Ok(None)
    }
    //
    //
    fn reset(&mut self) {
        if let Some(x) = &self.x {
            x.borrow_mut().reset();
        }
        for (_, input) in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnPlot instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
lazy_static! {
    static ref UI_PLOT: Sender<(String, egui::accesskit::Point)> = ui_plot();
}
// #[cfg(not(feature = "plot"))]
#[cfg(feature = "plot")]
fn ui_plot() -> Sender<(String, egui::accesskit::Point)> {
    use sal_sync::sync::channel;
    let (send, recv) = channel::unbounded();
    thread::spawn(|| {
        let event_loop_builder: Option<eframe::EventLoopBuilderHook> = Some(Box::new(|event_loop_builder| {
            // event_loop_builder.build().unwrap();
            winit::platform::x11::EventLoopBuilderExtX11::with_any_thread(event_loop_builder, true);
        }));
        eframe::run_native(
            "TaskPlot",
            eframe::NativeOptions {
                // fullscreen: true,
                // maximized: true,
                event_loop_builder,
                viewport: egui::ViewportBuilder::default()
                    .with_min_inner_size([ 1920.0, 840.0]),
                ..Default::default()
            },
            Box::new(|cc| Ok(Box::new(
                super::ui_plot::UiPlot::new(
                    "", //parent,
                    cc,
                    recv,
                ),
            ))),
        ).unwrap();
    });
    send
}
// #[cfg(feature = "plot")]
#[cfg(not(feature = "plot"))]
fn ui_plot() -> Sender<(String, egui::accesskit::Point)> {
    use sal_sync::sync::channel;
    let (send, recv) = channel::unbounded();
    println!(
        "fn_plot.ui_plot | To activate fn Plot use: \n\t`cargo test --features=plot` or \n\t`cargo run --features=plot`",
    );
    thread::spawn(move || {
        loop {
            if let Err(err) = recv.recv_timeout(sal_sync::services::RECV_TIMEOUT) {
                use sal_sync::sync::channel::RecvTimeoutError;
                match err {
                    RecvTimeoutError::Timeout => {},
                    _ => break,
                }
            }
        }
    });
    send
}
}
#[cfg(feature = "plot")]
mod ui_plot {
use std::{cell::RefCell, rc::Rc, sync::Arc};
use eframe::CreationContext;
use egui_plot::{Line, Plot, Points};
use hsl::HSL;
use indexmap::IndexMap;
use egui::{
    accesskit::Point, vec2, Align2, Color32, FontFamily, FontId, TextStyle
};
use sal_sync::{kernel::state::ChangeNotify, sync::channel::Receiver};
///
/// Plot the point values
/// used in `FnPlot`
pub struct UiPlot {
    id: String,
    // renderDelay: Duration,
    real_input_min_y: f64,
    real_input_max_y: f64,
    real_input_autoscale_y: bool,
    show_events: bool,
    input: Receiver<(String, Point)>,
    plot_style: IndexMap<String, Rc<RefCell<PlotStyle>>>,
    points: IndexMap<String, Vec<[f64; 2]>>,
    events: Vec<String>,
    #[allow(unused)]
    status: Rc<RefCell<ChangeNotify<UiStatus, String>>>,
}
//
//
impl UiPlot {
    ///
    ///
    pub fn new(
        parent: impl Into<String>,
        cc: &CreationContext,
        recv: Receiver<(String, Point)>,
        // renderDelay: Duration,
    ) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);
        Self::configure_text_styles(&cc.egui_ctx);
        let self_id = format!("{}/UiPlot", parent.into());
        let status = Rc::new(RefCell::new(ChangeNotify::new(
            &self_id,
            UiStatus::Ok,
            vec![
                (UiStatus::Ok,  Box::new(|message| log::info!("{}", message))),
                (UiStatus::Err, Box::new(|message| log::warn!("{}", message))),
            ],
        )));
        Self {
            id: self_id,
            real_input_min_y: -10.0,
            real_input_max_y: 200.0,
            // real_input_len: 1024,
            // realInputAutoscroll: true,
            real_input_autoscale_y: false,
            show_events: false,
            // send,
            input: recv,
            plot_style: IndexMap::new(),
            points: IndexMap::new(),
            events: vec![],
            status,
        }
    }
    ///
    ///
    fn setup_custom_fonts(ctx: &egui::Context) {
        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();
        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "Icons".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "./../../../../../assets/fonts/icons.ttf"
            ))),
        );
        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Icons".to_owned());
        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("Icons".to_owned());
        // Tell egui to use these fonts:
        ctx.set_fonts(fonts);
    }
    ///
    fn configure_text_styles(ctx: &egui::Context) {
        use FontFamily::{Monospace, Proportional};
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(24.0, Proportional)),
            // (heading2(), FontId::new(22.0, Proportional)),
            // (heading3(), FontId::new(19.0, Proportional)),
            (TextStyle::Body, FontId::new(16.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (TextStyle::Button, FontId::new(16.0, Proportional)),
            (TextStyle::Small, FontId::new(8.0, Proportional)),
        ].into();
        ctx.set_style(style);
    }
    ///
    /// Generates different color
    fn different_color(&self, index: usize) -> Color32 {
        let colors = self.points.len() as f64;
        let h = ((index as f64) * (360.0 / colors)) % 360.0;
        let rgb = HSL { h, s: 1.0, l: 0.5 }.to_rgb();
        Color32::from_rgb(rgb.0, rgb.1, rgb.2)
    }
}
///
///
impl eframe::App for UiPlot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let window_size = match ctx.input(|i| i.viewport().inner_rect) {
            Some(rect) => rect,
            None => ctx.input(|i: &egui::InputState| i.screen_rect),
        };
        let head_hight = 34.0;
        while let Ok(Some((name, value))) = self.input.try_recv() {
            self.points.entry(name.clone())
                .or_insert(vec![[value.x, value.y]])
                .push([value.x, value.y]);
            self.events.push(format!("{}: {:.3} ", name, value.y));
            let color = self.different_color(self.plot_style.len());
            self.plot_style.entry(name)
                .or_insert(Rc::new(RefCell::new(PlotStyle { show: true, square: true, width: 1.0, color, line: PlotLineStyle::Dots, scale: Scale::default() })));
        }
        // match self.input.try_recv() {
        //     Ok((name, value)) => {
        //         self.points.entry(name.clone())
        //             .or_insert(vec![[value.x, value.y]])
        //             .push([value.x, value.y]);
        //         self.events.push(format!("{}: {:.3} ", name, value.y));
        //         let color = self.different_color(self.plot_style.len());
        //         self.plot_style.entry(name)
        //             .or_insert(Rc::new(RefCell::new(PlotStyle { show: true, square: true, width: 1.0, color, line: PlotLineStyle::Dots, scale: Scale::default() })));
        //     }
        //     Err(err) => {
        //         self.status.borrow_mut().add(UiStatus::Err, &format!("{}.update | self.input.recv error: {:?}", self.id, err));
        //     }
        // };
        egui::Window::new("Settings")
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .default_size(vec2(0.4 * window_size.width(), 0.5 * (window_size.height() - head_hight)))
            .show(ctx, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .column(egui_extras::Column::initial(32.0))
                    .column(egui_extras::Column::initial(250.0))
                    .column(egui_extras::Column::initial(72.0))
                    .column(egui_extras::Column::initial(72.0))
                    .column(egui_extras::Column::initial(72.0))
                    .column(egui_extras::Column::initial(72.0))
                    .header(20.0, |mut header| {
                        header.col(|ui| {ui.label("-");});
                        header.col(|ui| {ui.label("Name");});
                        header.col(|ui| {ui.label("Line");});
                        header.col(|ui| {ui.label("Bold");});
                        header.col(|ui| {ui.label("Square");});
                        header.col(|ui| {ui.label("Y-Scale");});
                    })
                    .body(|mut body| {
                        for (i, (name, style)) in self.plot_style.iter().enumerate() {
                            let color = style.borrow().color;
                            body.row(32.0, |mut row| {
                                row.col(|ui| {
                                    ui.checkbox(&mut (*style.borrow_mut()).show, "");
                                });
                                row.col(|ui| {
                                    ui.label(egui::RichText::new(format!("{:?}\t|\t{:?}", i, name)).color(color));
                                });
                                row.col(|ui| {
                                    let mut is_line = style.borrow().line == PlotLineStyle::Line;
                                    ui.add(egui::Checkbox::without_text(&mut is_line));
                                    // ui.checkbox(&mut is_line, "");
                                    (*style.borrow_mut()).line = if is_line {PlotLineStyle::Line} else {PlotLineStyle::Dots};
                                });
                                row.col(|ui| {
                                    let mut is_bold = style.borrow().width == 2.0;
                                    ui.add(egui::Checkbox::without_text(&mut is_bold));
                                    (*style.borrow_mut()).width = if is_bold {2.0} else {1.0};
                                });
                                row.col(|ui| {
                                    ui.add(egui::Checkbox::without_text(&mut (*style.borrow_mut()).square));
                                });
                                row.col(|ui| {
                                    let mut y_scale = style.borrow().scale.y.to_string();
                                    if ui.add(egui::TextEdit::singleline(&mut y_scale)).changed() {
                                        if let Ok(value) = y_scale.parse() {
                                            (*style.borrow_mut()).scale.y = value
                                        };
                                    };
                                });
                            });
                        }
                    });
            });
        if self.show_events {
            egui::Window::new("Events")
                .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
                .default_size(vec2(0.4 * window_size.width(), 0.5 * (window_size.height() - head_hight)))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for (i, event) in self.events.iter().enumerate() {
                            ui.label(format!("{:?}\t|\t{:?}", i, event));
                            ui.separator();
                        }
                    });
                });
        }
        egui::Window::new(self.id.clone())
            // .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .default_size(vec2(0.8 * window_size.width(), 0.8 * window_size.height() - head_hight))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [200.0, 16.0],
                        egui::Label::new(format!("window width: {:?}", window_size.width())),
                    );
                    ui.label(format!("max length: {}", 0));
                    ui.separator();
                    ui.checkbox(&mut self.show_events, "Events");
                    ui.separator();
                    ui.checkbox(&mut self.real_input_autoscale_y, "Autoscale Y");
                    ui.separator();
                });
                ui.separator();
                let mut min = format!("{}", self.real_input_min_y);
                let mut max = format!("{}", self.real_input_max_y);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [32.0, 16.0 * 2.0 + 6.0],
                        egui::Label::new(format!("↕")), //⇔⇕   ↔
                    );
                    ui.separator();
                    ui.vertical(|ui| {
                        if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut max)).changed() {
                            if !self.real_input_autoscale_y {
                                self.real_input_max_y = match max.parse() {Ok(value) => {value}, Err(_) => {self.real_input_max_y}};
                            }
                        };
                        if ui.add_sized([64.0, 16.0], egui::TextEdit::singleline(&mut min)).changed() {
                            if !self.real_input_autoscale_y {
                                self.real_input_min_y = match min.parse() {Ok(value) => {value}, Err(_) => {self.real_input_min_y}};
                            }
                        };
                    });
                });
                let mut plot = Plot::new(self.id.clone());
                if !self.real_input_autoscale_y {
                    plot = plot.include_y(self.real_input_min_y);
                    plot = plot.include_y(self.real_input_max_y);
                }
                plot.show(ui, |plot_ui| {
                    let mut prev = None;
                    for (_i, (label, points)) in self.points.iter().enumerate() {
                        match self.plot_style.get(label) {
                            Some(plot_style) => {
                                if plot_style.borrow().show {
                                    match plot_style.borrow().line {
                                        PlotLineStyle::Dots => {
                                            let scale = plot_style.borrow().scale.clone();
                                            plot_ui.points(
                                                Points::new(
                                                    "",
                                                    points.iter().map(|p| [p[0], scale.scale_y(p[1])] ).collect::<Vec<[f64; 2]>>()
                                                )
                                                    .name(label)
                                                    .color(plot_style.borrow().color)
                                                    .radius(plot_style.borrow().width)
                                                    .filled(true),
                                            );
                                        }
                                        PlotLineStyle::Line => {
                                            let square = plot_style.borrow().square;
                                            let scale = plot_style.borrow().scale.clone();
                                            plot_ui.line(
                                                Line::new(
                                                    "",
                                                    points.iter().fold(Vec::<[f64; 2]>::new(), |mut acc, p| {
                                                        if square {
                                                            if let Some(prev_) = &prev {
                                                                if p[1] != *prev_ {acc.push([p[0], scale.scale_y(*prev_)])}
                                                            };
                                                            prev = Some(p[1]);
                                                        }
                                                        acc.push([p[0], scale.scale_y(p[1])]);
                                                        acc
                                                    })
                                                    // points.iter().map(|p| [p[0], plot_style.borrow().scale.scale_y(p[1])] ).collect::<Vec<[f64; 2]>>()
                                                )
                                                    .color(plot_style.borrow().color),
                                            );
                                        }
                                    }
                                }
                            }
                            None => log::error!("{}.update | Unknown plot key '{}'", self.id, label),
                        }
                    }
                });
            });
        // std::thread::sleep(self.renderDelay);
        ctx.request_repaint();
    }
}
///
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum UiStatus {
    Ok,
    Err,
}
///
///
#[derive(Clone, Debug, PartialEq)]
struct PlotStyle {
    show: bool,
    square: bool,
    width: f32,
    line: PlotLineStyle,
    color: Color32,
    scale: Scale,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PlotLineStyle {
    Dots,
    Line,
}
#[derive(Clone, Debug, PartialEq)]
struct Scale {
    x: f64,
    y: f64,
}
impl Default for Scale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}
impl Scale {
    #[allow(unused)]
    pub fn scale_x(&self, x: f64) -> f64 {
        x * self.x
    }
    pub fn scale_y(&self, y: f64) -> f64 {
        y * self.y
    }
}
// pub trait ExtendedColors {
//     const orange: Color32 = Color32::from_rgb(255, 152, 0);
//     const orangeAccent: Color32 = Color32::from_rgb(255, 152, 0);
//     const lightGreen10: Color32 = Color32::from_rgba_premultiplied(0x90, 0xEE, 0x90, 10);
//     fn with_opacity(&self, opacity: u8) -> Self;
// }
// impl ExtendedColors for Color32 {
//     fn with_opacity(&self, opacity: u8) -> Self {
//         let [r, g, b, _] = self.to_array();
//         Color32::from_rgba_premultiplied(r, g, b, opacity)
//     }
// }
}
pub use fn_plot::*;
#[cfg(feature = "plot")]
pub use ui_plot::*;
}
mod sql {
//!
//! `Task` Service functions intended for the API/SQL purposes
mod fn_sql {
use function_name::named;
use sal_core::error::Error;
use sal_sync::{collections::FxIndexMap, services::{Services, entity::{Name, Point, PointHlr}, task::functions::FnConfig}};
use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}};
use crate::{
    domain::{
        FnOutRef, PointMeta, format::{FormatPoint, Sufix}
    }, err, err_pass, services::task::{
        FlowContext, FnFlow, functions::{FnBuilder, FnKind, FnOut, FnResult}, task_nodes::TaskNodes
    }
};
///
/// ### Function | FnSql
///
/// Строит SQL-запрос, подставляя актуальные значения входов вместо маркеров {xyz}.
///
/// Является чистой функцией (Stateless), не хранит внутренний кэш и пересобирает строку при каждом такте.
///
/// **Example 1**
/// - `input1.value = 'Valid'`
/// - `input2.value = 10`
/// - `inpur1.timestamp = '20'`
/// - `input1.status = Ok`
/// "UPDATE table SET kind = '{input1}' WHERE id = '{input2}';"    =>  UPDATE table SET kind = 'Valid' WHERE id = '10';
///
/// **Example 2**
/// ```yaml
/// fn Sql:
///     sql: "UPDATE table_name SET value = '{input1}' WHERE id = '{input2}';"
///     input1: point int '/path/Point.Name'
///     input2: const int 11
/// ```
#[derive(Debug)]
pub struct FnSql {
    txid: usize,
    kind: FnKind,
    /// `Map<marker, (input, name, sufix)>`
    inputs: FxIndexMap<String, (FnOutRef, String, Sufix)>,
    sql: FormatPoint,
    id: String,
}
//
//
impl FnSql {
    ///
    /// Returns `FnSql` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `inputs`: Вектор входных сигналов, должен содержать не менее одного входа
    /// - `nodes`: Граф `TaskNodes`
    /// - `services`: Ссылка на контейнер всех сервисов
    #[named]
    pub fn new(parent: impl Into<String>, conf: &FnConfig, nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnSql, Error> {
        let self_name = Name::new(parent, format!("FnSql{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let id = self_name.join();
        let txid = nodes.txid();
        let sql = conf.param("sql")
            .ok_or_else(|| err!(id, "Can't find 'sql'"))?
            .as_param();
        let sql = sql.conf.as_str()
            .ok_or_else(|| err!(id, "Wrong conf in 'sql': {:?}", sql.conf))?;
        let sql = FormatPoint::new(sql).map_err(|err| err_pass!(id, err))?;
        let markers = sql.markers();
        let mut inputs = FxIndexMap::default();
        for (marker, (name, sufix)) in markers {
            log::trace!("{}.new | input name: {:?}", id, name);
            let input_conf = conf.input_conf(&name)
                .map_err(|err| err_pass!(id, err, "Can't get input '{name}' conf"))?;
            inputs.insert(
                marker,
                (
                    FnBuilder::new(&self_name, input_conf, nodes, services.clone())
                        .map_err(|err| err_pass!(id, err, "Can't build input '{name}'"))?,
                    name,
                    sufix,
                )
            );
        }
        Ok(FnSql {
            txid,
            kind: FnKind::Fn,
            inputs,
            sql,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnSql {
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
        for (_, (input, _, _)) in &self.inputs {
            inputs.extend(input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let inputs: FxIndexMap<&String, (FnResult<FnFlow, String>, &String, &Sufix)> = self.inputs.iter().map(|(marker, (input, name, sufix))| {
            (marker, (input.borrow_mut().out(), name, sufix))
        }).collect();
        let mut flow = FlowContext::new();
        let mut meta = PointMeta::default();
        for (marker, (input, name, _sufix)) in inputs {
            // log::trace!("{}.out | name: {:?}, sufix: {:?}", self_id, name, sufix);
            log::trace!("{}.out | input: {:?} - found", self.id, name);
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            self.sql.insert(marker, input);
        }
        let value = self.sql.out();
        log::trace!("{}.out | sql: {:?}", self.id, self.sql.out());
        flow.wrap(Point::String(Self::point_with(self.txid, &meta, &self.id, value)))
    }
    //
    fn reset(&mut self) {
        for (_, (input, _, _)) in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnSql instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
pub use fn_sql::*;
}
mod timers {
//!
//! # `Task` Service | Time based functions
//!
//! ## `fn TimerOnDelay` (TON)
//!
//! **Delayed activation of the output.**
//!
//! - Retirns TRUE only after the Input has remained TRUE for the specified duration.
//! - If Input drops to FALSE at any time, Output immediately resets to FALSE and the timer clears.
//! - Useful for signals debouncing, defining persistent conditions or staggering
//!
//!
//! ## `fn TimerOffDelay` (TOF)
//!
//! **Extends the duration of a signal after it ends.**
//!
//! - Returns TRUE immediately when the Input has TRUE.
//! - When Input becomes to FALSE, Output remains TRUE for the specified duration.
//! - If Input becomes TRUE again during the cooldown, the timer resets and Output stays TRUE.
//! - Useful for cool-down device, interior lighting delays, or maintaining a state during brief signal dropouts.
//!
//! ## fn TimerPulse (TP)
//!
//! **Generates a single pulse of a fixed length.**
//!
//! - Returns TRUE for the specified duration only after Input becomes to TRUE,
//! regardless of how long Input stays TRUE or if it drops to FALSE early.
//! - The timer is non-retriggerable; it must finish the pulse before it can be started again.
//! - Useful for consistent trigger pulses, valve pulsing, or triggering a "one-shot" physical action.
//!
//! ## fn Timer
//!
//! **Measures the time while Input is TRUE**
//!
//! - Returns time elapsed in seconds (double) since Input raised (>0) to dropped (<=0)
//! - If option `repeat` = true, then returns total elapsed secods of multiple periods
//!
mod fn_timer_off_delay {
use function_name::named;
use sal_core::error::Error;
use concat_string::concat_string;
use sal_sync::services::{conf::ConfDuration, entity::{Point, PointHlr}, types::Bool};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
use crate::{
    domain::{FnOutRef, TryTo}, err_pass, services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | `FnTimerOffDelay`
///
/// Таймер задержки выключения TOF (Timer-Off-Delay)
///
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `reset`: Доминантный сбрас. При `true` обнуляет выход в `false` и сбрасывает секундомер.
/// - `input`: `true` - сразу проходит на выход, `false` - пройдет на выход по окончании заданного `duration`.
/// - `delay`: `Duration`
/// - Вернет `true` сразу как на входе `true`, сброс произойдет с задержкой в заданый `duration`.
#[derive(Debug)]
pub struct FnTimerOffDelay {
    id: String,
    kind: FnKind,
    reset: Option<FnChange>,
    delay: Duration,
    input: FnChange,
    active_t: Option<Instant>,
    trigg: Option<bool>,
    state: Option<bool>,
    ts: chrono::DateTime<chrono::Utc>,
}
//
impl FnTimerOffDelay {
    ///
    /// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
    /// - `reset`: Сбрасывает секундомер и выход по переднему фронту сигнала (переход 0 -> 1).
    /// - `input`: `true` - сразу проходит на выход, `false` - пройдет на выход по окончании заданного `duration`.
    /// - `delay`: `Duration`
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, delay: ConfDuration, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnTimerOffDelay{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            reset: reset.map(FnChange::new),
            delay: delay.to_duration(),
            input: FnChange::new(input),
            active_t: None,
            trigg: None,
            state: None,
            ts: chrono::Utc::now(),
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T, t: chrono::DateTime<chrono::Utc>) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), t)
    }
}
//
impl FnOut for FnTimerOffDelay {
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
        let mut inputs = self.input.inputs();
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.inputs());
        }
        inputs
    }
    //
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let reset = self.reset.as_mut().map(|f| f.out());
        let input = self.input.out();
        let flow = FlowContext::new();
        let reset = if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if reset {
                    self.active_t = None;
                    self.trigg = None;
                }
                reset
            } else { false }
        } else { false };
        let Some(input) = flow.ignore(input)? else {
            self.active_t = None;
            self.trigg = None;
            self.state = None;
            return Ok(None);
        };
        let is_active: bool = (&input).try_to().map_err(|err: Error| err_pass!(self.id, err, "Invalid input").to_string())? && !reset;
        log::trace!("{} | Input: {}", self.id, is_active);
        let value = match (self.trigg, is_active) {
            (_, true) => {
                self.active_t = None;
                true
            }
            (None, false) => false,
            (Some(true), false) => {
                if self.delay.is_zero() {
                    false
                } else {
                    let t = *self.active_t.get_or_insert_with(Instant::now);
                    t.elapsed() <= self.delay
                }
            }
            (Some(false), false) => {
                if let Some(t) = self.active_t {
                    t.elapsed() <= self.delay
                } else {
                    false
                }
            }
        };
        if !value {
            self.active_t = None;
        }
        self.trigg = Some(is_active);
        let is_changed = match (self.state, value) {
            (None, _) => true,
            (Some(false), true) => true,
            (Some(true), false) => true,
            _ => false,
        };
        self.state = Some(value);
        if is_changed {
            self.ts = chrono::Utc::now();
        }
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value), self.ts));
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.active_t = None;
        self.trigg = None;
        self.state = None;
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.input.reset();
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_timer_on_delay {
use function_name::named;
use sal_core::error::Error;
use concat_string::concat_string;
use sal_sync::services::{conf::ConfDuration, entity::{Point, PointHlr}, types::Bool};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
use crate::{
    domain::{FnOutRef, Level, LevelTrigger, TryTo}, err_pass, services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | `FnTimerOnDelay`
///
/// Таймера задержки включения — TON (Timer On-Delay)
///
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `reset`: Сбрасывает секундомер по переднему фронту сигнала (переход 0 -> 1).
/// - `input`: `true` - активирует секундомер, `false` - сбрасывает секундрмер,
/// - `delay`: `Duration`
/// - Вернет `true` если секундомер насчитал заданый `duration`.
#[derive(Debug)]
pub struct FnTimerOnDelay {
    id: String,
    kind: FnKind,
    reset: Option<FnChange>,
    delay: Duration,
    input: FnChange,
    active_t: Option<Instant>,
    trigg: LevelTrigger,
    state: Option<bool>,
}
//
impl FnTimerOnDelay {
    ///
    /// Returns `FnTimerOnDelay` new instance
    /// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
    /// - `reset`: Сбрасывает секундомер по переднему фронту сигнала (переход 0 -> 1).
    /// - `input`: `true` - активирует секундомер, `false` - сбрасывает секундрмер,
    /// - `delay`: `Duration`
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, delay: ConfDuration, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnTimerOnDelay{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            reset: reset.map(FnChange::new),
            delay: delay.to_duration(),
            input: FnChange::new(input),
            active_t: None,
            trigg: LevelTrigger::new(),
            state: None,
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T, t: chrono::DateTime<chrono::Utc>) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), t)
    }
}
//
impl FnOut for FnTimerOnDelay {
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
        let mut inputs = self.input.inputs();
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.inputs());
        }
        inputs
    }
    //
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let reset = self.reset.as_mut().map(|f| f.out());
        let input = self.input.out();
        let flow = FlowContext::new();
        let reset = if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if reset {
                    self.active_t = None;
                    self.trigg.reset();
                }
                reset
            } else { false }
        } else { false };
        let Some(input) = flow.ignore(input)? else {
            self.active_t = None;
            self.trigg.reset();
            self.state = None;
            return Ok(None);
        };
        let is_active: bool = (&input).try_to().map_err(|err: Error| err_pass!(self.id, err, "Invalid input").to_string())? && !reset;
        log::trace!("{} | Input: {}", self.id, is_active);
        let value = match self.trigg.add(is_active) {
            Some(Level::Up) => {
                let t = Instant::now();
                self.active_t = Some(t);
                let val = t.elapsed() >= self.delay;
                // log::debug!("{} | State: Level::Up | {:?} (Go)", self.id, self.active_t.unwrap().elapsed());
                val
            }
            Some(Level::Down) => {
                // log::debug!("{} | State: Level::Down | 0 ms (Stop)", self.id);
                self.active_t = None;
                false
            }
            _ => {
                if let Some(t) = self.active_t {
                    let val = t.elapsed() >= self.delay;
                    // log::debug!("{} | State: None::IsActive: {val} | {:?} (Go)", self.id, self.active_t.unwrap().elapsed());
                    val
                } else {
                    // log::debug!("{} | State: None::NotActive: false | 0 ms (Stop)", self.id);
                    self.active_t = None;
                    false
                }
            }
        };
        let is_changed = match (self.state, value) {
            (None, _) => true,
            (Some(false), true) => true,
            (Some(true), false) => true,
            _ => false,
        };
        self.state = Some(value);
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value), chrono::Utc::now()));
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.active_t = None;
        self.trigg.reset();
        self.state = None;
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.input.reset();
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
}
mod fn_timer {
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr};
use concat_string::concat_string;
use std::{sync::atomic::{AtomicUsize, Ordering}, time::Instant};
use crate::{domain::{Edge, EdgeDetector, FnOutRef, TryTo}, services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult}};
///
/// ### Function | `FnTimer`
///
/// Интегратор времени (накопительный секундомер / моточасы).
/// Считает время в секундах, пока на входе `true` (> 0).
///
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `initial`: Начальное значение. Применяется строго один раз при первом успешном чтении.
/// - `reset`: Сбрасывает накопленную сумму и счетчик если `> 0`.
/// - `input`: `true` - секундомер тикает (отдает `FnFlow::New`),
/// `false` - замирает и хранит значение, отдает его в `Flow::Old`,
/// снова `true` - счет продолжается с точки остановки.
#[derive(Debug)]
pub struct FnTimer {
    id: String,
    kind: FnKind,
    initial: Option<FnChange>,
    reset: Option<FnChange>,
    input: FnChange,
    edge: EdgeDetector,
    first: bool,
    total_t: f64,
    active_t: Option<Instant>,
    ts: chrono::DateTime<chrono::Utc>
}
//
impl FnTimer {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, initial: Option<FnOutRef>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        let first = initial.is_some();
        Self {
            id: format!("{}/FnTimer{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            initial: initial.map(FnChange::new),
            reset: reset.map(FnChange::new),
            input: FnChange::new(input),
            edge: EdgeDetector::new(),
            first,
            total_t: 0.0,
            active_t: None,
            ts: chrono::Utc::now(),
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T, ts: chrono::DateTime<chrono::Utc>) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), ts)
    }
}
//
impl FnOut for FnTimer {
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
        let mut inputs = self.input.inputs();
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.inputs());
        }
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let initial = self.initial.as_mut().map(|f| f.out());
        let reset = self.reset.as_mut().map(|f| f.out());
        let input = self.input.out();
        let flow = FlowContext::new();
        let mut is_changed = false;
        let reset = if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if reset {
                    self.edge.reset();
                    if self.total_t != 0.0 { is_changed = true; }
                    self.total_t = 0.0;
                    self.ts = chrono::Utc::now();
                    self.active_t = None;
                }
                reset
            } else { false }
        } else { false };
        let Some(input) = flow.ignore(input)? else {
            if let Some(t) = self.active_t {
                self.total_t = self.total_t + t.elapsed().as_secs_f64();
                self.ts = chrono::Utc::now();
                self.active_t = None;
                self.edge.reset();
            }
            return Ok(None);
        };
        if self.first {
            if let Some(initial) = initial {
                if let Some(initial) = flow.ignore(initial)? {
                    let initial: f64 = (&initial).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid initial ", err.to_string()))?;
                    self.total_t += initial;
                    is_changed = initial != 0.0;
                    self.first = false;
                }
            }
        }
        // trace!("{}.out | input: {:?}", self.id, self.input.print());
        let is_active: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))? && !reset;
        let elapsed = match self.edge.add(is_active) {
            Some(Edge::Rising) => {
                self.active_t = Some(Instant::now());
                self.total_t
            }
            Some(Edge::Falling) => {
                if let Some(t) = self.active_t {
                    self.total_t = self.total_t + t.elapsed().as_secs_f64();
                    self.ts = chrono::Utc::now();
                }
                is_changed = true;
                self.active_t = None;
                self.total_t
            }
            _ => {
                if let Some(t) = self.active_t {
                    is_changed = true;
                    self.ts = chrono::Utc::now();
                    self.total_t + t.elapsed().as_secs_f64()
                } else {
                    self.total_t
                }
            }
        };
        log::trace!("{}.out | elapsed: {:?}", self.id, self.total_t);
        let point = Point::Double(Self::point_with(&input, &self.id, elapsed, self.ts));
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.edge.reset();
        self.first = true;
        self.total_t = 0.0;
        self.ts = chrono::Utc::now();
        self.active_t = None;
        if let Some(initial) = &mut self.initial {
            initial.reset();
        }
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.input.reset();
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic tests
}
pub use fn_timer_off_delay::*;
pub use fn_timer_on_delay::*;
pub use fn_timer::*;
}
mod fn_builder {
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{LinkName, Services, conf::ConfDuration, entity::{Name, Point, ToPoint}, task::functions::{FnConfKind, FnConfPointType, FnConfig}};
use std::{cell::RefCell, rc::Rc, str::FromStr, sync::Arc};
use indexmap::IndexMap;
use crate::{
    domain::FnOutRef, err_pass, services::task::{
        functions::{functions::Functions, *},
        task_nodes::TaskNodes
    }
};
///
/// Creates nested functions tree from it config
pub struct FnBuilder {}
//
impl FnBuilder {
    ///
    /// Creates nested functions tree from it config
    pub fn new(parent: &Name, conf: &FnConfKind, nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnOutRef, Error> {
        Self::function(parent, "", conf, nodes, services)
        // trace!("{}.function | fn '{}': {:#?}", format!("{}/FnBuilder", parent), conf.borrow().id(), conf);
        // conf
    }
    pub fn get_input_config(parent: &Name, name: &str, conf: &FnConfig, nodes: &mut TaskNodes, services: &Arc<Services>) -> Result<Option<FnOutRef>, Error> {
        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
        Ok(match input_conf {
            Some(input_conf) => Some(Self::function(parent, name, input_conf, nodes, services.clone())?),
            None => None,
        })
    }
    ///
    ///
    #[named]
    fn function(parent: &Name, input_name: &str, conf: &FnConfKind, nodes: &mut TaskNodes, services: Arc<Services>) -> Result<FnOutRef, Error> {
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
                    //
                    Functions::Count => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnCount | Can't get 'initial'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services)
                            .map_err(|err| error.pass_with(format!("FnCount | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnCount::new(parent, initial, input),
                        )))
                    }
                    //
                    Functions::Add => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAdd | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnAdd::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Timer => {
                        let enable = Self::get_input_config(parent, "enable", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(parent, "reset", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get 'reset'"), err))?;
                        let initial = Self::get_input_config(parent, "initial", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get 'initial'"), err))?;
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnTimer | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnTimer | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(FnTimer::new(parent, initial, reset, input), nodes.enable_mode(), enable))),
                            None => Rc::new(RefCell::new(FnTimer::new(parent, initial, reset, input))),
                        })
                    }
                    //
                    Functions::Ton => {
                        let enable = Self::get_input_config(parent, "enable", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(parent, "reset", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnTimerOnDelay | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Can't get 'input'"), err))?;
                        let delay = {
                            let name = "delay";
                            let param = conf.param(name).ok_or(error.err(format!("FnTimerOnDelay | Can't get '{name}'")))?;
                            let delay = param.as_param().conf;
                            let delay = delay.as_str()
                                .ok_or(error.err(format!("FnTimerOnDelay | Wrong conf in '{name}': '{:?}'", param)))?;
                            ConfDuration::from_str(delay)
                                .map_err(|err| error.pass_with(format!("FnTimerOnDelay | Wrong conf in '{name}': {:?}", param), err))?
                        };
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(
                                FnTimerOnDelay::new(parent, reset, delay, input), nodes.enable_mode(), enable))),
                            None => Rc::new(RefCell::new(FnTimerOnDelay::new(parent, reset, delay, input))),
                        })
                    }
                    //
                    Functions::Tof => {
                        let enable = Self::get_input_config(parent, "enable", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(parent, "reset", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnTimerOffDelay | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Can't get 'input'"), err))?;
                        let delay = {
                            let name = "delay";
                            let param = conf.param(name).ok_or(error.err(format!("FnTimerOffDelay | Can't get '{name}'")))?;
                            let delay = param.as_param().conf;
                            let delay = delay.as_str()
                                .ok_or(error.err(format!("FnTimerOffDelay | Wrong conf in '{name}': '{:?}'", param)))?;
                            ConfDuration::from_str(delay)
                                .map_err(|err| error.pass_with(format!("FnTimerOffDelay | Wrong conf in '{name}': {:?}", param), err))?
                        };
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(
                                FnTimerOffDelay::new(parent, reset, delay, input), nodes.enable_mode(), enable))),
                            None => Rc::new(RefCell::new(FnTimerOffDelay::new(parent, reset, delay, input))),
                        })
                    }
                    //
                    Functions::ToApiQueue => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes ,services.clone())
                            .map_err(|err| error.pass_with(format!("FnToApiQueue | Can't get '{name}'"), err))?;
                        let Some(queue_name) = conf.param("queue").map(|v| v.as_param()) else {
                            return Err(error.err(format!("FnToApiQueue | Parameter 'queue' is missed in '{}'", conf.name)));
                        };
                        let queue_name = queue_name.conf.as_str().unwrap();
                        let link_name = LinkName::from_str(queue_name).unwrap();
                        let send_queue = services.get_link(&link_name)
                            .map_err(|err| error.pass_with(format!("FnToApiQueue | Can't get link '{link_name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToApiQueue::new(parent, nodes.txid(), input, send_queue)
                        )))
                    }
                    //
                    Functions::Gt => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnGt | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnGt::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Ge => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnGe | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnGe::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Eq => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnEq | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnEq::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Le => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnLe | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnLe::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Lt => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnLt | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnLt::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Ne => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnNe | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnNe::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Sql | Functions::SqlMetric => {
                        Ok(Rc::new(RefCell::new(
                            FnSql::new(parent, conf, nodes, services)
                                .map_err(|err| err_pass!(dbg, err, "Can't build Sql"))?
                        )))
                    }
                    //
                    Functions::PointId => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnPointId | Can't get '{name}'"), err))?;
                        // debug!("{}.functions | Functions::PointId | input: {:?}", self_id, input);
                        log::debug!("{}.functions | Functions::PointId | requesting points...", dbg);
                        let points = services.points(&parent.join())
                            .then(|points| points, |err| {
                                log::error!("{}.functions | Functions::PointId | Requesting points error: {:?}", dbg, err);
                                vec![]
                            });
                        // debug!("{}.functions | Functions::PointId | points: {:?}", self_id, points);
                        Ok(Rc::new(RefCell::new(
                            FnPointId::new(parent, input, points)
                        )))
                    }
                    //
                    Functions::Debug => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnDebug | Can't get '{name}'"), err))?;
                            inputs.push((name.to_string(), input));
                        }
                        Ok(Rc::new(RefCell::new(
                            FnDebug::new(parent, inputs)
                        )))
                    }
                    Functions::Plot => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "x";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let x = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let mut inputs = IndexMap::new();
                        let mut conf_inputs: IndexMap<String, FnConfKind> = conf.inputs
                            .iter()
                            .filter(|(name, _)| ! ["enable", "x", "legend"].contains(&(name.as_str())))
                            .map(|(n, c)| (n.to_owned(), c.clone())).collect();
                        for (name, input_conf) in &mut conf_inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnPlot | Can't get '{name}'"), err))?;
                            inputs.insert(name.to_owned(), input);
                        }
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(
                                FnPlot::new(parent, x, inputs), nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnPlot::new(parent, x, inputs))),
                        })
                    }
                    //
                    Functions::ToBool => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToBool | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToBool::new(parent, input).map_err(|err| err_pass!(dbg, err))?
                        )))
                    }
                    //
                    Functions::ToInt => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToInt | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToInt::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToReal => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToReal | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToReal::new(parent, input).map_err(|err| err_pass!(dbg, err))?
                        )))
                    }
                    //
                    Functions::ToDouble => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnToDouble | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToDouble::new(parent, input).map_err(|err| err_pass!(dbg, err))?
                        )))
                    }
                    //
                    Functions::Export => {
                        let name = "input";
                        let input_conf = conf.input_conf(name)
                            .map_err(|err| error.pass_with(format!("FnExport | Can't get '{name}'"), err))?;
                        let input = Self::function(parent, name, input_conf, nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnExport | Can't get '{name}'"), err))?;
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, name, input_conf, nodes, services.clone())
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
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(
                                FnExport::new(parent, nodes.txid(), point_conf, input, send_queue), nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnExport::new(parent, nodes.txid(), point_conf, input, send_queue))),
                        })
                    }
                    //
                    Functions::Select => {
                        let select = Self::get_input_config(parent, "select", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnSelect | 'select' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'select'"), err))?;
                        let default = Self::get_input_config(parent, "default", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'default'"), err))?;
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnSelect | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnSelect | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnSelect::new(parent, default, input, select)
                        )))
                    }
                    //
                    Functions::RisingEdge => {
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnRisingEdge | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnRisingEdge | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnRisingEdge::new(parent, input)
                        )))
                    }
                    //
                    Functions::FallingEdge => {
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnFallingEdge | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnFallingEdge | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnFallingEdge::new(parent, input)
                        )))
                    }
                    //
                    Functions::Retain => FnRetain::new(parent, &conf, nodes, &services).map_err(|err| error.pass(err)),
                    //
                    Functions::Acc => {
                        let name = "initial";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let initial = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAcc | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services)
                            .map_err(|err| error.pass_with(format!("FnAcc | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnAcc::new(parent, initial, input),
                        )))
                    }
                    //
                    Functions::Mul => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnMul::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Div => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnDiv::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Sub => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnSub::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::BitAnd => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitAnd | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitAnd::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::BitOr => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitOr | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitOr::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::BitXor => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnBitXor | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnBitXor::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    Functions::Or => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnOr | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnOr::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    Functions::And => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAnd | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnOr::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::Not => {
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnBitNot | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnNot::new(parent, input)
                        )))
                    }
                    //
                    Functions::Threshold => {
                        let enable = Self::get_input_config(parent, "enable", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'enable'"), err))?;
                        let threshold = Self::get_input_config(parent, "threshold", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnThreshold | 'threshold' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'threshold'"), err))?;
                        let factor = Self::get_input_config(parent, "factor", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'factor'"), err))?;
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnThreshold | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnThreshold | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(enable) => Rc::new(RefCell::new(FnEnable::new(
                                FnThreshold::new(parent, threshold, factor, input),
                                nodes.enable_mode(),
                                enable
                            ))),
                            None => Rc::new(RefCell::new(FnThreshold::new(parent, threshold, factor, input))),
                        })
                    }
                    //
                    Functions::Smooth => {
                        let name = "factor";
                        let input_conf = conf.input_conf(name).unwrap();
                        let factor = Self::function(parent, name, input_conf, nodes, services.clone())
                            .map_err(|err| error.pass_with(format!("FnSmooth | Can't get '{name}'"), err))?;
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services)
                            .map_err(|err| error.pass_with(format!("FnSmooth | Can't get '{name}'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnSmooth::new(parent, factor, input).map_err(|err| err_pass!(dbg, err))?
                        )))
                    }
                    //
                    Functions::Average => {
                        let name = "enable";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let enable = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnAverage | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "reset";
                        let input_conf = conf.input_conf(name).map_or(None, |conf| Some(conf));
                        let reset = match input_conf {
                            Some(input_conf) => Some(Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnMax | Can't get '{name}'"), err))?),
                            None => None,
                        };
                        let name = "input";
                        let input_conf = conf.input_conf(name).unwrap();
                        let input = Self::function(parent, name, input_conf, nodes, services)
                            .map_err(|err| error.pass_with(format!("FnAverage | Can't get '{name}'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnAverage::new(parent, reset, input), nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnAverage::new(parent, reset, input))),
                        })
                    }
                    //
                    Functions::Pow => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnSub | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnPow::new(parent, inputs).map_err(|err| error.pass(err))?
                        )))
                    }
                    //
                    Functions::RecOpCycleMetric => {
                        let enable = Self::get_input_config(parent, "enable", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(parent, "reset", conf, nodes, &services)
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
                        let op_cycle = Self::get_input_config(parent, name, conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnRecOpCycleMetric | '{name}' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get '{name}'"), err))?;
                        let mut inputs = IndexMap::new();
                        let conf_inputs = conf.inputs.iter().filter(|(name, _)| {
                            ! ["enable", "reset", "send-to", "conf", "op-cycle"].contains(&name.as_str())
                        });
                        for (name, input_conf) in conf_inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnRecOpCycleMetric | Can't get '{name}'"), err))?;
                            inputs.insert(name.to_owned(), input);
                        }
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(
                                FnRecOpCycleMetric::new(parent, send_to, reset, op_cycle, inputs),
                                nodes.enable_mode(),
                                en,
                            ))),
                            None => Rc::new(RefCell::new(FnRecOpCycleMetric::new(parent, send_to, reset, op_cycle, inputs))),
                        })
                    }
                    //
                    Functions::Max => {
                        let enable = Self::get_input_config(parent, "enable", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(parent, "reset", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnMax | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnMax | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnMax::new(parent, reset, input), nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnMax::new(parent, reset, input))),
                        })
                    }
                    //
                    Functions::Min => {
                        let enable = Self::get_input_config(parent, "enable", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'enable'"), err))?;
                        let reset = Self::get_input_config(parent, "reset", conf, nodes, &services)
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'reset'"), err))?;
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnMin | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnMin | Can't get 'input'"), err))?;
                        Ok(match enable {
                            Some(en) => Rc::new(RefCell::new(FnEnable::new(FnMin::new(parent, reset, input), nodes.enable_mode(), en))),
                            None => Rc::new(RefCell::new(FnMin::new(parent, reset, input))),
                        })
                    }
                    //
                    Functions::PiecewiseLineApprox => {
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
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
                    //
                    Functions::IsChangedValue => {
                        let mut inputs = vec![];
                        for (name, input_conf) in &conf.inputs {
                            let input = Self::function(parent, name, input_conf, nodes, services.clone())
                                .map_err(|err| error.pass_with(format!("FnIsChangedValue | Can't get '{name}'"), err))?;
                            inputs.push(input);
                        }
                        Ok(Rc::new(RefCell::new(
                            FnIsChangedValue::new(parent, inputs)
                        )))
                    }
                    //
                    Functions::Hold => {
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnHold | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnHold | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnHold::new(parent, input)
                        )))
                    }
                    //
                    Functions::ToString => {
                        let input = Self::get_input_config(parent, "input", conf, nodes, &services)
                            .and_then(|v| v.ok_or_else(|| error.err(format!("FnToString | 'input' - is missed"))))
                            .map_err(|err| error.pass_with(format!("FnToString | Can't get 'input'"), err))?;
                        Ok(Rc::new(RefCell::new(
                            FnToString::new(parent, input).map_err(|err| err_pass!(dbg, err))?
                        )))
                    }
                    //
                    // Add a new function here...
                    _ => Err(error.err(format!("Unknown function name: {:?}", conf.name))),
                }
            }
            FnConfKind::Var(conf) => {
                let var_name = conf.name.clone();
                log::trace!("{}.function | Var: {:?}...", dbg, var_name);
                match conf.inputs.iter().next() {
                    //
                    // New var declaration
                    Some((input_conf_name, input_conf)) => {
                        let var = Self::fn_var(
                            var_name,
                            Self::function(parent, input_conf_name, input_conf, nodes, services)
                                .map_err(|err| error.pass_with(format!("Var | Can't get '{input_conf_name}'"), err))?,
                        );
                        log::trace!("{}.function | Var: {:?}: {:?}", dbg, &conf.name, var.clone());
                        nodes.add_var(conf.name.clone(), var.clone())
                            .map_err(|err| error.pass_with(format!("Var | Can't add var '{}'", conf.name), err))?;
                        // log::debug!("{}.function | Var: {:?}", input);
                        Ok(var)
                    }
                    // Usage declared variable
                    None => {
                        let var = nodes.get_var(&var_name)
                            .ok_or(error.err(format!("Var {var_name} - is not declared")))?
                            .to_owned();
                        // let var = nodeVar.var();
                        nodes.add_var_out(conf.name.clone())
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
                    FnConfPointType::Bool => value.parse::<bool>().unwrap().to_point(nodes.txid(), &name),
                    FnConfPointType::Int => value.parse::<i64>().unwrap().to_point(nodes.txid(), &name),
                    FnConfPointType::Real => value.parse::<f32>().unwrap().to_point(nodes.txid(), &name),
                    FnConfPointType::Double => value.parse::<f64>().unwrap().to_point(nodes.txid(), &name),
                    FnConfPointType::String => value.to_point(nodes.txid(), &name),
                    FnConfPointType::Any => Err(error.err(format!("Const '{name}': type 'any' - is not supported")))?,
                    FnConfPointType::Unknown => Err(error.err(format!("Const '{name}': type required")))?,
                };
                let fn_const = Self::fn_const(&name, value);
                // taskNodes.addInput(inputName, input.clone());
                log::trace!("{}.function | Const: {:?} - done", dbg, fn_const);
                Ok(fn_const)
            }
            FnConfKind::Point(conf) => {
                log::trace!("{}.function | Input (Point<{:?}>): {:?} ({:?})...", dbg, conf.type_, input_name, conf.name);
                let point_name = conf.name.clone();
                let input = nodes.add_input(
                    &point_name,
                    Rc::new(RefCell::new(
                        FnInput::new(&point_name, nodes.txid(), conf, &nodes.cycle())
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
                let enable = match conf.enable.as_ref() {
                    Some(input_conf) => Some(Self::function(parent, "enable", input_conf, nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'enable'"), err))?),
                    None => None,
                };
                let input = match conf.input.as_ref() {
                    Some(input_conf) => Some(Self::function(parent, "input", input_conf, nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'input'"), err))?),
                    None => None,
                };
                let changes_only = match conf.changes_only.as_ref() {
                    Some(input_conf) => Some(Self::function(parent, "changes-only", input_conf, nodes, services.clone())
                        .map_err(|err| error.pass_with(format!("PointConf | Can't get 'input'"), err))?),
                    None => None,
                };
                Ok(Rc::new(RefCell::new(
                    FnPoint::new(parent, conf.conf.clone(), enable, changes_only, input, send_to)
                        .map_err(|err| err_pass!(dbg, err))?,
                )))
            }
            FnConfKind::Param(conf) => {
                Err(error.err(format!("Param | Undefined variable or unknown custom parameters in the function conf: {:#?}", conf)))
            }
        }
    }
    ///
    ///
    fn fn_var(parent: impl Into<String>, input: FnOutRef,) -> FnOutRef {
        Rc::new(RefCell::new(
        FnVar::new(parent, input),
        ))
    }
        ///
    ///
    fn fn_const(parent: &str, value: Point) -> FnOutRef {
        Rc::new(RefCell::new(
        FnConst::new(parent, value)
        ))
    }
}
}
mod functions {
//!
//! Here must be defined all functions to be awalible in the `Task` -> FnBuilder
use std::str::FromStr;
///
/// Entair list of public functions
/// supported by FnBuilder
#[derive(Debug)]
pub enum Functions {
    /// core
    Input,
    Const,
    Var,
    /// debuging functions
    Debug,
    Plot,
    /// user defined functions
    Add,
    Count,
    Gt,
    Ge,
    Eq,
    Le,
    Lt,
    Ne,
    Or,
    And,
    /// timers
    Timer,
    Ton,
    Tof,
    /// Conversion
    ToBool,
    ToInt,
    ToReal,
    ToDouble,
    ToString,
    ///
    ToApiQueue,
    ToMultiQueue,
    Sql,
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
    /// Recorder functions
    RecOpCycleMetric,
}
//
//
impl Functions {
    /// embedded functions
    const INPUT                         : &'static str = "input";
    const CONST                         : &'static str = "const";
    const VAR                           : &'static str = "var";
    const DEBUG                         : &'static str = "Debug";
    const PLOT                          : &'static str = "Plot";
    const ADD                           : &'static str = "Add";
    const COUNT                         : &'static str = "Count";
    const GT                            : &'static str = "Gt";
    const GE                            : &'static str = "Ge";
    const EQ                            : &'static str = "Eq";
    const LE                            : &'static str = "Le";
    const LT                            : &'static str = "Lt";
    const NE                            : &'static str = "Ne";
    const OR                            : &'static str = "Or";
    const AND                           : &'static str = "And";
    const TIMER                         : &'static str = "Timer";
    const TON                           : &'static str = "Ton";
    const TIMER_ON_DELAY                : &'static str = "TimerOnDelay";
    const TOF                           : &'static str = "Tof";
    const TIMER_OFF_DELAY               : &'static str = "TimerOffDelay";
    const TO_API_QUEUE                  : &'static str = "ToApiQueue";
    const TO_MULTI_QUEUE                : &'static str = "ToMultiQueue";
    const SQL                           : &'static str = "Sql";
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
            Self::Or                    => Self::OR,
            Self::And                   => Self::AND,
            Self::Input                 => Self::INPUT,
            Self::Timer                 => Self::TIMER,
            Self::Ton                   => Self::TON,
            Self::Tof                   => Self::TOF,
            Self::Var                   => Self::VAR,
            Self::ToApiQueue            => Self::TO_API_QUEUE,
            Self::ToMultiQueue          => Self::TO_MULTI_QUEUE,
            Self::Sql                   => Self::SQL,
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
            Self::ADD                                   => Ok( Self::Add ),
            Self::CONST                                 => Ok( Self::Const ),
            Self::COUNT                                 => Ok( Self::Count ),
            Self::GT                                    => Ok( Self::Gt ),
            Self::GE                                    => Ok( Self::Ge ),
            Self::EQ                                    => Ok( Self::Eq ),
            Self::LE                                    => Ok( Self::Le ),
            Self::LT                                    => Ok( Self::Lt ),
            Self::NE                                    => Ok( Self::Ne ),
            Self::OR                                    => Ok( Self::Or ),
            Self::AND                                   => Ok( Self::And ),
            Self::INPUT                                 => Ok( Self::Input ),
            Self::TIMER                                 => Ok( Self::Timer ),
            Self::TON | Self::TIMER_ON_DELAY            => Ok( Self::Ton ),
            Self::TOF | Self::TIMER_OFF_DELAY           => Ok( Self::Tof ),
            Self::VAR                                   => Ok( Self::Var ),
            Self::TO_API_QUEUE                          => Ok( Self::ToApiQueue ),
            Self::TO_MULTI_QUEUE                        => Ok( Self::ToMultiQueue ),
            Self::SQL                                   => Ok( Self::Sql ),
            Self::SQL_METRIC                            => Ok( Self::SqlMetric ),
            Self::POINT_ID                              => Ok( Self::PointId ),
            Self::DEBUG                                 => Ok( Self::Debug ),
            Self::PLOT                                  => Ok( Self::Plot ),
            Self::TO_BOOL                               => Ok( Self::ToBool ),
            Self::TO_INT                                => Ok( Self::ToInt ),
            Self::TO_REAL                               => Ok( Self::ToReal ),
            Self::TO_DOUBLE                             => Ok( Self::ToDouble ),
            Self::TO_STRING                             => Ok( Self::ToString ),
            Self::EXPORT                                => Ok( Self::Export ),
            Self::SELECT                                => Ok( Self::Select ),
            Self::RISING_EDGE                           => Ok( Self::RisingEdge ),
            Self::FALLING_EDGE                          => Ok( Self::FallingEdge ),
            Self::RETAIN                                => Ok( Self::Retain ),
            Self::ACC                                   => Ok( Self::Acc ),
            Self::MUL                                   => Ok( Self::Mul ),
            Self::DIV                                   => Ok( Self::Div ),
            Self::SUB                                   => Ok( Self::Sub ),
            Self::BIT_AND                               => Ok( Self::BitAnd ),
            Self::BIT_OR                                => Ok( Self::BitOr ),
            Self::BIT_XOR                               => Ok( Self::BitXor ),
            Self::NOT                                   => Ok( Self::Not ),
            Self::THRESHOLD                             => Ok( Self::Threshold ),
            Self::SMOOTH                                => Ok( Self::Smooth ),
            Self::AVERAGE                               => Ok( Self::Average ),
            Self::POW                                   => Ok( Self::Pow ),
            Self::REC_OP_CYCLE_METRIC                   => Ok( Self::RecOpCycleMetric ),
            Self::MAX                                   => Ok( Self::Max ),
            Self::MIN                                   => Ok( Self::Min ),
            Self::PIECEWISE_LINE_APPROX                 => Ok( Self::PiecewiseLineApprox ),
            Self::IS_CHANGED_VALUE                      => Ok( Self::IsChangedValue ),
            Self::HOLD | Self::KEEP_VALID               => Ok( Self::Hold ),
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
mod retain {
mod initial_ctx {
use std::{path::{Path, PathBuf}, sync::Arc, time::Duration};
use function_name::named;
use sal_core::dbg::Dbg;
use sal_sync::services::entity::Point;
use crate::{domain::FxSccHashMap, err_pass, services::task::{RetainMode, TaskRetainConf, retain::EvalResult}};
use super::{Eval, RetainCtx};
///
/// Создает стартовый `RetainCtx` и передает его дальше по конвейеру
pub struct InitialCtx<Child> {
    txid: usize,
    conf: TaskRetainConf,
    path: PathBuf,
    child: Child,
    dbg: Dbg,
}
//
impl<Child> InitialCtx<Child> {
    pub fn new(parent: impl Into<String>, txid: usize, conf: &TaskRetainConf, path: impl AsRef<Path>, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            txid,
            conf: conf.clone(),
            path: path.as_ref().to_path_buf(),
            child,
            dbg,
        }
    }
}
//
impl<Child> Eval<Arc<FxSccHashMap<String, Point>>, EvalResult> for InitialCtx<Child>
where
    Child: Eval<RetainCtx, EvalResult>, {
    #[named]
    fn eval(&self, cache: Arc<FxSccHashMap<String, Point>>) -> EvalResult {
        let ctx = RetainCtx {
            txid: self.txid,
            path: match self.conf.mode {
                RetainMode::Debug => self.path.with_extension("json"),
                RetainMode::Release => self.path.with_extension("dat"),
            },
            cache,
            writer: None,
            file_size_bytes: 0,
            compactation_trigger: super::compactate_journal::Trigger::new(Duration::from_hours(4)).with_mb_limit(self.conf.journal.compaction_limit_mb),
            compacted: false,
            flush_trigger: super::compactate_journal::Trigger::new(self.conf.journal.flush.interval),
            error: None,
        };
        self.child.eval(ctx).map_err(|err| err_pass!(self.dbg, err))
    }
}
}
pub(super) use initial_ctx::*;
mod append_journal {
use std::{cell::Cell, collections::VecDeque, fs::File, io::{BufWriter, Write}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::kernel::state::ChangeNotify;
use serde::{Serialize, Serializer, ser::SerializeMap};
use crate::{err_pass, services::task::{RetainEvent, RetainMode, retain::RetainState}};
use super::{Eval, RetainCtx};
///
/// Current state of IO
pub(super) enum IoState {
    /// Continue using IO
    Err(Error),
    /// IO is corrupted, must be closed
    Closed(Error),
}
impl IoState {
    #[named]
    pub fn map(dbg: impl ToString, err: std::io::Error) -> Self {
        if !is_retryable(err.kind()) {
            return IoState::Closed(err_pass!(dbg, err));
        }
        IoState::Err(err_pass!(dbg, err))
    }
}
impl std::fmt::Debug for IoState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Err(err) => write!(f, "{:?}", err),
            Self::Closed(err) => write!(f, "{:?}", err),
        }
    }
}
const fn is_retryable(kind: std::io::ErrorKind) -> bool {
    matches!(
        kind,
        std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    )
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    Ok,
    Err,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BufState {
    Ok,
    Err,
}
pub struct AppendJournal<Child> {
    txid: usize,
    mode: RetainMode,
    buffer: Cell<VecDeque<RetainEvent>>,
    child: Child,
    notify: ChangeNotify<State, String>,
    buf_notify: ChangeNotify<BufState, String>,
    dbg: Dbg,
}
impl<Child> AppendJournal<Child> {
    const MAX_BUFFER_SIZE: usize = 16_000;
    pub fn new(parent: impl Into<String>, txid: usize, mode: RetainMode, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        let notify = ChangeNotify::builder(&dbg, State::Ok)
            .on(State::Ok, |msg| log::info!("{:?}", msg))
            .on(State::Err, |msg| log::warn!("{:?}", msg))
            .build();
        let buf_notify = ChangeNotify::builder(&dbg, BufState::Ok)
            .on(BufState::Ok, |msg| log::info!("{:?}", msg))
            .on(BufState::Err, |msg| log::warn!("{:?}", msg))
            .build();
        Self {
            txid,
            mode,
            buffer: Cell::new(VecDeque::new()),
            child,
            notify,
            buf_notify,
            dbg,
        }
    }
}
impl<Child> Eval<(Option<RetainEvent>, RetainCtx), RetainCtx> for AppendJournal<Child>
where
    Child: Eval<RetainCtx, RetainCtx>, {
    #[named]
    fn eval(&self, (event, mut ctx): (Option<RetainEvent>, RetainCtx)) -> RetainCtx {
        let mut buf = self.buffer.take();
        if ctx.compacted() {
            ctx.appended();
            buf.clear();
        }
        if let Some(event) = event {
            if let Err(err) = ctx.cache.insert_sync(event.key.clone(), event.p.clone()) {
                log::warn!("{}.load | Can't extend cache: {:?}", self.dbg, err);
            }
            if buf.len() >= Self::MAX_BUFFER_SIZE {
                if buf.pop_front().is_some() {
                    self.buf_notify.update(BufState::Err, || format!("{}.run | Buffer state: Overflow. Dropping oldest events to protect memory", self.dbg));
                }
            } else {
                self.buf_notify.update(BufState::Ok, || format!("{}.run | Buffer state: Normal.", self.dbg));
            }
            buf.push_back(event);
        }
        if let Some(writer) = &mut ctx.writer {
            let mut last_err = None;
            buf.retain(|event| {
                let state = RetainState::from(&event.p);
                if let Some(IoState::Closed(_)) = last_err {
                    return true;
                }
                match append(&self.dbg, writer, &self.mode, &event.key, &state) {
                    Ok(bytes) => {
                        ctx.file_size_bytes += bytes as u64;
                        false
                    }
                    Err(err) => {
                        last_err = Some(err);
                        true
                    }
                }
            });
            if let Some(err) = last_err {
                if let IoState::Closed(_) = err {
                    ctx.writer_close();
                }
                self.notify.update(State::Err, || format!("{}.run | Retain-storage I/O problems. Persistence impossible, buffering events. Reason: {:?}", self.dbg, err));
            } else {
                self.notify.update(State::Ok, || format!("{}.run | Retain storage I/O operational", self.dbg));
            }
        }
        self.buffer.set(buf);
        let mut ctx = self.child.eval(ctx);
        ctx.error = ctx.error.map(|err| err_pass!(self.dbg, err));
        ctx
    }
}
#[named]
pub(super) fn append<T: Serialize>(
    dbg: &Dbg,
    writer: &mut BufWriter<File>,
    mode: &RetainMode,
    key: &String,
    state: &T,
) -> Result<usize, IoState> {
    match mode {
        RetainMode::Debug => {
            let mut writer = CountingWriter::new(writer);
            let mut serializer = serde_json::Serializer::new(&mut writer);
            let mut map = serializer.serialize_map(Some(1))
                .map_err(|err| IoState::Err(err_pass!(dbg, err)))?;
            map.serialize_entry(key, state)
                .map_err(|err| IoState::Err(err_pass!(dbg, err)))?;
            map.end().map_err(|err| {
                if let Some(kind) = err.io_error_kind() {
                    if !is_retryable(kind) {
                        return IoState::Closed(err_pass!(dbg, err))
                    }
                }
                IoState::Err(err_pass!(dbg, err))
            })?;
            writer.write_all(b"\n").map_err(|err| IoState::map(dbg, err))?;
            Ok(writer.bytes_written())
        }
        RetainMode::Release => {
            writer.write_all(&(key.len() as u32).to_le_bytes()).map_err(|err| IoState::map(dbg, err))?;
            writer.write_all(key.as_bytes()).map_err(|err| IoState::map(dbg, err))?;
            let bytes = postcard::to_allocvec(state).map_err(|err| IoState::Err(err_pass!(dbg, err)))?;
            writer.write_all(&(bytes.len() as u32).to_le_bytes()).map_err(|err| IoState::map(dbg, err))?;
            writer.write_all(&bytes).map_err(|err| IoState::map(dbg, err))?;
            Ok(4 + key.len() + 4 + bytes.len())
        }
    }
}
struct CountingWriter<W: Write> {
    inner: W,
    bytes_written: usize,
}
impl<W: Write> CountingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, bytes_written: 0 }
    }
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }
    pub fn into_inner(self) -> W {
        self.inner
    }
}
impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let result = self.inner.write(buf);
        if let Ok(bytes) = result {
            self.bytes_written += bytes;
        }
        result
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
}
pub(super) use append_journal::*;
mod compactate_journal {
use std::{fs::File, io::BufWriter, time::{Duration, Instant}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use crate::{err_pass, services::task::{RetainMode, TaskRetainConf, retain::{RetainCtx, RetainState}}};
use super::Eval;
pub struct CompactateJournal {
    conf: TaskRetainConf,
    dbg: Dbg,
}
impl CompactateJournal {
    pub fn new(parent: impl Into<String>, conf: &TaskRetainConf) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            conf: conf.clone(),
            dbg,
        }
    }
    #[named]
    fn store(dbg: &Dbg, conf: &TaskRetainConf, ctx: &mut RetainCtx) -> Result<(), Error> {
        let (tmp_path, path) = match conf.mode {
            RetainMode::Debug => (ctx.path.with_extension("json.tmp"), ctx.path.with_extension("json")),
            RetainMode::Release => (ctx.path.with_extension("dat.tmp"), ctx.path.with_extension("dat")),
        };
        let file = File::create(&tmp_path)
            .map_err(|err| err_pass!(dbg, err, "Can't open '{}'", tmp_path.display()))?;
        let mut tmp_writer = BufWriter::with_capacity(128 * 1024, file);
        ctx.cache.iter_sync(|key, point| {
            if let Err(err) = super::append(dbg, &mut tmp_writer, &conf.mode, key, &RetainState::from(point)) {
                log::warn!("{}.store | Can't store '{}' into '{}', error: {:?}", dbg, key, tmp_path.display(), err);
            }
            true
        });
        let file = tmp_writer.into_inner().map_err(|err| err_pass!(dbg, err, "Can't flush '{}'", tmp_path.display()))?;
        file.sync_data().map_err(|err| err_pass!(dbg, err, "Can't Sync '{}'", tmp_path.display()))?;
        drop(file);
        if cfg!(target_os = "windows") {
            ctx.writer_close();
        }
        std::fs::rename(&tmp_path, &path).map_err(|err| err_pass!(dbg, err, "Can't Rename '{}' -> '{}'", tmp_path.display(), path.display()))?;
        log::trace!("{}.store | Compactation done to '{}'", dbg, path.display());
        Ok(())
    }
}
impl Eval<RetainCtx, RetainCtx> for CompactateJournal {
    fn eval(&self, mut ctx: RetainCtx) -> RetainCtx {
        if ctx.compactation_trigger.is_exceeded(ctx.file_size_bytes) {
            match Self::store(&self.dbg, &self.conf, &mut ctx) {
                Ok(_) => {
                    ctx.compactation_done();
                    ctx.writer_close();
                    ctx.compactation_trigger.start();
                }
                Err(err) => {
                    log::warn!("{}.run | Store error: {:?}", self.dbg, err);
                }
            }
        }
        ctx
    }
}
pub struct Trigger {
    interval: Duration,
    bytes_limit: u64,
    t: std::cell::Cell<Instant>,
}
impl Trigger {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            bytes_limit: 0,
            t: std::cell::Cell::new(Instant::now()),
        }
    }
    pub fn with_mb_limit(self, mb: impl Into<u64>) -> Self {
        Self {
            interval: self.interval,
            bytes_limit: mb.into() * 1024 * 1024,
            t: self.t,
        }
    }
    pub fn start(&self) {
        self.t.replace(Instant::now());
    }
    pub fn is_exceeded(&self, bytes: impl Into<u64>) -> bool {
        if self.bytes_limit > 0 {
            return self.t.get().elapsed() >= self.interval || bytes.into() >= self.bytes_limit;
        }
        self.t.get().elapsed() >= self.interval
    }
}
impl Default for Trigger {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),
            bytes_limit: 512,
            t: std::cell::Cell::new(Instant::now()),
        }
    }
}}
pub(super) use compactate_journal::*;
mod flush_journal {
use std::io::Write;
use function_name::named;
use sal_core::dbg::Dbg;
use crate::err_pass;
use super::{Eval, RetainCtx};
const fn is_retryable(kind: std::io::ErrorKind) -> bool {
    matches!(
        kind,
        std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    )
}
pub struct FlushJournal<Child> {
    child: Child,
    dbg: Dbg,
}
impl<Child> FlushJournal<Child> {
    pub fn new(parent: impl Into<String>, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            child,
            dbg,
        }
    }
}
impl<Child> Eval<RetainCtx, RetainCtx> for FlushJournal<Child>
where
    Child: Eval<RetainCtx, RetainCtx>,
{
    #[named]
    fn eval(&self, mut ctx: RetainCtx) -> RetainCtx {
        if ctx.flush_trigger.is_exceeded(0u64) {
            ctx.flush_trigger.start();
            if let Some(writer) = &mut ctx.writer {
                if let Err(err) = writer.flush() {
                    log::warn!("{}.run | Can't flush to '{:?}', error: {:?}", self.dbg, ctx.path.display(), err);
                    if !is_retryable(err.kind()) {
                        ctx.writer_close();
                    }
                }
            }
        }
        let mut ctx = self.child.eval(ctx);
        ctx.error = ctx.error.map(|err| err_pass!(self.dbg, err));
        ctx
    }
}
}
pub(super) use flush_journal::*;
mod load_journal {
use std::{collections::HashMap, fs::File, io::{BufRead, BufReader, Read}, path::Path, sync::Arc};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{entity::{Cot, Point, PointHlr}, types::Bool};
use crate::{domain::FxSccHashMap, err, err_pass, services::task::retain::{RetainState, RetainValue}};
use super::{EvalResult, Eval, RetainCtx};
enum IoState {
    Continue((String, RetainState)),
    Done,
}
pub struct LoadJournal<Child> {
    child: Child,
    dbg: Dbg,
}
impl<Child> LoadJournal<Child> {
    pub fn new(parent: impl Into<String>, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            child,
            dbg,
        }
    }
    fn point(state: &RetainState, txid: usize, name: impl Into<String>) -> Point {
        match &state.value {
            RetainValue::Bool(v) => Point::Bool(PointHlr::new(txid, name, Bool(*v), state.status, Cot::Inf, state.ts)),
            RetainValue::Int(v) => Point::Int(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::Real(v) => Point::Real(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::Double(v) => Point::Double(PointHlr::new(txid, name, *v, state.status, Cot::Inf, state.ts)),
            RetainValue::String(v) => Point::String(PointHlr::new(txid, name, v.clone(), state.status, Cot::Inf, state.ts)),
            RetainValue::Bytes(v) => Point::Bytes(PointHlr::new(txid, name, v.clone(), state.status, Cot::Inf, state.ts)),
        }
    }
    #[named]
    fn decode_entry(dbg: &Dbg, reader: &mut BufReader<File>, len_buf: &mut [u8; 4], key_buf: &mut Vec<u8>, state_buf: &mut Vec<u8>) -> Result<IoState, Error> {
        if reader.read_exact(len_buf).is_err() { return Ok(IoState::Done); }
        let len = u32::from_le_bytes(*len_buf);
        if len > 1024 {
            return Err(err!(dbg, "Размер ключа retain-записи: {} - превышает 1KB, файл кэша поврежден", len));
        }
        key_buf.resize(len as usize, 0u8);
        reader.read_exact(key_buf).map_err(|err| err_pass!(dbg, err))?;
        let key = std::str::from_utf8(key_buf).map_err(|err| err_pass!(dbg, err))?;
        reader.read_exact(len_buf).map_err(|err| err_pass!(dbg, err))?;
        let len = u32::from_le_bytes(*len_buf);
        if len > 10 * 1024 * 1024 {
            return Err(err!(dbg, "Размер значения retain-записи: {} - превышает 100MB, файл кэша поврежден", len));
        }
        state_buf.resize(len as usize, 0u8);
        reader.read_exact(state_buf).map_err(|err| err_pass!(dbg, err))?;
        let state = postcard::from_bytes::<RetainState>(state_buf).map_err(|err| err_pass!(dbg, err))?;
        Ok(IoState::Continue((key.to_owned(), state)))
    }
    fn load(&self, path: &Path, txid: usize, cache: &Arc<FxSccHashMap<String, Point>>) -> Result<(), Error> {
        let dat_path = path.with_extension("dat");
        match File::open(&dat_path) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut len_buf = [0u8; 4];
                let mut key_buf = Vec::with_capacity(1024);
                let mut state_buf = Vec::with_capacity(4096);
                loop {
                    match Self::decode_entry(&self.dbg, &mut reader, &mut len_buf, &mut key_buf, &mut state_buf) {
                        Ok(IoState::Done) => break,
                        Ok(IoState::Continue((name, state))) => {
                            let val = Self::point(&state, txid, &name);
                            if let Err(err) = cache.insert_sync(name, val) {
                                log::warn!("{}.load | Can't extend cache: {:?}", self.dbg, err);
                            }
                        }
                        Err(err) => {
                            log::error!("{}.load | Retain файл журнала оборван или поврежден '{}'.\n\tОшибка: {:?}.\n\tТолько часть данных загружено: {:#?}.",
                                self.dbg, dat_path.display(), err, cache);
                            break;
                        }
                    }
                }
                return Ok(());
            }
            Err(err) => if err.kind() != std::io::ErrorKind::NotFound {
                log::warn!("{}.load | Can't read cache '{}': {:?}", self.dbg, dat_path.display(), err);
            }
        }
        let json_path = path.with_extension("json");
        match File::open(&json_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut lines = reader.lines();
                while let Some(line) = lines.next() {
                    match line {
                        Ok(line) => {
                            match serde_json::from_str::<HashMap<String, RetainState>>(&line) {
                                Ok(parsed) => {
                                    if let Some((key, state)) = parsed.into_iter().next() {
                                        let val = Self::point(&state, txid, &key);
                                        if let Err(err) = cache.insert_sync(key, val) {
                                            log::warn!("{}.load | Can't extend cache: {:?}", self.dbg, err);
                                        }
                                    }
                                }
                                Err(err) => log::warn!("{}.load | Can't parse entry in {}, error: {:?}", self.dbg, json_path.display(), err),
                            }
                        }
                        Err(err) => log::warn!("{}.load | Can't read entry from {}, error: {:?}", self.dbg, json_path.display(), err),
                    }
                }
            }
            Err(err) => if err.kind() != std::io::ErrorKind::NotFound {
                log::warn!("{}.load | Can't read cache '{}': {:?}", self.dbg, json_path.display(), err);
            }
        }
        Ok(())
    }
}
impl<Child> Eval<RetainCtx, EvalResult> for LoadJournal<Child>
where
    Child: Eval<RetainCtx, EvalResult>, {
    #[named]
    fn eval(&self, ctx: RetainCtx) -> EvalResult {
        self.load(&ctx.path, ctx.txid, &ctx.cache).map_err(|err| err_pass!(self.dbg, err))?;
        self.child.eval(ctx).map_err(|err: Error| err_pass!(self.dbg, err))
    }
}
}
pub(super) use load_journal::*;
mod mark_old_journal {
use std::path::Path ;
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use crate::{err_pass, services::task::RetainMode };
use super::{EvalResult, Eval, RetainCtx};
pub struct MarkOldJournal {
    mode: RetainMode,
    dbg: Dbg,
}
impl MarkOldJournal {
    pub fn new(parent: impl Into<String>, mode: RetainMode) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            mode,
            dbg,
        }
    }
    #[named]
    fn mark_inactive_old(dbg: &Dbg, path: &Path, mode: RetainMode) -> Result<(), Error> {
        let src_path = match mode {
            RetainMode::Debug => path.with_extension("dat"),
            RetainMode::Release => path.with_extension("json"),
        };
        let dst_path = src_path.with_added_extension("old");
        if let Err(err) = std::fs::rename(&src_path, &dst_path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                std::fs::remove_file(&src_path).map_err(|err| err_pass!(dbg, err, "Can't rename/remove '{}'", src_path.display()))?;
            }
        }
        Ok(())
    }
}
impl Eval<RetainCtx, EvalResult> for MarkOldJournal {
    fn eval(&self, ctx: RetainCtx) -> EvalResult {
        Self::mark_inactive_old(&self.dbg, &ctx.path, self.mode)?;
        Ok(ctx)
    }
}
}
pub(super) use mark_old_journal::*;
mod open_journal {
use std::{cell::RefCell, fs::OpenOptions, io::{BufWriter, Write}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use crate::{err, err_pass, services::task::{FlushConf, RetainEvent}};
use super::{Eval, RetainCtx};
pub struct OpenJournal<Child> {
    conf: FlushConf,
    ctx: RefCell<Option<RetainCtx>>,
    child: Child,
    dbg: Dbg,
}
impl<Child> OpenJournal<Child> {
    pub fn new(parent: impl Into<String>, conf: &FlushConf, ctx: RetainCtx, child: Child) -> Self {
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        Self {
            conf: conf.clone(),
            ctx: RefCell::new(Some(ctx)),
            child,
            dbg,
        }
    }
    #[named]
    pub fn close(&self) -> Result<(), Error> {
        if let Some(ctx) = self.ctx.borrow_mut().as_mut() {
            if let Some(mut w) = ctx.writer.take() {
                w.flush().map_err(|err| err_pass!(self.dbg, err))?;
                if let Ok(file) = w.into_inner() {
                    file.sync_all().map_err(|err| err_pass!(self.dbg, err))?;
                }
            }
        }
        Ok(())
    }
}
impl<Child> Eval<Option<RetainEvent>, Result<(), Error>> for OpenJournal<Child>
where
    Child: Eval<(Option<RetainEvent>, RetainCtx), RetainCtx>, {
    #[named]
    fn eval(&self, event: Option<RetainEvent>) -> Result<(), Error> {
        match self.ctx.borrow_mut().as_mut() {
            Some(ctx) => {
                let path = ctx.path.clone();
                if ctx.writer.is_none() {
                    if let Some(parent) = std::path::Path::new(&path).parent() {
                        std::fs::create_dir_all(parent).map_err(|err| err_pass!(self.dbg, err))?;
                    }
                    let file = OpenOptions::new().append(true).create(true).open(&path).map_err(|err| err_pass!(self.dbg, err))?;
                    ctx.file_size_bytes = file.metadata()
                        .map_err(|err| err_pass!(self.dbg, err))?
                        .len();
                    ctx.writer = Some(BufWriter::with_capacity(self.conf.bytes_limit, file));
                }
            }
            None => return Err(err!(self.dbg, "Context not found")),
        }
        match self.ctx.replace(None) {
            Some(ctx) => {
                let mut ctx = self.child.eval((event, ctx));
                let result = ctx.error.take();
                self.ctx.replace(Some(ctx));
                match result {
                    Some(err) => Err(err_pass!(self.dbg, err)),
                    None => Ok(()),
                }
            }
            None => Err(err!(self.dbg, "Context not found")),
        }
    }
}
}
pub(super) use open_journal::*;
mod retain_state {
use sal_sync::services::entity::{Point, Status};
use serde::{Deserialize, Serialize};
#[derive(Clone)]
pub(crate) struct RetainEvent {
    pub key: String,
    pub p: Point,
}
impl RetainEvent {
    pub fn new(key: String, p: Point) -> Self {
        Self { key, p }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) enum RetainValue {
    Bool(bool),
    Int(i64),
    Real(f32),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
}
impl From<sal_sync::services::entity::Point> for RetainValue {
    fn from(p: sal_sync::services::entity::Point) -> Self {
        match p {
            sal_sync::services::entity::Point::Bool(p) => Self::Bool(p.value.0),
            sal_sync::services::entity::Point::Int(p) => Self::Int(p.value),
            sal_sync::services::entity::Point::Real(p) => Self::Real(p.value),
            sal_sync::services::entity::Point::Double(p) => Self::Double(p.value),
            sal_sync::services::entity::Point::String(p) => Self::String(p.value),
            sal_sync::services::entity::Point::Bytes(p) => Self::Bytes(p.value),
        }
    }
}
impl From<&sal_sync::services::entity::Point> for RetainValue {
    fn from(p: &sal_sync::services::entity::Point) -> Self {
        match p {
            sal_sync::services::entity::Point::Bool(p) => Self::Bool(p.value.0),
            sal_sync::services::entity::Point::Int(p) => Self::Int(p.value),
            sal_sync::services::entity::Point::Real(p) => Self::Real(p.value),
            sal_sync::services::entity::Point::Double(p) => Self::Double(p.value),
            sal_sync::services::entity::Point::String(p) => Self::String(p.value.clone()),
            sal_sync::services::entity::Point::Bytes(p) => Self::Bytes(p.value.clone()),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(super) struct RetainState {
    pub value: RetainValue,
    pub status: Status,
    pub ts: chrono::DateTime<chrono::Utc>,
}
impl From<&sal_sync::services::entity::Point> for RetainState {
    fn from(p: &sal_sync::services::entity::Point) -> Self {
        Self { value: RetainValue::from(p), status: p.status(), ts: p.timestamp() }
    }
}
impl From<sal_sync::services::entity::Point> for RetainState {
    fn from(p: sal_sync::services::entity::Point) -> Self {
        Self { status: p.status(), ts: p.timestamp(), value: RetainValue::from(p) }
    }
}
}
pub use retain_state::*;
mod task_retain_conf {
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::{Name, PointConf}};
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};
use crate::{err, err_pass, domain::me};
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum RetainMode {
    #[serde(alias = "debug")]
    Debug,
    #[serde(alias = "release")]
    Release,
}
impl Default for RetainMode {
    fn default() -> Self {
        Self::Release
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct JournalConf {
    pub flush: FlushConf,
    pub compaction_limit_mb: u64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FlushConf {
    pub bytes_limit: usize,
    pub interval: Duration,
}
#[derive(Debug, PartialEq, Clone)]
pub struct TaskRetainConf {
    pub name: Name,
    pub journal: JournalConf,
    pub mode: RetainMode,
}
impl TaskRetainConf {
    #[named]
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Result<TaskRetainConf, Error> {
        let me = me::<Self>();
        let parent = parent.into();
        let name = Name::new(&parent, me);
        let dbg = Dbg::new(parent, "TaskRetainConf");
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        log::trace!("{}.new | name: {:?}", dbg, name);
        let mode = conf.get("mode").map(|v: serde_yaml::Value| serde_yaml::from_value(v)).unwrap_or(Ok(RetainMode::Release))
            .map_err(|err| err_pass!(dbg, err, "'mode' - wrong config, 'release' / 'debug' expected"))?;
        log::trace!("{}.new | mode: {:#?}", dbg, mode);
        Ok(TaskRetainConf {
            name,
            journal: JournalConf {
                flush: FlushConf {
                    bytes_limit: 16 * 1024,
                    interval: Duration::from_secs(16),
                },
                compaction_limit_mb: 32,
            },
            mode,
        })
    }
    #[named]
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> Result<TaskRetainConf, Error> {
        let (key, value) = value.as_mapping().unwrap().into_iter().next()
            .ok_or_else(|| err!(Self, "Wrong or empty conf: {:#?}", value))?;
        let key = key.as_str().ok_or_else(|| err!(Self, "Wrong conf: {:#?}", value))?;
        Self::new(parent, ConfTree::new(key, value.clone()))
    }
    #[allow(unused)]
    #[named]
    pub fn read(parent: impl Into<String>, path: &str) -> Result<TaskRetainConf, Error> {
        let yaml_string = fs::read_to_string(path)
            .map_err(|err| err_pass!(Self, err, "Can't read file '{}'", path))?;
        let conf = serde_yaml::from_str(&yaml_string)
            .map_err(|err| err_pass!(Self, err, "Can't parse conf '{:?}'", yaml_string))?;
        TaskRetainConf::from_yaml(parent, &conf)
    }
    pub fn points(&self) -> Vec<PointConf> {
        vec![]
    }
}
impl Default for TaskRetainConf {
    fn default() -> Self {
        Self {
            name: Name::new("", crate::domain::me::<Self>()),
            journal: JournalConf {
                flush: FlushConf {
                    bytes_limit: 16 * 1024,
                    interval: Duration::from_secs(16),
                },
                compaction_limit_mb: 32,
            },
            mode: RetainMode::Release
        }
    }
}}
pub(crate) use task_retain_conf::*;
mod task_retain {
use std::{io::Write, path::PathBuf, sync::Arc, time::{Duration, Instant}};
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{Service, Services, entity::{Name, Object, Point, PointConf}}, sync::{Handles, Owner}, thread_pool::Scheduler};
use crate::{domain::{FxSccHashMap, RECV_TIMEOUT, Receiver, RecvTimeoutError, Sender, bounded}, err, err_pass, services::task::{AppendJournal, CompactateJournal, FlushJournal, InitialCtx, LoadJournal, MarkOldJournal, OpenJournal, TaskRetainConf, retain::{Eval, RetainEvent}}};
pub struct TaskRetain {
    txid: usize,
    name: Name,
    cache: Arc<FxSccHashMap<String, Point>>,
    conf: TaskRetainConf,
    path: PathBuf,
    send: Sender<RetainEvent>,
    recv: Owner<Receiver<RetainEvent>>,
    scheduler: Option<Scheduler>,
    handles: Handles<()>,
    exit: Arc<ExitNotify>,
    dbg: Dbg,
}
#[allow(unused)]
impl TaskRetain {
    const BUFFER_SIZE: usize = 16 * 1024;
    #[named]
    pub fn new(parent: &Name, txid: usize, conf: TaskRetainConf, services: &Arc<Services>, scheduler: Scheduler) -> Result<Self, Error> {
        let name = Name::new(parent, crate::domain::me::<Self>());
        let dbg = Dbg::new(parent, crate::domain::me::<Self>());
        let Some(retain_path) = services.retain().path else {
            return Err(err!(dbg, "Retain: path - missed in Application config"));
        };
        let cw_dir = std::env::current_dir().map_err(|err| err_pass!(dbg, err))?;
        let dir = cw_dir.join(retain_path).join(parent.join().trim_start_matches('/'));
        std::fs::create_dir_all(&dir).map_err(|err| err_pass!(dbg, err, "Error creating dir: '{}'", dir.display()))?;
        let path = dir.join("retain").with_extension("json");
        let (send, recv) = bounded(Self::BUFFER_SIZE);
        Ok(Self {
            txid,
            name,
            cache: Arc::new(FxSccHashMap::default()),
            conf,
            path,
            send,
            recv: Owner::new(recv),
            scheduler: Some(scheduler),
            handles: Handles::new(parent),
            exit: Arc::new(ExitNotify::new(parent, None, None)),
            dbg,
        })
    }
    pub fn mock(parent: impl Into<String>, cache: impl IntoIterator<Item = (String, Point)>) -> Self {
        let name = Name::new(parent, crate::domain::me::<Self>());
        let dbg = Dbg::new(name.parent(), crate::domain::me::<Self>());
        let (send, recv) = bounded(Self::BUFFER_SIZE);
        Self {
            txid: 0,
            name,
            cache: Arc::new(cache.into_iter().collect()),
            conf: TaskRetainConf::default(),
            path: PathBuf::new(),
            send,
            recv: Owner::new(recv),
            scheduler: None,
            handles: Handles::new(&dbg),
            exit: Arc::new(ExitNotify::new(&dbg, None, None)),
            dbg,
        }
    }
    pub fn link(&self) -> Sender<RetainEvent> {
        self.send.clone()
    }
    pub fn get(&self, key: &str) -> Option<Point> {
        self.cache.read_sync(key, |_, p| p.clone())
    }
}
impl Object for TaskRetain {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
impl std::fmt::Debug for TaskRetain {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskRetain")
            .field("name", &self.name)
            .finish()
    }
}
//
impl Service for TaskRetain {
    //
    // fn get_link(&self, _: &str) -> Sender<Point> {
    //     self.send.clone()
    // }
    //
    #[named]
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let rx_recv = self.recv.take().ok_or_else(|| err!(dbg, "Can't take recv"))?;
        let ctx = InitialCtx::new(&dbg, self.txid, &conf, &self.path,
            LoadJournal::new(&dbg,
                MarkOldJournal::new(&dbg, conf.mode),
            ),
        ).eval(self.cache.clone())?;
        match self.scheduler.as_ref() {
            Some(scheduler) => {
                let handle = scheduler.spawn({
                    let dbg = dbg.clone();
                    let retain = OpenJournal::new(&dbg, &conf.journal.flush, ctx,
                        AppendJournal::new(&dbg, self.txid, conf.mode,
                            FlushJournal::new(&dbg,
                                CompactateJournal::new(&dbg, &conf),
                            ),
                        ),
                    );
                    move || {
                    'main: while !exit.get() {
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {  // 100ms
                            Ok(event) => {
                                log::trace!("{dbg}.run | point '{}': {:?}", event.key, event.p);
                                if let Err(err) = retain.eval(Some(event)).map_err(|err| err_pass!(dbg, err)) {
                                    log::warn!("{err}");
                                }
                            }
                            Err(RecvTimeoutError::Timeout) => {
                                if let Err(err) = retain.eval(None).map_err(|err| err_pass!(dbg, err)) {
                                    log::warn!("{err}");
                                }
                            }
                            Err(err) => {
                                log::error!("{dbg}.run | Receiv error: {:?}", err);
                                break 'main;
                            }
                        }
                    }
                    if let Err(err) = retain.close() {
                        log::error!("{dbg}.run | Can't close retain journal: {:?}", err);
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
                    Err(err) => Err(err_pass!(dbg, err, "Start failed")),
                }
            }
            None => {
                let handle = std::thread::spawn({
                    let dbg = dbg.clone();
                    let cache = self.cache.clone();
                    move || {
                    'main: while !exit.get() {
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {
                            Ok(event) => {
                                log::trace!("{dbg}.run | point '{}': {:?}", event.key, event.p);
                                if let Err(err) = cache.insert_sync(event.key, event.p) {
                                    log::error!("{dbg}.run | Can't update retain cache: {:?}", err);
                                }
                            }
                            Err(RecvTimeoutError::Timeout) => {}
                            Err(err) => {
                                log::error!("{dbg}.run | Receiv error: {:?}", err);
                                break 'main;
                            }
                        };
                    }
                    log::info!("{dbg}.run | Exit");
                }});
                self.handles.push(handle);
                log::info!("{dbg}.run | Starting (Mock) - ok");
                Ok(())
            }
        }
    }
    //
    fn points(&self) -> Vec<PointConf> {
        self.conf.points()
    }
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    fn exit(&self) {
        self.exit.exit();
    }
}
///
/// Cycle measuring
#[allow(unused)]
struct Trigger {
    interval: Duration,
    bytes_limit: u64,
    t: std::cell::Cell<Instant>,
}
//
#[allow(unused)]
impl Trigger {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            bytes_limit: 0,
            t: std::cell::Cell::new(Instant::now()),
        }
    }
    ///
    /// Maximum buffer length allowed before exceeded, MB.
    pub fn with_mb_limit(self, mb: impl Into<u64>) -> Self {
        Self {
            interval: self.interval,
            bytes_limit: mb.into() * 1024 * 1024,
            t: self.t,
        }
    }
    pub fn start(&self) {
        self.t.replace(Instant::now());
    }
    ///
    /// ### Returns `true` if time interval or bytes limit is exceeded
    /// - `bytes`: Current size in bytes
    pub fn is_exceeded(&self, bytes: impl Into<u64>) -> bool {
        if self.t.get().elapsed() >= self.interval {
            return true;
        }
        if self.bytes_limit > 0 {
            return bytes.into() >= self.bytes_limit;
        }
        false
    }
}
///
/// Wraps a writer and counts the total number of bytes written.
#[allow(unused)]
struct CountingWriter<W: Write> {
    inner: W,
    bytes_written: usize,
}
//
#[allow(unused)]
impl<W: Write> CountingWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner, bytes_written: 0 }
    }
    ///
    /// Получить текущее значение счетчика
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }
    pub fn into_inner(self) -> W {
        self.inner
    }
}
//
impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.inner.write(buf) {
            Ok(bytes) => {
                self.bytes_written += bytes;
                Ok(bytes)
            }
            Err(err) => Err(err),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
}
pub use task_retain::*;
use crate::err_pass;
use function_name::named;
pub(self) type EvalResult = Result<RetainCtx, sal_core::error::Error>;
///
/// `TaskRetain` evaluation
pub(self) trait Eval<In, Out> {
    fn eval(&self, _: In) -> Out;
}
///
/// Context provides tranfer data in the `TaskRetain` evaluation
pub(super) struct RetainCtx {
    pub txid: usize,
    pub cache: std::sync::Arc<crate::domain::FxSccHashMap<String, sal_sync::services::entity::Point>>,
    pub path: std::path::PathBuf,
    pub writer: Option<std::io::BufWriter<std::fs::File>>,
    pub file_size_bytes: u64,
    pub compactation_trigger: compactate_journal::Trigger,
    /// Весь retain cache только что был записан надиск, необходимо очистить буфер в `AppendJournal`
    pub compacted: bool,
    pub flush_trigger: compactate_journal::Trigger,
    pub error: Option<sal_core::error::Error>,
}
//
impl RetainCtx {
    /// Отмечаем что компактация успешно выполнена
    pub fn compactation_done(&mut self) {
        self.compacted = true;
    }
    /// Проверяем была ли компактация
    pub fn compacted(&self) -> bool {
        self.compacted
    }
    pub fn appended(&mut self) {
        self.compacted = false;
    }
    #[named]
    pub fn writer_close(&mut self) {
        if let Some(w) = self.writer.take() {
            if let Err(err) = w.into_inner().map_err(|err| err_pass!(Self, err, "Can't close writer")) {
                log::warn!("{err}");
            }
        }
    }
}
}
pub(super) use retain::*;
mod task_conf {
use function_name::named;
use indexmap::IndexMap;
use sal_core::error::Error;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::{Name, PointConf}, ConfSubscribe, task::functions::{FnConfKind, FnConfig}};
use std::{fs, time::Duration};
use crate::{err, err_pass, services::task::TaskRetainConf};
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
    pub retain: TaskRetainConf,
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
    #[named]
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Result<TaskConf, Error> {
        let mut vars = vec![];
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("TaskConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, self_name);
        let cycle = conf.get_duration("cycle").ok();
        log::trace!("{}.new | cycle: {:?}", dbg, cycle);
        let retain: TaskRetainConf = conf.get("retain")
            .map(|conf: ConfTree| TaskRetainConf::new(&self_name, conf)).transpose()
            .map_err(|err| err_pass!(dbg, err, "'retain' - wrong config"))?
            .unwrap_or_default();
        log::trace!("{}.new | retain: {:?}", dbg, retain);
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
        Ok(TaskConf {
            name: self_name,
            cycle,
            retain,
            rx,
            rx_max_length,
            subscribe,
            nodes,
            vars,
        })
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    #[named]
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> Result<TaskConf, Error> {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                Err(err!(Self, "Wrong or empty conf: {:#?}", value))
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    #[named]
    pub fn read(parent: impl Into<String>, path: &str) -> Result<TaskConf, Error> {
        let yaml_str = fs::read_to_string(path).map_err(|err| err_pass!(Self, err, "Can't read file '{}'", path))?;
        let conf = serde_yaml::from_str(&yaml_str).map_err(|err| err_pass!(Self, err, "Can't parse file '{}'", path))?;
        TaskConf::from_yaml(parent, &conf)
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
use function_name::named;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ExitNotify, services::{
    ConfSubscribe, Service, ServiceCycle, Services, SubscriptionCriteria, entity::{Name, Object, Point, PointConf, PointTxId}
}, sync::{Handles, Owner, channel::{self, Receiver, RecvTimeoutError, Sender}}, thread_pool::Scheduler};
use std::{
    collections::HashMap, fmt::Debug, sync::Arc, time::Duration
};
use concat_string::concat_string;
use crate::{
    domain::RECV_TIMEOUT, err, err_pass, services::task::{TaskRetain, task_conf::TaskConf, task_nodes::TaskNodes}, sync::SendWrapper
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
//
impl Service for Task {
    //
    //
    fn get_link(&self, name: &str) -> Sender<Point> {
        // match self.in_send.get(name) {
        match self.in_send.iter().next() {
            Some((_, send)) => send.clone(),
            None => {
                log::error!("{}.get_link | link '{}' - not found", self.dbg, name);
                panic!("{}.get_link | Error", self.dbg);
            }
        }
    }
    //
    #[named]
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        log::trace!("{}.run | Self tx_id: {}", self.dbg, PointTxId::from_str(&self.name.join()));
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let conf_cycle = conf.cycle;
        let services = self.services.clone();
        let txid = PointTxId::from_str(&self_name.join());
        let retain = Arc::new(TaskRetain::new(&self_name, txid, conf.retain.clone(), &services, self.scheduler.clone())
            .map_err(|err| err_pass!(dbg, err))?);
        let task_nodes = {
            let mut task_nodes = TaskNodes::new(&dbg, txid, retain.clone());
            task_nodes.build_nodes(&self_name, &conf, services.clone())
                .map_err(|err| Error::new(&dbg, "run").pass(err))?;
            SendWrapper::wrap(task_nodes)
        };
        let subscriptions = self.subscriptions_(&conf.subscribe, &services);
        let rx_recv = self.subscribe_(&subscriptions, &services);
        retain.run().map_err(|err| err_pass!(dbg, err))?;
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
            retain.exit();
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
    services::task::{EvalCycle, EvalCycleRef, FnEnableMode, FnEvalOnce, TaskRetain, functions::{FnBuilder, FnKind}, task_conf::TaskConf},
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
    txid: usize,
    retain: Arc<TaskRetain>,
    nodes: IndexMap<String, Rc<RefCell<TaskEvalNode>>>,
    vars: IndexMap<String, FnOutRef>,
    new_node_vars: Option<TaskNodeVars>,
    /// Текущий номер вычислительного цикла, инкремнтируется с каждым входом в `self.eval`
    cycle: EvalCycleRef,
    /// Enable Strategy: Cold Standby / Warm Standby (TODO: read from config)
    enable_mode: FnEnableMode,
    dbg: String,
}
//
//
impl TaskNodes {
    ///
    /// Returns `TaskNodes` new instance
    pub fn new(parent: impl Into<String>, txid: usize, retain: Arc<TaskRetain>,) ->Self {
        Self {
            txid,
            retain,
            nodes: IndexMap::new(),
            vars: IndexMap::new(),
            new_node_vars: None,
            cycle: Rc::new(EvalCycle::new()),
            enable_mode: FnEnableMode::Cold,
            dbg: format!("{}/TaskNodes", parent.into()),
        }
    }
    ///
    /// Returns `TaskNodes` new instance
    pub fn without_retain(parent: impl Into<String>, txid: usize) ->Self {
        let dbg = format!("{}/TaskNodes", parent.into());
        Self {
            txid,
            retain: TaskRetain::mock(&dbg, []).into(),
            nodes: IndexMap::new(),
            vars: IndexMap::new(),
            new_node_vars: None,
            cycle: Rc::new(EvalCycle::new()),
            enable_mode: FnEnableMode::Cold,
            dbg,
        }
    }
    ///
    /// Returns `txid` of the parent `Task`
    pub fn txid(&self) -> usize {
        self.txid
    }
    ///
    /// Returns `retain` service of the parent `Task`
    pub fn retain(&self) -> Arc<TaskRetain> {
        self.retain.clone()
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
    pub fn cycle(&self) -> EvalCycleRef {
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
    // ///
    // /// Returns input by it's name
    // pub fn get_input(&self, name: &str) -> Option<FnInOutRef> {
    //     self.inputs.get(name).map(|node| node.get_input())
    // }
    ///
    /// Returns variable by it's name
    pub fn get_var(&self, name: &str) -> Option<&FnOutRef> {
        log::trace!("{}.getVar | trying to find variable {:?} in {:?}", self.dbg, &name, self.vars);
        self.vars.get(name)
    }
    ///
    /// Adding new input reference
    pub fn add_input(&mut self, name: impl Into<String>, input: FnInOutRef) -> Result<FnOutRef, Error> {
        let name = name.into();
        match self.new_node_vars {
            Some(_) => {
                match self.nodes.get_mut(&name) {
                    // Same name - adding to the existing node, if input has different 'options hash'
                    Some(node) => {
                        log::trace!("{}.add_input | input {:?}:{} - adding to the existing node if has different 'options hash'", self.dbg, name, input.borrow().hash());
                        Ok(node.borrow_mut().add_input(input))
                    }
                    // New name - adding new TaskEvalNode
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
    ///
    /// Adding new variable refeerence
    pub fn add_var(&mut self, name: impl Into<String>, var: FnOutRef) -> Result<(), Error> {
        let name = name.into();
        // assert!(!self.vars.contains_key(name.as_str()), "Dublicated variable name: {:?}", name);
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
    ///
    /// Adding already declared variable as out to the newNodeStuff
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
    ///
    /// Call this method to finish configuration of jast created task node
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
    ///
    /// Creates all task nodes depending on it config
    ///  - if Task config contains 'point [type] every' then single evaluation node allowed only
    pub fn build_nodes(&mut self, parent: &Name, conf: &TaskConf, services: Arc<Services>) -> Result<(), Error>{
        // TODO: Добавить проверку на ацикличность направленного графа (DAG). Например, алгоритм поиска в глубину (DFS) по связям inputs, проверяющий, не возвращаемся ли мы в уже посещенный узел.
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
                        FnBuilder::new(parent, &mut node_conf, self, services.clone())
                            .map_err(|err| error.pass_with(format!("Can't build eval node '{node_name}': {:?}", conf), err))?,
                    )))
                }
                FnConfKind::Var(_) => {
                    Rc::new(RefCell::new(FnEvalOnce::new(parent, self.cycle.clone(),
                    FnBuilder::new(parent, &mut node_conf, self, services.clone())
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
        // if let Some(eval_node) = self.get_eval_node("every") {
        //     let eval_node_name = eval_node.name();
        //     for (_name, input) in &self.nodes {
        //         let len = input.get_outs().len();
        //         if len > 1 {
        //             return Err(error.err(format!("evalNode '{}' - contains {} Out's, but single Out allowed when 'point [type] every' was used", eval_node_name, len)));
        //         }
        //     }
        // }
        Ok(())
    }
    ///
    /// Evaluates all containing node:
    ///  - adding new point
    ///  - evaluating each node
    pub fn eval(&self, point: Point) {
        let dbg = self.dbg.clone();
        self.cycle.increment();
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
            // Some(eval_node) => {
            // }
            // None => {}
                // log::warn!("{dbg}.eval | evalNode '{}' - not fount, input point ignored", point_name);
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
// use std::collections::HashMap;
// use crate::core_::FnInOutRef;
///
/// A container for storing variable names
/// during configuring single TaskEvalNode only
#[derive(Debug)]
pub struct TaskNodeVars {
    vars: Vec<String>,
}
impl TaskNodeVars {
    ///
    /// Creates new container for storing variable & input names
    /// during configuring single TaskEvalNode only
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
        }
    }
    ///
    /// Adding new variable name
    pub fn add_var(&mut self, name: impl Into<String> + Clone) -> Result<(), Error> {
        let name = name.into();
        // assert!(!self.vars.contains(&name), "Dublicated variable name: {:?}", name);
        if name.is_empty() {
            return Err(Error::new("TaskNodeVars", "add_var").err("Variable name can't be emty"));
        }
        log::trace!("TaskNodeStuff.addVar | adding variable {:?}", name);
        self.vars.push(name);
        Ok(())
    }
    // ///
    // ///
    // fn names(collection: &HashMap<String, FnInOutRef>) -> Vec<String> {
    //     collection.keys().cloned().collect()
    // }
    ///
    /// Returns all collected var names
    pub fn get_vars(&self) -> Vec<String> {
        self.vars.clone()
    }
}
}
mod task_eval_node {
use sal_core::dbg::Dbg;
use sal_sync::services::entity::Point;
use crate::{domain::{FnInOutRef, FnOutRef}, services::task::FnResult};
///
/// Holds Task input and all dipendent variables & outputs
#[derive(Debug)]
pub struct TaskEvalNode {
    name: String,
    input: Vec<FnInOutRef>,
    vars: Vec<FnOutRef>,
    outs: Vec<FnOutRef>,
    dbg: Dbg,
}
//
//
impl TaskEvalNode {
    ///
    /// Creates new instance from input name, input it self and dependent vars & outs
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
    ///
    /// Adds input if it's has different 'Options hash'
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
pub(crate) use eval_cycle::*;
pub(super) use fn_eval_once::*;
pub use functions::*;
pub use task_conf::*;
pub use task::*;
pub use task_nodes::*;
pub use task_node_vars::*;
pub use task_eval_node::*;
pub use task_test_receiver::*;
pub use task_test_producer::*;
