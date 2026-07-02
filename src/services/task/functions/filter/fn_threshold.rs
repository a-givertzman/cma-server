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
    fn hard_reset(&mut self) {
        self.threshold.hard_reset();
        if let Some(factor) = &mut self.factor {
            factor.hard_reset();
        }
        self.input.hard_reset();
        self.filter.reset();
    }
    //
    fn reset(&mut self) {
        self.filter.reset();
    }
}
///
/// Global static counter of FnThreshold instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
