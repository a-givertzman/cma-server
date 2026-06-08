use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr};
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
    active_t: Option<Instant>,
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
            reset_edge: EdgeDetector::new(),
            first,
            total_t: 0.0,
            active_t: None,
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
                    self.active_t = None;
                    is_changed = true;
                }
            };
        }
        let Some(input) = flow.ignore(input)? else {
            if let Some(t) = self.active_t {
                self.total_t = self.total_t + t.elapsed().as_secs_f64();
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
                    is_changed = true;
                    self.first = false;
                }
            }
        }
        // trace!("{}.out | input: {:?}", self.id, self.input.print());
        let is_active: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let elapsed = match self.edge.add(is_active) {
            Some(Edge::Rising) => {
                self.active_t = Some(Instant::now());
                self.total_t
            }
            None => {
                if let Some(t) = self.active_t {
                    is_changed = true;
                    self.total_t + t.elapsed().as_secs_f64()
                } else {
                    self.total_t
                }
            }
            Some(Edge::Falling) => {
                if let Some(t) = self.active_t {
                    self.total_t = self.total_t + t.elapsed().as_secs_f64();
                }
                is_changed = true;
                self.active_t = None;
                self.total_t
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc, thread::sleep, time::Duration};
    use crate::services::task::{FnFlow, FnKind, FnOut, FnResult};
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    #[derive(Debug)]
    struct FakeNode {
        id: String,
        flow: Option<FnFlow>,
    }
    impl FakeNode {
        fn new(id: &str) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self {
                id: id.to_string(),
                flow: None,
            }))
        }
        fn push_bool(&mut self, val: bool) {
            self.flow = Some(FnFlow::New(Point::Bool(PointHlr::new(
                0, &self.id, val, Status::Ok, Cot::Inf, chrono::Utc::now(),
            ))));
        }
        fn push_double(&mut self, val: f64) {
            self.flow = Some(FnFlow::New(Point::Double(PointHlr::new(
                0, &self.id, val, Status::Ok, Cot::Inf, chrono::Utc::now(),
            ))));
        }
        fn push_none(&mut self) {
            self.flow = None;
        }
    }
    impl FnOut for FakeNode {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            Ok(self.flow.clone())
        }
        fn reset(&mut self) {}
    }
    fn extract_val(flow: &FnFlow) -> f64 {
        match flow {
            FnFlow::New(p) | FnFlow::Old(p) => p.value().as_double().unwrap_or(0.0),
        }
    }
    #[test]
    fn test_lifecycle_start_pause_reset() {
        let input = FakeNode::new("input");
        let reset = FakeNode::new("reset");
        let mut timer = FnTimer::new("test", None, Some(reset.clone()), input.clone());
        // 1. Исходное состояние: спим
        input.borrow_mut().push_bool(false);
        reset.borrow_mut().push_bool(false);
        let res1 = timer.out().unwrap().unwrap();
        assert!(matches!(res1, FnFlow::Old(_)), "В режиме сна должен возвращать Old");
        assert_eq!(extract_val(&res1), 0.0);
        // 2. Запуск (Передний фронт)
        input.borrow_mut().push_bool(true);
        let res2 = timer.out().unwrap().unwrap();
        // Внимание: Здесь зафиксировано то самое спорное поведение. Старт генерирует New
        assert!(matches!(res2, FnFlow::New(_)), "Старт таймера должен генерировать событие New");
        assert_eq!(extract_val(&res2), 0.0);
        // 3. Активный счет (пауза для накопления времени)
        sleep(Duration::from_millis(15));
        let res3 = timer.out().unwrap().unwrap();
        assert!(matches!(res3, FnFlow::Old(_)), "При неизменном активном входе генерируется Old");
        let active_val = extract_val(&res3);
        assert!(active_val > 0.01, "Таймер должен накапливать время");
        // 4. Пауза (Задний фронт)
        input.borrow_mut().push_bool(false);
        let res4 = timer.out().unwrap().unwrap();
        assert!(matches!(res4, FnFlow::New(_)), "Остановка таймера обязана генерировать New");
        let paused_val = extract_val(&res4);
        assert!(paused_val >= active_val, "Накопленное время не должно уменьшаться");
        // 5. Удержание паузы
        let res5 = timer.out().unwrap().unwrap();
        assert!(matches!(res5, FnFlow::Old(_)), "При удержании паузы генерируется Old");
        assert_eq!(extract_val(&res5), paused_val, "Время в паузе должно быть заморожено");
        // 6. Сброс
        reset.borrow_mut().push_bool(true);
        let res6 = timer.out().unwrap().unwrap();
        assert!(matches!(res6, FnFlow::New(_)), "Сброс таймера обязан генерировать New");
        assert_eq!(extract_val(&res6), 0.0, "Значение после сброса должно быть 0.0");
    }
    #[test]
    fn test_transition_active_to_none_and_back() {
        let input = FakeNode::new("input");
        let mut timer = FnTimer::new("test", None, None, input.clone());
        // 1. Старт
        input.borrow_mut().push_bool(true);
        let _ = timer.out().unwrap();
        sleep(Duration::from_millis(15));
        // 2. Обрыв связи (вход вернул None)
        input.borrow_mut().push_none();
        let res_none = timer.out().unwrap();
        assert!(res_none.is_none(), "Если вход None, таймер обязан вернуть None (прозрачность)");
        // 3. Восстановление связи
        input.borrow_mut().push_bool(true);
        let res_restore = timer.out().unwrap().unwrap();
        assert!(matches!(res_restore, FnFlow::New(_)), "Восстановление активности должно читаться как передний фронт (New)");
        let restored_val = extract_val(&res_restore);
        assert!(restored_val > 0.01, "Таймер должен был сохранить время, прошедшее до обрыва связи");
    }
    #[test]
    fn test_transition_sleep_to_none_and_back() {
        let input = FakeNode::new("input");
        let mut timer = FnTimer::new("test", None, None, input.clone());
        // 1. Усыпляем
        input.borrow_mut().push_bool(false);
        let _ = timer.out().unwrap();
        // 2. Обрыв связи
        input.borrow_mut().push_none();
        let res_none = timer.out().unwrap();
        assert!(res_none.is_none());
        // 3. Возврат связи в состоянии сна
        input.borrow_mut().push_bool(false);
        let res_restore = timer.out().unwrap().unwrap();
        assert!(matches!(res_restore, FnFlow::Old(_)), "Возврат спящего сигнала не должен генерировать событий");
        assert_eq!(extract_val(&res_restore), 0.0);
    }
    #[test]
    fn test_initial_value_applied_once() {
        let input = FakeNode::new("input");
        let initial = FakeNode::new("initial");
        let mut timer = FnTimer::new("test", Some(initial.clone()), None, input.clone());
        initial.borrow_mut().push_double(100.0);
        input.borrow_mut().push_bool(false);
        // 1. Первый такт: initial должен примениться
        let res1 = timer.out().unwrap().unwrap();
        assert_eq!(extract_val(&res1), 100.0, "Начальное значение должно быть применено");
        // 2. Второй такт: меняем initial, но таймер уже не должен его читать
        initial.borrow_mut().push_double(500.0);
        let res2 = timer.out().unwrap().unwrap();
        assert_eq!(extract_val(&res2), 100.0, "Начальное значение применяется строго один раз");
    }
}
