use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointType};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::domain::{FnOutRef, TryTo};
use crate::services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult};

/// ### Function | FnAverage (Time-Weighted Average)
/// 
/// Вычисляет взвешенное по времени среднее (Time-Weighted Average) входного сигнала.
///
/// Особенности работы:
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `reset`: Сбрасывает накопленную сумму и счетчик если `> 0`.
/// - `input`: Источник числовых данных. Выходной `Point` автоматически наследует 
///   тип данных входа (Int, Real или Double).
/// - Игнорирует нечисловые типы (возвращает `Err`).
#[derive(Debug)]
pub struct FnAverage {
    id: String,
    kind: FnKind,
    reset: Option<FnOutRef>,
    input: FnOutRef,
    count: u64,
    sum: f64,
    average: Option<Point>,
}
//
// 
impl FnAverage {
    ///
    /// Creates new instance of the FnAverage
    /// * `parent` - Идентификатор родительского узла
    /// * `reset` - Входной сигнал для сброса накопителя (опциональный).
    /// * `input` - Входной числовой сигнал для расчета среднего.
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnAverage{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            reset: reset,
            input: input,
            count: 0,
            sum: 0.0,
            average: None,
        }
    }
    ///
    /// Возвращает `Point` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
}
//
// 
impl FnOut for FnAverage {
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
        if let Some(reset) = self.reset.as_ref() {
            inputs.append(&mut reset.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = self.input.borrow_mut().out();
        let reset = self.reset.as_mut().map(|f| f.borrow_mut().out());
        let Some(input) = flow.map(input)? else { return Ok(None) };
        if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if reset {
                    self.count = 0;
                    self.sum = 0.0;
                }
            }
        }
        // trace!("{}.out | input: {:?}", self.id, input);
        let value = match input.typ() {
            PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        self.sum += value;
        self.count += 1;
        let average = self.sum / (self.count as f64);
        // log::debug!("{}.out | sum: {:?}", self.id, self.sum);
        // log::debug!("{}.out | count: {:?}", self.id, self.count);
        // log::debug!("{}.out | average: {:?}", self.id, average);
        let point = match input.typ() {
            PointType::Int => Point::Int(Self::point_with(&input, &self.id, average.round() as i64)),
            PointType::Real => Point::Real(Self::point_with(&input, &self.id, average as f32)),
            PointType::Double => Point::Double(Self::point_with(&input, &self.id, average)),
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        let is_changed = self.average.as_ref().map_or(true, |prev| {
            (prev.to_double().as_double().value - point.to_double().as_double().value).abs() > f64::EPSILON ||
            prev.status() != input.status()
        });
        self.average = Some(point.clone());
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn hard_reset(&mut self) {
        self.count = 0;
        self.sum = 0.0;
        self.average = None;
        if let Some(reset) = &mut self.reset {
            reset.borrow_mut().hard_reset();
        }
        self.input.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {
        self.count = 0;
        self.sum = 0.0;
        self.average = None;
    }
}
///
/// Global static counter of FnAverage instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
