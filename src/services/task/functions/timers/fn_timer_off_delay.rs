use sal_core::error::Error;
use concat_string::concat_string;
use sal_sync::services::{conf::ConfDuration, entity::{Point, PointHlr}, types::Bool};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
use crate::{
    domain::{Edge, EdgeDetector, FnOutRef},
    services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult, TryTo},
};
///
/// ### Function | `FnTimerOffDelay`
/// 
/// Таймер задержки выключения TOF (Timer-Off-Delay)
/// 
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `reset`: Сбрасывает секундомер и выход по переднему фронту сигнала (переход 0 -> 1).
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
    edge: EdgeDetector,
    reset_edge: EdgeDetector,
    value: EdgeDetector,
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
            edge: EdgeDetector::new(),
            reset_edge: EdgeDetector::new(),
            value: EdgeDetector::new(),
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
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let reset = self.reset.as_mut().map(|f| f.out());
        let input = self.input.out();
        let flow = FlowContext::new();
        if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if let Some(Edge::Rising) = self.reset_edge.add(reset) {
                    self.active_t = None;
                    self.ts = chrono::Utc::now();
                    self.edge.reset();
                }
            };
        }
        let Some(input) = flow.ignore(input)? else {
            self.active_t = None;
            self.edge.reset();
            self.value.reset();
            return Ok(None);
        };
        let is_active: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let value = match self.edge.add(is_active) {
            Some(Edge::Rising) => {
                self.ts = chrono::Utc::now();
                true
            }
            Some(Edge::Falling) => {
                self.active_t = Some(Instant::now());
                true
            }
            None => {
                if let Some(t) = self.active_t {
                    let remains = t.elapsed() <= self.delay;
                    if !remains {
                        self.active_t = None;
                        self.ts = chrono::Utc::now();
                    }
                    remains
                } else {
                    self.active_t = None;
                    false
                }
            }
        };
        let is_changed = self.value.add(value).is_some();
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value), self.ts));
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.ts = chrono::Utc::now();
        self.active_t = None;
        self.edge.reset();
        self.reset_edge.reset();
        self.value.reset();
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.input.reset();
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
