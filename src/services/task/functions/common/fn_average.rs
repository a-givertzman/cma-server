use sal_sync::services::entity::{Point, PointHlr, PointType};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::domain::{Edge, EdgeDetector, FnOutRef};
use crate::services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult};

/// ### Function | Average
/// 
/// Вычисляет накопительное среднее значение (Cumulative Average) входного сигнала.
///
/// Особенности работы:
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `reset`: Сбрасывает накопленную сумму и счетчик по переднему фронту сигнала (переход 0 -> 1).
/// - `input`: Источник числовых данных. Выходной `Point` автоматически наследует 
///   тип данных входа (Int, Real или Double).
/// - Игнорирует нечисловые типы (возвращает `Err`).
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
            reset: reset.map(FnChange::new),
            input: FnChange::new(input),
            count: 0,
            sum: 0.0,
            average: None,
            reset_edge: EdgeDetector::new(),
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
        let mut inputs = self.input.inputs();
        if let Some(reset) = self.reset.as_ref() {
            inputs.append(&mut reset.inputs());
        }
        inputs
    }
    //
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
        // Возвращаем предыдущее значение, если нет новых данных на входе и не было сброса
        if !flow.is_new() && !force_recalc {
            let Some(average) = self.average.as_ref() else { return Ok(None) };
            return flow.wrap_old(average.clone());
        }
        // trace!("{}.out | input: {:?}", self.id, input);
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
    //
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
///
/// Global static counter of FnAverage instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
