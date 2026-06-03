use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use sal_sync::services::{entity::{Point, PointConfType, PointHlr, PointType}, types::Bool};
use crate::{
    domain::{FnOutRef, filter::{filter::Filter, filter_threshold::FilterThreshold}}, services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    }
};
///
/// Function | Returns filtered input:
/// - if [enable] is specified and true, or not specified (default true)
/// - if [factor] is not specified:
///     - new input value returned if |prev - [input]| > [threshold]
/// - if [factor] is specified:
///     - each cycle: delta = |prev - [input]| * factor
///     - new input value returned if delta >= [threshold]
/// 
/// Example
/// 
/// ```yaml
/// fn Threshold:
///     enable: const bool true         # optional, default true
///     threshold: const real 0.5   # absolute threshold if [factor] is not specified
///     factor: 0.1                 # optional, use for integral threshold
///     input: point real '/App/Service/Point.Name'
/// ```
#[derive(Debug)]
pub struct FnThreshold {
    id: String,
    kind: FnKind,
    threshold: FnOutRef,
    factor: Option<FnOutRef>,
    input: FnOutRef,
    value: Option<f64>,
    filter: Option<FilterThreshold<f64>>,
    delta: f64,
}
//
// 
impl FnThreshold {
    ///
    /// Creates new instance of the FnThreshold
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, threshold: FnOutRef, factor: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnThreshold{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            threshold,
            factor,
            input,
            value: None,
            filter: None,
            delta: 0.0,
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.timestamp())
    }
    ///
    /// Возвращает `Point` с обновленными `name` и `value` сохраняя тип
    #[inline]
    fn point(id: &str, input: &Point, val: f64) -> Result<Point, String> {
        match input.type_() {
            PointType::Bool => Ok(Point::Bool(Self::point_with(input, id, Bool(val != 0.0)))),
            PointType::Int => Ok(Point::Int(Self::point_with(input, id, val.round() as i64))),
            PointType::Real => Ok(Point::Real(Self::point_with(input, id, val as f32))),
            PointType::Double => Ok(Point::Double(Self::point_with(input, id, val))),
            _ => Err(concat_string!(id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        }
    }
}
//
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
        let mut inputs = vec![];
        inputs.append(&mut self.threshold.borrow().inputs());
        if let Some(factor) = &self.factor {
            inputs.append(&mut factor.borrow().inputs());
        }
        inputs.append(&mut self.input.borrow().inputs());
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(threshold) = flow.ignore(self.threshold.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | threshold: {:?}", self.id, threshold);
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        let value = match input.type_() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        };
        log::trace!("{}.out | input: {:?}", self.id, value);
        if self.filter.is_none() {
            let threshold = match threshold.type_() {
                PointType::Bool | PointType::Int | PointType::Real | PointType::Double => threshold.to_double().as_double().value,
                _ => return Err(concat_string!(self.id, ".out | Invalid threshold type '", threshold.type_().to_string(), "'")),
            };
            let factor = match &self.factor {
                Some(factor) => {
                    let Some(factor) = flow.ignore(factor.borrow_mut().out())? else { return Ok(None) };
                    log::trace!("{}.out | factor: {:?}", self.id, factor);
                    match factor.type_() {
                        PointType::Bool | PointType::Int | PointType::Real | PointType::Double => factor.to_double().as_double().value,
                        _ => return Err(concat_string!(self.id, ".out | Invalid factor type '", factor.type_().to_string(), "'")),
                    }
                }
                None => 0.0,
            };
            self.filter = Some(FilterThreshold::new(None, threshold, factor));
        }
        let Some(filter) = self.filter else { return Ok(None) };
        let value = match filter.add(value) {
            Some(v) => v,
            None => filter.last() ,
        };
        let value = Self::point(self.id, &input, self.value)
        log::trace!("{}.out | value: {:?}", self.id, value);
        flow.wrap(value)
    }
    //
    //
    fn reset(&mut self) {
        self.threshold.borrow_mut().reset();
        if let Some(factor) = &self.factor {
            factor.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
        self.value = None;
        self.factor = None;
        self.delta = 0.0;
    }
}
///
/// Global static counter of FnThreshold instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
