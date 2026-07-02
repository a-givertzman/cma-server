use function_name::named;
use sal_core::error::Error;
use concat_string::concat_string;
use sal_sync::services::{conf::ConfDuration, entity::{Point, PointHlr}, types::Bool};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
use crate::{
    domain::{FnOutRef, Level, LevelTrigger, TryTo}, err_pass, services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult}
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
    trigg: LevelTrigger,
    state: Option<bool>,
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
            trigg: LevelTrigger::new(),
            state: None,
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
                    self.trigg.reset();
                }
                reset
            } else { false }
        } else { false };
        let Some(input) = flow.ignore(input)? else {
            self.active_t = None;
            self.trigg.reset();
            self.state = None;
            return Ok(None);
        };
        let is_active: bool = (&input).try_to().map_err(|err: Error| err_pass!(self.id, err, "Invalid input").to_string())? && !reset;
        log::trace!("{} | Input: {}", self.id, is_active);
        let value = match self.trigg.add(is_active) {
            Some(Level::Up) => {
                let t = Instant::now();
                self.active_t = Some(t);
                let val = t.elapsed() >= self.delay;
                // log::debug!("{} | State: Level::Up | {:?} (Go)", self.id, self.active_t.unwrap().elapsed());
                val
            }
            Some(Level::Down) => {
                // log::debug!("{} | State: Level::Down | 0 ms (Stop)", self.id);
                self.active_t = None;
                false
            }
            _ => {
                if let Some(t) = self.active_t {
                    let val = t.elapsed() >= self.delay;
                    // log::debug!("{} | State: None::IsActive: {val} | {:?} (Go)", self.id, self.active_t.unwrap().elapsed());
                    val
                } else {
                    // log::debug!("{} | State: None::NotActive: false | 0 ms (Stop)", self.id);
                    self.active_t = None;
                    false
                }
            }
        };
        let is_changed = match (self.state, value) {
            (None, _) => true,
            (Some(false), true) => true,
            (Some(true), false) => true,
            _ => false,
        };
        self.state = Some(value);
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value), chrono::Utc::now()));
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn hard_reset(&mut self) {
        self.active_t = None;
        self.trigg.reset();
        self.state = None;
        if let Some(reset) = &mut self.reset {
            reset.hard_reset();
        }
        self.input.hard_reset();
    }
    //
    fn reset(&mut self) {
        self.active_t = None;
        self.trigg.reset();
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
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }
    #[test]
    fn test_zero_delay_emits_true_on_first_tick() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOnDelay-zero_delay";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", None, ConfDuration::new(0, ConfDurationUnit::Millis), input.clone());
        // Шаг 1: Первая подача True на вход. Узел фиксирует передний фронт, возвращает New(true)
        input.borrow_mut().push(true);
        let res1 = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 1: input true | state: {:?}", res1);
        assert!(matches!(res1, FnFlow::New(Point::Bool(ref p)) if p.value.0 == true), "\n result: {:?}\n target: New(Point::Bool(true))", res1);
        // Шаг 2: Входной сигнал остается неизменным. Так как задержка нулевая, на этом такте генерирует New(True)
        input.borrow_mut().push_old(true);
        let res2 = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 2: input old true | state: {:?}", res2);
        assert!(matches!(res2, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == true), "\n result: {:?}\n target: New(Point::Bool(true))", res2);
    }
    #[test]
    fn test_normal_delay_and_falling_edge() {
        DebugSession::new().filter(LogLevel::Debug).init();
        let dbg = "FnTimerOnDelay-normal_delay";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", None, ConfDuration::new(50, ConfDurationUnit::Millis), input.clone());
        // Шаг 1: Подача True на вход. Внутренний секундомер пошел, но время задержки (50мс) еще не вышло, на выходе Old(False)
        input.borrow_mut().push(true);
        let res_first = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 1: input true | state: {:?}", res_first);
        assert!(matches!(res_first, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: Old(Point::Bool(false))", res_first);
        // Шаг 2: Повторный опрос графа при неизменном входе. Таймер продолжает считать, выход Old(False)
        input.borrow_mut().push_old(true);
        let res_wait = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 2: input old true | state: {:?}", res_wait);
        assert!(matches!(res_wait, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: Old(Point::Bool(false))", res_wait);
        // Шаг 3: Выжидаем паузу в 55мс, полностью перекрывая уставку таймера. На выходе должно родиться событие True
        sleep(Duration::from_millis(55));
        let res_done = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 3: after sleep | state: {:?}", res_done);
        assert!(matches!(res_done, FnFlow::New(Point::Bool(ref p)) if p.value.0 == true), "\n result: {:?}\n target: New(Point::Bool(true))", res_done);
        // Шаг 4: Срез входного сигнала в False. Таймер обязан отреагировать мгновенно и сбросить выходное значение
        input.borrow_mut().push(false);
        let res_fall = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 4: input false | state: {:?}", res_fall);
        assert!(matches!(res_fall, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: New(Point::Bool(false))", res_fall);
    }
    #[test]
    fn test_reset_interrupts_timer() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOnDelay-reset_interrupt";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let reset = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", Some(reset.clone()), ConfDuration::new(100, ConfDurationUnit::Millis), input.clone());
        // Шаг 1: Инициализация таймера подачей True при неактивном сбросе
        input.borrow_mut().push(true);
        reset.borrow_mut().push(false);
        let res_init = ton.out().unwrap();
        log::debug!("{dbg} | step 1: init | state: {:?}", res_init);
        // Шаг 2: Взвод линии Reset через 20мс. Накопленное время обязано обнулиться, выходной сигнал принудительно заблокирован
        sleep(Duration::from_millis(20));
        reset.borrow_mut().push(true);
        input.borrow_mut().push_old(true);
        let res_interrupted = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 2: reset high | state: {:?}", res_interrupted);
        assert!(matches!(res_interrupted, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: Old(Point::Bool(false))", res_interrupted);
        // Шаг 3: Спим еще 100мс при зажатом сбросе. Проверяем, что даже по истечении полной уставки выход не переключился в True
        sleep(Duration::from_millis(100));
        input.borrow_mut().push_old(true);
        let res_after_time = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 3: after full delay | state: {:?}", res_after_time);
        assert!(matches!(res_after_time, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: Old(Point::Bool(false))", res_after_time);
    }
    #[test]
    fn test_initialization_emits_new() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOnDelay-init";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", None, ConfDuration::new(10, ConfDurationUnit::Secs), input.clone());
        // Шаг 1: На первом такте мы подаем false. Так как состояние не изменилось, выход Old(false)
        input.borrow_mut().push(false);
        let res1 = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 1: input false | state: {:?}", res1);
        assert!(matches!(res1, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: Old(Point::Bool(false))", res1);
        // Шаг 2: При неизменном false узел все еще возвращает Old(false)
        input.borrow_mut().push_old(false);
        let res2 = ton.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == false), "\n result: {:?}\n target: Old(Point::Bool(false))", res2);
    }
    #[test]
    fn test_dominant_reset_blocks_zero_delay() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOnDelay-dominant_reset_zero_delay";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let reset = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", Some(reset.clone()), ConfDuration::new(0, ConfDurationUnit::Millis), input.clone());
        // Шаг 1: Зажимаем жесткий сброс и подаем активный вход.
        // Даже при нулевой задержке сброс обязан перехватить управление
        reset.borrow_mut().push(true);
        input.borrow_mut().push(true);
        let res1 = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 1: reset=true, input=true | state: {:?}", res1);
        assert!(matches!(res1, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "Сброс не сработал \n result: {:?}\n target: New(Point::Bool(false))", res1);
        // Шаг 2: Отпускаем сброс. Вход по-прежнему зажат.
        // Таймер должен воспринять это как свежий старт и пропустить сигнал (так как delay=0)
        reset.borrow_mut().push(false);
        input.borrow_mut().push_old(true);
        let res2 = ton.out().unwrap().unwrap();
        log::debug!("{dbg} | step 2: reset=false, input=true | state: {:?}", res2);
        assert!(matches!(res2, FnFlow::New(Point::Bool(ref p)) if p.value.0 == true), "Таймер не восстановил работу после снятия сброса");
    }
    #[test]
    fn test_cold_mode_clears_state() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOnDelay-cold_mode";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", None, ConfDuration::new(1, ConfDurationUnit::Secs), input.clone());
        // Шаг 1: Взводим таймер
        input.borrow_mut().push(true);
        let _ = ton.out().unwrap();
        assert!(ton.active_t.is_some(), "Таймер должен запуститься");
        // Шаг 2: Обрыв связи или выключение через FnEnable (input отдает Ok(None))
        input.borrow_mut().next_flow = None;
        let res_cold = ton.out().unwrap();
        assert!(res_cold.is_none(), "Узел обязан вернуть None в режиме Cold Mode");
        assert!(ton.active_t.is_none(), "Внутренний таймер должен быть сброшен");
        assert!(ton.state.is_none(), "Кэш состояния должен быть уничтожен");
    }
    #[test]
    fn test_accident_simultaneous_reset_and_input() {
        DebugSession::new().filter(LogLevel::Info).init();
        let dbg = "FnTimerOnDelay-accident";
        log::info!("{dbg}");
        let input = Rc::new(RefCell::new(MockOrigin::new()));
        let reset = Rc::new(RefCell::new(MockOrigin::new()));
        let mut ton = FnTimerOnDelay::new("test", Some(reset.clone()), ConfDuration::new(200, ConfDurationUnit::Millis), input.clone());
        // 1. Аварийный старт: одновременно приходит рабочий сигнал и сигнал сброса
        input.borrow_mut().push(true);
        reset.borrow_mut().push(true);
        let res1 = ton.out().unwrap().unwrap();
        assert!(matches!(res1, FnFlow::New(Point::Bool(ref p)) if p.value.0 == false), "Выход должен быть заблокирован");
        assert!(ton.active_t.is_none(), "Секундомер не должен запускаться в фоне");
        // 2. Имитация простоя: сигналы висят, система спит (100мс)
        sleep(Duration::from_millis(100));
        // 3. Отпускаем сброс. input все еще true.
        // Триггер обязан увидеть это как чистый передний фронт (Level::Up) и только сейчас запустить таймер
        reset.borrow_mut().push(false);
        input.borrow_mut().push_old(true);
        let res2 = ton.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::Old(Point::Bool(ref p)) if p.value.0 == false), "Таймер не должен выдать true мгновенно!");
        assert!(ton.active_t.is_some(), "Внутренний секундомер пошел только сейчас");
        // 4. Ждем честную уставку (210мс)
        sleep(Duration::from_millis(210));
        input.borrow_mut().push_old(true);
        let res3 = ton.out().unwrap().unwrap();
        assert!(matches!(res3, FnFlow::New(Point::Bool(ref p)) if p.value.0 == true), "Таймер отработал штатно после снятия сброса");
    }
}
