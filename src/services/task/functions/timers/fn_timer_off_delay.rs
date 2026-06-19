use function_name::named;
use sal_core::error::Error;
use concat_string::concat_string;
use sal_sync::services::{conf::ConfDuration, entity::{Point, PointHlr}, types::Bool};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
use crate::{
    domain::{FnOutRef, TryTo}, err_pass, services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | `FnTimerOffDelay`
/// 
/// Таймер задержки выключения TOF (Timer-Off-Delay)
/// 
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `reset`: Доминантный сбрас. При `true` обнуляет выход в `false` и сбрасывает секундомер.
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
    trigg: Option<bool>,
    state: Option<bool>,
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
            trigg: None,
            state: None,
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
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let reset = self.reset.as_mut().map(|f| f.out());
        let input = self.input.out();
        let flow = FlowContext::new();
        let reset = if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if reset {
                    self.active_t = None;
                    self.trigg = None;
                }
                reset
            } else { false }
        } else { false };
        let Some(input) = flow.ignore(input)? else {
            self.active_t = None;
            self.trigg = None;
            self.state = None;
            return Ok(None);
        };
        let is_active: bool = (&input).try_to().map_err(|err: Error| err_pass!(self.id, err, "Invalid input").to_string())? && !reset;
        log::trace!("{} | Input: {}", self.id, is_active);
        let value = match (self.trigg, is_active) {
            (_, true) => true,
            (None, false) => false,
            (Some(true), false) => {
                if self.delay.is_zero() {
                    false
                } else {
                    let t = *self.active_t.get_or_insert_with(Instant::now);
                    t.elapsed() <= self.delay
                }
            }
            (Some(false), false) => {
                if let Some(t) = self.active_t {
                    t.elapsed() <= self.delay
                } else {
                    false
                }
            }
        };
        if !value {
            self.active_t = None;
        }
        self.trigg = Some(is_active);
        let is_changed = match (self.state, value) {
            (None, _) => true,
            (Some(false), true) => true,
            (Some(true), false) => true,
            _ => false,
        };
        self.state = Some(value);
        if is_changed {
            self.ts = chrono::Utc::now();
        }
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value), self.ts));
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.active_t = None;
        self.trigg = None;
        self.state = None;
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
    use debugging::session::debug_session::{DebugSession, LogLevel};
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
    fn test_zero_delay_follows_input() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOffDelay-zero_delay";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut tof = FnTimerOffDelay::new("test", None, ConfDuration::new(0, ConfDurationUnit::Millis), input.clone());
        // Шаг 1: Первая подача True на вход. Выход мгновенно повторяет вход
        input.borrow_mut().push(true);
        let res1 = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 1: input true | state: {:?}", res1);
        assert!(matches!(res1, FnFlow::New(Point::Bool(ref p)) if p.value.0 == true), "\n result: {:?}\n target: New(Point::Bool(true))", res1);
        // Шаг 2: Входной сигнал неизменен. На выходе Old(True)
        input.borrow_mut().push_old(true);
        let res2 = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 2: input old true | state: {:?}", res2);
        assert!(matches!(res2, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == true), "\n result: {:?}\n target: Old(Point::Bool(true))", res2);
        // Шаг 3: Спад входного сигнала в False. Так как задержка нулевая, выход мгновенно падает
        input.borrow_mut().push(false);
        let res3 = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 3: input false | state: {:?}", res3);
        assert!(matches!(res3, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: New(Point::Bool(false))", res3);
    }
    #[test]
    fn test_normal_delay_holds_true_on_falling_edge() {
        DebugSession::new().filter(LogLevel::Debug).init();
        let dbg = "FnTimerOffDelay-normal_delay";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut tof = FnTimerOffDelay::new("test", None, ConfDuration::new(50, ConfDurationUnit::Millis), input.clone());
        // Шаг 1: Подача True на вход. Сигнал проходит мгновенно.
        input.borrow_mut().push(true);
        let res_up = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 1: input true | state: {:?}", res_up);
        assert!(matches!(res_up, FnFlow::New(Point::Bool(ref p)) if p.value.0 == true), "\n result: {:?}\n target: New(Point::Bool(true))", res_up);
        // Шаг 2: Срез входного сигнала в False. Внутренний секундомер пошел. Выход удерживает True через Old!
        input.borrow_mut().push(false);
        let res_fall = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 2: input false | state: {:?}", res_fall);
        assert!(matches!(res_fall, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == true), "\n result: {:?}\n target: Old(Point::Bool(true))", res_fall);
        // Шаг 3: Выжидаем паузу в 55мс, перекрывая уставку. На выходе рождается False
        sleep(Duration::from_millis(55));
        input.borrow_mut().push_old(false);
        let res_done = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 3: after sleep | state: {:?}", res_done);
        assert!(matches!(res_done, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: New(Point::Bool(false))", res_done);
    }
    #[test]
    fn test_reset_interrupts_off_delay() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOffDelay-reset_interrupt";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let reset = Rc::new(RefCell::new(MockOrigin::new()));
        let mut tof = FnTimerOffDelay::new("test", Some(reset.clone()), ConfDuration::new(100, ConfDurationUnit::Millis), input.clone());
        // Шаг 1: Инициализация таймера (пропускаем true на выход)
        input.borrow_mut().push(true);
        reset.borrow_mut().push(false);
        let res_init = tof.out().unwrap();
        log::debug!("{dbg} | step 1: init | state: {:?}", res_init);
        // Шаг 2: Спад сигнала, таймер пошел отсчитывать 100мс
        input.borrow_mut().push(false);
        let res_delay = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 2: delay started | state: {:?}", res_delay);
        assert!(matches!(res_delay, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == true));
        // Шаг 3: Взвод линии Reset. Выход обязан упасть в false до истечения таймера
        sleep(Duration::from_millis(20));
        reset.borrow_mut().push(true);
        input.borrow_mut().push_old(false);
        let res_interrupted = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 3: reset high | state: {:?}", res_interrupted);
        assert!(matches!(res_interrupted, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: New(Point::Bool(false))", res_interrupted);
    }
    #[test]
    fn test_initialization_emits_old_on_false() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOffDelay-init";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut tof = FnTimerOffDelay::new("test", None, ConfDuration::new(10, ConfDurationUnit::Secs), input.clone());
        // Шаг 1: На первом такте мы подаем false. На выходе New(false)
        input.borrow_mut().push(false);
        let res1 = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 1: input false | state: {:?}", res1);
        assert!(matches!(res1, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: Old(Point::Bool(false))", res1);
        // Шаг 2: При неизменном false узел все еще возвращает Old(false)
        input.borrow_mut().push_old(false);
        let res2 = tof.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: Old(Point::Bool(false))", res2);
    }
    #[test]
    fn test_dominant_reset_blocks_zero_delay() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOffDelay-dominant_reset_zero_delay";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let reset = Rc::new(RefCell::new(MockOrigin::new()));
        let mut tof = FnTimerOffDelay::new("test", Some(reset.clone()), ConfDuration::new(0, ConfDurationUnit::Millis), input.clone());
        // Шаг 1: Зажимаем жесткий сброс и подаем активный вход
        reset.borrow_mut().push(true);
        input.borrow_mut().push(true);
        let res1 = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 1: reset=true, input=true | state: {:?}", res1);
        assert!(matches!(res1, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "Сброс не сработал \n result: {:?}\n target: Old(Point::Bool(false))", res1);
        // Шаг 2: Отпускаем сброс. Вход зажат. Сигнал проходит, New(true)
        reset.borrow_mut().push(false);
        input.borrow_mut().push_old(true);
        let res2 = tof.out().unwrap().unwrap();
        log::debug!("{dbg} | step 2: reset=false, input=true | state: {:?}", res2);
        assert!(matches!(res2, FnFlow::New(Point::Bool(ref p)) if p.value.0 == true), "Таймер не восстановил работу после снятия сброса");
    }
    #[test]
    fn test_cold_mode_clears_active_timer() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOffDelay-cold_mode";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut tof = FnTimerOffDelay::new("test", None, ConfDuration::new(1, ConfDurationUnit::Secs), input.clone());
        // Шаг 1: Взводим таймер
        input.borrow_mut().push(true);
        let _ = tof.out().unwrap();
        assert!(tof.active_t.is_none(), "Таймер TOF не должен запуститься");
        input.borrow_mut().push(false);
        let _ = tof.out().unwrap();
        assert!(tof.active_t.is_some(), "Таймер TOF должен запуститься");
        // Шаг 2: Обрыв связи (Ok(None))
        input.borrow_mut().next_flow = None;
        let res_cold = tof.out().unwrap();
        assert!(res_cold.is_none(), "Узел обязан вернуть None в режиме Cold Mode");
        assert!(tof.active_t.is_none(), "Внутренний таймер должен быть сброшен");
        assert!(tof.state.is_none(), "Кэш состояния должен быть уничтожен");
    }
    #[test]
    fn test_accident_simultaneous_reset_and_input() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOffDelay-accident";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let reset = Rc::new(RefCell::new(MockOrigin::new()));
        let mut tof = FnTimerOffDelay::new("test", Some(reset.clone()), ConfDuration::new(200, ConfDurationUnit::Millis), input.clone());
        // 1. Аварийный старт: одновременно рабочий сигнал и сброс
        input.borrow_mut().push(true);
        reset.borrow_mut().push(true);
        let res1 = tof.out().unwrap().unwrap();
        assert!(matches!(res1, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "При холодном старте выход должен быть New(false)");
        // assert!(tof.active_t.is_none(), "Секундомер не должен запускаться в фоне");
        // 2. Имитация простоя: система спит (100мс)
        sleep(Duration::from_millis(100));
        // 3. Отпускаем сброс. input все еще true. TOF мгновенно выдает New(true)
        reset.borrow_mut().push(false);
        input.borrow_mut().push_old(true);
        let res3 = tof.out().unwrap().unwrap();
        // assert!(tof.active_t.is_none(), "Секундомер не должен запускаться в фоне");
        assert!(matches!(res3, FnFlow::New(Point::Bool(ref p)) if p.value.0 == true), "Таймер должен выдать New(true) при снятии сброса на активном входе");
        // 4. Опускаем вход. TOF выдает Old(true) и запускает таймер
        input.borrow_mut().push(false);
        let res4 = tof.out().unwrap().unwrap();
        assert!(tof.active_t.is_some(), "Внутренний секундомер пошел только сейчас");
        assert!(matches!(res4, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == true), "Таймер должен выдать true при сбросе входа");
        // 5. Ждем честную уставку (210мс)
        sleep(Duration::from_millis(210));
        input.borrow_mut().push_old(false);
        let res3 = tof.out().unwrap().unwrap();
        assert!(matches!(res3, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "Таймер не отработал штатно после снятия входа и установленного времени \n result: {:?}\n target: New(Point::Bool(false))", res3);
    }
}
