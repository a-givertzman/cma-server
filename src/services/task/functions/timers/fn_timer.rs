use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointType};
use concat_string::concat_string;
use std::{sync::atomic::{AtomicUsize, Ordering}, time::Instant};
use crate::{domain::{Edge, EdgeDetector, FnOutRef}, services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult, TryTo}};
///
/// Function | FnTimer
/// 
/// Интегратор времени (накопительный секундомер / моточасы).
/// Считает время в секундах, пока на входе `true` (> 0).
/// 
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `initial`: Начальное значение. Применяется строго один раз при первом успешном чтении.
/// - `reset`: Сбрасывает накопленную сумму и счетчик по переднему фронту сигнала (переход 0 -> 1).
/// - `input`: `true` - секундомер тикает (отдает `FnFlow::New`),
/// `false` - замирает и хранит значение, отдает его в Flow::Old,
/// снова `true` - счет продолжается с точки остановки.
#[derive(Debug)]
pub struct FnTimer {
    id: String,
    kind: FnKind,
    initial: Option<FnChange>,
    reset: Option<FnChange>,
    input: FnChange,
    edge: EdgeDetector,
    reset_edge: EdgeDetector,
    first: bool,
    total_t: f64,
    t: Instant,
}
// 
impl FnTimer {
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, initial: Option<FnOutRef>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnTimer{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            initial: initial.map(FnChange::new),
            reset: reset.map(FnChange::new),
            input: FnChange::new(input),
            edge: EdgeDetector::new(),
            reset_edge: EdgeDetector::new(),
            first: true,
            total_t: 0.0,
            t: Instant::now(),
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.timestamp())
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
        if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if let Some(Edge::Rising) = self.reset_edge.add(reset) {
                    self.edge.reset();
                    self.total_t = 0.0;
                    self.t = Instant::now();
                }
            };
        }
        let Some(input) = flow.ignore(input)? else {
            self.total_t = self.total_t + self.t.elapsed().as_secs_f64();
            self.edge.reset();
            return Ok(None);
        };
        if self.first {
            if let Some(initial) = initial {
                let Some(initial) = flow.ignore(initial)? else { return Ok(None) };
                let initial: f64 = (&initial).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid initial ", err.to_string()))?;
                self.total_t += initial;
                is_changed = true;
                self.first = false;
            }
        }
        // trace!("{}.out | input: {:?}", self.id, self.input.print());
        let is_active = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let elapsed = match self.edge.add(is_active) {
            Some(Edge::Rising) => {
                self.t = Instant::now();
                self.total_t
            }
            Some(Edge::Falling) => {
                self.total_t = self.total_t + self.t.elapsed().as_secs_f64();
                is_changed = true;
                self.total_t
            }
            None => {
                if is_active {
                    is_changed = true;
                    self.total_t + self.t.elapsed().as_secs_f64()
                } else {
                    self.total_t
                }
            }
        };
        log::trace!("{}.out | elapsed: {:?}", self.id, self.total_t);
        let point = Point::Double(Self::point_with(&input, &self.id, elapsed));
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
