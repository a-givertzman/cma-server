use function_name::named;
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointType};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use concat_string::concat_string;
use crate::domain::{FnOutRef, TryTo};
use crate::err_pass;
use crate::services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult};

/// ### Function | `FnAverage` (Time-Weighted Average)
/// 
/// Вычисляет взвешенное по времени среднее (Time-Weighted Average) входного сигнала.
///
/// Особенности работы:
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `reset`: Сбрасывает накопленную сумму и таймеры если `> 0`.
/// - `input`: Источник числовых данных. Выходной `Point` автоматически наследует 
///   тип данных входа (Bool, Int, Real или Double).
/// - Игнорирует нечисловые типы (возвращает `Err`).
#[derive(Debug)]
pub struct FnAverage {
    id: String,
    kind: FnKind,
    reset: Option<FnOutRef>,
    input: FnOutRef,
    sum: f64,
    average: Option<Point>,
    total_t: f64,
    prev: f64,
    last_t: Option<Instant>,
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
            reset,
            input,
            sum: 0.0,
            average: None,
            total_t: 0.0,
            prev: 0.0,
            last_t: None,
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
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = self.input.borrow_mut().out();
        let reset = self.reset.as_mut().map(|f| f.borrow_mut().out());
        let Some(input) = flow.map(input)? else {
            self.last_t = None;
            return Ok(None)
        };
        // let mut force_recalc = false;
        if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if reset {
                    self.total_t = 0.0;
                    self.last_t = None;
                    self.prev = 0.0;
                    self.sum = 0.0;
                    // force_recalc = true;
                }
            }
        }
        // Закоментировано потому что из двух вариантов реализации среднего: "Событийный" и "Взвешенный по времени"
        // более подходящим и универсальным является "Взвешенный по времени", поэтому пока оставляю его.
        // В будущем можно добавить отдельно событийный вариант FnEventAverage, который будет считать только FlowNew.
        // // Возвращаем предыдущее значение, если нет новых данных на входе и не было сброса
        // if !flow.is_new() && !force_recalc {
        //     let Some(average) = self.average.as_ref() else { return Ok(None) };
        //     return flow.wrap_old(average.clone());
        // }
        // trace!("{}.out | input: {:?}", self.id, input);
        let value = input.try_to().map_err(|err: Error| err_pass!(self.id, err, "Invalid input type {:?}", input.typ()).to_string())?;
        let now = Instant::now();
        let dt = self.last_t.map_or(0.0, |last| now.duration_since(last).as_secs_f64());
        self.last_t = Some(now);
        self.total_t += dt;
        self.sum += self.prev * dt;
        self.prev = value;
        let average = if self.total_t > 0.0 { self.sum / self.total_t } else { value };
        // log::debug!("{}.out | sum: {:?}", self.id, self.sum);
        // log::debug!("{}.out | count: {:?}", self.id, self.count);
        // log::debug!("{}.out | average: {:?}", self.id, average);
        let (average, point) = match input.typ() {
            PointType::Int => {
                let av = average.round();
                (av, Point::Int(Self::point_with(&input, &self.id, av as i64)))
            }
            PointType::Real => (average, Point::Real(Self::point_with(&input, &self.id, average as f32))),
            PointType::Double => (average, Point::Double(Self::point_with(&input, &self.id, average))),
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        let is_changed = self.average.as_ref().map_or(true, |prev| {
            (prev.to_double().as_double().value - average).abs() > 1e-12 * average.abs() ||
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
        self.last_t = None;
        self.total_t = 0.0;
        self.prev = 0.0;
        self.sum = 0.0;
        self.average = None;
        if let Some(reset) = &mut self.reset {
            reset.borrow_mut().hard_reset();
        }
        self.input.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {
        self.last_t = None;
        self.total_t = 0.0;
        self.prev = 0.0;
        self.sum = 0.0;
        self.average = None;
    }
}
///
/// Global static counter of FnAverage instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
