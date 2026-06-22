use indexmap::IndexMap;
use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::{Point, PointType}, sync::channel::Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::{Edge, EdgeDetector, FnOutRef}, services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult}};
///
/// ### Function | Creates SQL requests on [op-cycle] falling edge:
/// - Operating cycle SQL request (id, start, stop)
/// - Operating cycle metrics SQL requests (cycle_id, pid, metric_id, value)
/// - Returns `op-cycle` input if all inputs are Ok
/// - **Note:** `enable` input logic is implemented via `FnEnable` decorator.
/// 
/// ### Example
/// 
/// ```yaml
/// ```
#[derive(Debug)]
pub struct FnRecOpCycleMetric {
    id: String,
    kind: FnKind,
    send_to: Option<Sender<Point>>,
    reset: Option<FnChange>,
    op_cycle: FnChange,
    inputs: FxIndexMap<String, FnChange>,
    values: FxIndexMap<String, Point>,
    state: State,
    reset_edge: EdgeDetector,
}
//
// 
impl FnRecOpCycleMetric {
    ///
    /// Creates new instance of the FnRecOpCycleMetric
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, send_to: Option<Sender<Point>>, reset: Option<FnOutRef>, op_cycle: FnOutRef, inputs: impl IntoIterator<Item = (String, FnOutRef)>) -> Self {
        let id = format!("{}/FnRecOpCycleMetric{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self { 
            kind: FnKind::Fn,
            send_to,
            reset: reset.map(FnChange::new),
            op_cycle: FnChange::new(op_cycle),
            inputs: inputs.into_iter().map(|(k, inp)| (k, FnChange::new(inp))).collect(),
            values: FxIndexMap::default(),
            state: State::new(&id),
            reset_edge: EdgeDetector::new(),
            id,
        }
    }
}
// 
impl FnOut for FnRecOpCycleMetric {
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
        let mut inputs = self.op_cycle.inputs();
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.inputs());
        }
        for (_, input) in &self.inputs {
            inputs.append(&mut input.inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let reset = self.reset.as_mut().map(|f| f.out());
        let op_cycle = self.op_cycle.out();
        let inputs: IndexMap<&String, FnResult<FnFlow, String>> = self.inputs.iter_mut()
            .map(|(key, input)| (key, input.out())).collect();
        if let Some(reset) = reset {
            if let Some(reset) = reset? {
                if let Some(Edge::Rising) = self.reset_edge.add(reset.into_value().to_bool().as_bool().value.0) {
                    self.state.reset();
                }
            }
        }
        let Some(op_cycle_point) = flow.map(op_cycle)? else { return Ok(None) };
        let op_cycle = match op_cycle_point.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => op_cycle_point.to_bool().as_bool().value.0,
            _ => return Err(format!("{}.out | Invalid op_cycle type '{:?}', expected bool or number", self.id, op_cycle_point.typ())),
        };
        match self.state.add(op_cycle) {
            Cycle::None => {}
            Cycle::Started => {
                log::trace!("{}.out | Operating Cycle - Active", self.id);
                for (input_name, input) in inputs {
                    if let Some(val_flow) = input? {
                        let value = val_flow.into_value();
                        if value.typ() == PointType::String {
                            // log::debug!("{}.out | '{}': {:?}", self.id, input_name, p.value);
                            // p.name = input_name.to_owned();
                            self.values.insert(input_name.to_owned(), value);
                        } else {
                            log::warn!("{}.out | Input '{}': unexpected type {:?}, string sql requared", self.id, input_name, value.typ());
                        }
                    }
                }
            }
            Cycle::Finished => {
                log::debug!("{}.out | Operating Cycle - SENDING {} values...", self.id, self.values.len());
                let log_values: Vec<String> = self.values.iter().map(|(key, point)| {
                    format!("'{}': '{}'", key, point.value().to_string())
                }).collect();
                log::debug!("{}.out | Operating Cycle - values ({}): {:#?}", self.id, self.values.len(), log_values);
                if let Some(tx) = &self.send_to {
                    for (_, value) in self.values.drain(..) {
                        if let Err(err) = tx.send(value) {
                            log::error!("{}.out | Send error: {:#?}", self.id, err);
                        }
                    }
                } else {
                    log::warn!("{}.out | Point can't be sent - 'send-to' is not specified", self.id);
                    self.values.clear();
                }
            }
        }
        flow.wrap(op_cycle_point)
    }
    //
    fn reset(&mut self) {
        self.state.reset();
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.reset_edge.reset();
        self.op_cycle.reset();
        for (_, input) in &mut self.inputs {
            input.reset();
        }
    }
}
///
/// Global static counter of FnRecOpCycleMetric instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// State of the Operating cycle
#[derive(Debug, Clone, Copy, PartialEq)]
enum Cycle {
    /// Initial state, nothing hapens while this state
    /// Used to detect the start of the operating cycle
    None,
    /// When op-cycle was raised from 0 to 1
    /// Previous state must be `None`
    Started,
    /// When op-cycle was reseted from 1 to 0.
    /// Used for performing the actions at the end of the operating cycle.
    /// After that immediately must be changed to `None`
    Finished,
}
#[derive(Debug)]
struct State {
    state: Cycle,
    dbg: Dbg,
}
//
impl State {
    ///
    /// Returns `State::None`
    pub fn new(parent: impl Into<String>) -> Self {
        Self {
            state: Cycle::None,
            dbg: Dbg::new(parent, "State")
        }
    }
    ///
    /// Returns current state depending on the Operating Cycle status
    /// `op_cycle` - the Operating Cycle is active as boolean
    pub fn add(&mut self, op_cycle: bool) -> Cycle {
        match self.state {
            Cycle::None => match op_cycle {
                true => {
                    log::debug!("{}.add | Operating Cycle - STARTED", self.dbg);
                    self.state = Cycle::Started;
                    self.state
                }
                false => Cycle::None,
            },
            Cycle::Started => match op_cycle {
                true => Cycle::Started,
                false => {
                    log::debug!("{}.add | Operating Cycle - FINISHED", self.dbg);
                    self.state = Cycle::None;
                    Cycle::Finished
                }
            },
            Cycle::Finished => unreachable!(),
        }
    }
    ///
    /// Reset the state to the initial
    #[allow(unused)]
    pub fn reset(&mut self) {
        self.state = Cycle::None;
    }
}
///
/// Проверяем полный жизненный цикл:
/// - спокойствие
/// - запуск (передний фронт)
/// - удержание
/// - завершение (задний фронт)
/// - а также поведение при принудительном сбросе
#[cfg(test)]
mod state_tests {
    use super::*;
    #[test]
    fn test_state_cycle_accuracy() {
        let mut state = State::new("TestMetric");
        // 1. Начальное состояние: система ждет старта (сигнал 0)
        assert_eq!(state.add(false), Cycle::None);
        assert_eq!(state.add(false), Cycle::None);
        // 2. Передний фронт: старт производственного цикла (сигнал 1)
        assert_eq!(state.add(true), Cycle::Started);
        // 3. Удержание: цикл продолжается, повторных стартов быть не должно
        assert_eq!(state.add(true), Cycle::Started);
        assert_eq!(state.add(true), Cycle::Started);
        // 4. Задний фронт: завершение цикла. Триггер должен сработать ровно один раз
        assert_eq!(state.add(false), Cycle::Finished);
        // 5. Возврат в режим ожидания
        assert_eq!(state.add(false), Cycle::None);
        assert_eq!(state.add(false), Cycle::None);
        // 6. Проверка принудительного сброса на «горячую»
        assert_eq!(state.add(true), Cycle::Started);
        state.reset();
        assert_eq!(state.add(false), Cycle::None);
    }
}
#[cfg(test)]
mod integration_tests {
    use std::{cell::RefCell, rc::Rc};
    use super::*;
    use crate::domain::unbounded;
    use sal_sync::services::entity::{PointHlr, Status, Cot};
    // Заглушка для имитации входящих метрик
    #[derive(Debug)]
    struct FakeOrigin {
        value: Option<FnFlow>,
    }
    impl FakeOrigin {
        fn new() -> Self { Self { value: None } }
        fn set_str(&mut self, val: impl Into<String>) {
            let p = Point::String(PointHlr::new(0, "test", val.into(), Status::Ok, Cot::Inf, chrono::Utc::now()));
            self.value = Some(FnFlow::New(p));
        }
        fn set_int(&mut self, val: i64) {
            let p = Point::Int(PointHlr::new(0, "test", val, Status::Ok, Cot::Inf, chrono::Utc::now()));
            self.value = Some(FnFlow::New(p));
        }
    }
    impl FnOut for FakeOrigin {
        fn id(&self) -> String { "fake".to_string() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.value.clone()) }
        fn reset(&mut self) {}
    }
    #[test]
    fn test_snapshot_physics_on_falling_edge() {
        fn wrap(f: FnOutRef) -> FnOutRef { f }
        let (tx, rx) =  unbounded();
        let op_cycle = Rc::new(RefCell::new(FakeOrigin::new()));
        let metric = Rc::new(RefCell::new(FakeOrigin::new()));
        // Инициализация узла
        let mut node = FnRecOpCycleMetric::new(
            "Test", Some(tx), None, op_cycle.clone(),
            vec![("speed".to_string(), wrap(metric.clone()))],
        );
        // Такт 1: Запуск цикла (op_cycle = 1, speed = 1500.0)
        metric.borrow_mut().set_str("1501.0");
        op_cycle.borrow_mut().set_int(1);
        node.out().unwrap();
        assert!(rx.try_recv().is_err(), "Отправки быть не должно, цикл активен");
        // Такт 2: Рабочий режим (op_cycle = 1, speed = 1550.0) - это значение должно стать финальным
        metric.borrow_mut().set_str("1552.0");
        op_cycle.borrow_mut().set_int(1);
        node.out().unwrap();
        assert!(rx.try_recv().is_err(), "Отправки быть не должно, цикл активен");
        // Такт 3: Остановка (op_cycle = 0, speed = 0.0) - двигатель встал
        op_cycle.borrow_mut().set_int(0);
        node.out().unwrap();
        // Проверка: мы должны получить в канал значение 1550.0 (срез прошлого такта), а не 0.0
        let sent_point = rx.try_recv()
            .expect("Данные должны быть отправлены по заднему фронту")
            .expect("Данные должны быть не None");
        assert_eq!(sent_point.value().to_string(), "1552.0", "Узел обязан отправить финальный рабочий срез");
    }
}
