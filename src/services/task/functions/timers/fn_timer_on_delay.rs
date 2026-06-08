use sal_core::error::Error;
use concat_string::concat_string;
use sal_sync::services::{conf::ConfDuration, entity::{Point, PointHlr}, types::Bool};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
use crate::{
    domain::{Edge, EdgeDetector, FnOutRef},
    services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult, TryTo},
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
    edge: EdgeDetector,
    reset_edge: EdgeDetector,
    value: EdgeDetector,
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
            edge: EdgeDetector::new(),
            reset_edge: EdgeDetector::new(),
            value: EdgeDetector::new(),
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
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let reset = self.reset.as_mut().map(|f| f.out());
        let input = self.input.out();
        let flow = FlowContext::new();
        if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if let Some(Edge::Rising) = self.reset_edge.add(reset) {
                    self.active_t = None;
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
                self.active_t = Some(Instant::now());
                false
            }
            Some(Edge::Falling) => {
                self.active_t = None;
                false
            }
            None => {
                if let Some(t) = self.active_t {
                    t.elapsed() >= self.delay
                } else {
                    self.active_t = None;
                    false
                }
            }
        };
        let is_changed = self.value.add(value).is_some();
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
        self.edge.reset();
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
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use std::thread::sleep;
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    use crate::services::task::{FnFlow, FnKind, FnOut, FnResult};
    use sal_sync::services::conf::{ConfDuration, ConfDurationUnit};
    use sal_sync::services::types::Bool;
    #[derive(Debug)]
    struct MockOrigin {
        next_flow: Option<FnFlow>,
    }
    impl MockOrigin {
        fn new() -> Self { Self { next_flow: None } }
        fn push(&mut self, val: bool) {
            self.next_flow = Some(FnFlow::New(Point::Bool(PointHlr::new(
                0, "test_in", Bool(val), Status::Ok, Cot::Inf, chrono::Utc::now()
            ))));
        }
        fn push_old(&mut self, val: bool) {
            self.next_flow = Some(FnFlow::Old(Point::Bool(PointHlr::new(
                0, "test_in", Bool(val), Status::Ok, Cot::Inf, chrono::Utc::now()
            ))));
        }
    }
    impl FnOut for MockOrigin {
        fn id(&self) -> String { "mock".to_string() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.next_flow.clone()) }
        fn reset(&mut self) {}
    }
    #[test]
    fn test_zero_delay_emits_true_on_next_tick() {
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", None, ConfDuration::new(0, ConfDurationUnit::Millis), input.clone());
        input.borrow_mut().push(true);
        let res1 = ton.out().unwrap().unwrap();
        assert!(matches!(res1, FnFlow::New(Point::Bool(p)) if p.value.0 == false));
        input.borrow_mut().push_old(true);
        let res2 = ton.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::New(Point::Bool(p)) if p.value.0 == true));
    }
    #[test]
    fn test_normal_delay_and_falling_edge() {
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", None, ConfDuration::new(50, ConfDurationUnit::Millis), input.clone());
        input.borrow_mut().push(true);
        let _ = ton.out().unwrap();
        input.borrow_mut().push_old(true);
        let res_wait = ton.out().unwrap().unwrap();
        assert!(matches!(res_wait, FnFlow::Old(Point::Bool(p)) if p.value.0 == false));
        sleep(Duration::from_millis(55));
        let res_done = ton.out().unwrap().unwrap();
        assert!(matches!(res_done, FnFlow::New(Point::Bool(p)) if p.value.0 == true));
        input.borrow_mut().push(false);
        let res_fall = ton.out().unwrap().unwrap();
        assert!(matches!(res_fall, FnFlow::New(Point::Bool(p)) if p.value.0 == false));
    }
    #[test]
    fn test_reset_interrupts_timer() {
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let reset = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", Some(reset.clone()), ConfDuration::new(100, ConfDurationUnit::Millis), input.clone());
        input.borrow_mut().push(true);
        reset.borrow_mut().push(false);
        let _ = ton.out().unwrap();
        sleep(Duration::from_millis(20));
        reset.borrow_mut().push(true);
        input.borrow_mut().push_old(true);
        let res_interrupted = ton.out().unwrap().unwrap();
        assert!(matches!(res_interrupted, FnFlow::Old(Point::Bool(p)) if p.value.0 == false));
        sleep(Duration::from_millis(100));
        input.borrow_mut().push_old(true);
        let res_after_time = ton.out().unwrap().unwrap();
        assert!(matches!(res_after_time, FnFlow::Old(Point::Bool(p)) if p.value.0 == false));
    }
}
