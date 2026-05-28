use sal_sync::services::entity::{Point, PointConfType, PointHlr};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::domain::FnOutRef;
use crate::services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult};

/// ### Function | Average
/// 
/// Вычисляет накопительное среднее значение (Cumulative Average) входного сигнала.
///
/// Особенности работы:
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) принудительно сбрасывает накопленную сумму 
///   и счетчик итераций, прерывая передачу данных (возвращает `None`).
/// - `input`: Источник числовых данных. Выходной `Point` автоматически наследует 
///   тип данных входа (Int, Real или Double).
/// - Игнорирует нечисловые типы (возвращает `Err`).
#[derive(Debug)]
pub struct FnAverage {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    count: i64,
    sum: f64,
    average: Option<Point>,
}
//
// 
impl FnAverage {
    ///
    /// Creates new instance of the FnAverage
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnAverage{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            input,
            count: 0,
            sum: 0.0,
            average: None,
        }
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
        self.input.borrow().inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        if !flow.is_new() {
            let Some(average) = self.average.as_ref() else { return Ok(None) };
            return flow.wrap(average.clone());
        }
        // trace!("{}.out | input: {:?}", self.id, input);
        let value = input.to_double().as_double().value;
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
        let average = match input.type_() {
            PointConfType::Int => {
                Point::Int(
                    PointHlr::new(
                        input.txid(),
                        &self.id,
                        average.round() as i64,
                        input.status(),
                        input.cot(),
                        input.timestamp(),
                    )
                )
            }
            PointConfType::Real => {
                Point::Real(
                    PointHlr::new(
                        input.txid(),
                        &self.id,
                        average as f32,
                        input.status(),
                        input.cot(),
                        input.timestamp(),
                    )
                )
            }
            PointConfType::Double => {
                Point::Double(
                    PointHlr::new(
                        input.txid(),
                        &self.id,
                        average,
                        input.status(),
                        input.cot(),
                        input.timestamp(),
                    )
                )
            }
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        };
        self.average = Some(average.clone());
        flow.wrap(average)
    }
    //
    fn reset(&mut self) {
        self.count = 0;
        self.sum = 0.0;
        self.average = None;
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnAverage instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
