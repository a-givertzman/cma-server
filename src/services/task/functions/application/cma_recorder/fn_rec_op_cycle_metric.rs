use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::entity::{Point, PointType}, sync::channel::Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}};
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
    op_cycle: FnOutRef,
    inputs: FxIndexMap<String, FnOutRef>,
    values: FxIndexMap<String, Point>,
    state: State,
}
//
// 
impl FnRecOpCycleMetric {
    ///
    /// Creates new instance of the FnRecOpCycleMetric
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, send_to: Option<Sender<Point>>, op_cycle: FnOutRef, inputs: impl IntoIterator<Item = (String, FnOutRef)>) -> Self {
        let id = format!("{}/FnRecOpCycleMetric{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self { 
            kind: FnKind::Fn,
            send_to,
            op_cycle,
            inputs: inputs.into_iter().collect(),
            values: FxIndexMap::default(),
            state: State::new(&id),
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
        let mut inputs = vec![];
        inputs.append(&mut self.op_cycle.borrow().inputs());
        for (_, input) in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(op_cycle_point) = flow.map(self.op_cycle.borrow_mut().out())? else { return Ok(None) };
        let op_cycle = op_cycle_point.to_bool().as_bool().value.0;
        match self.state.add(op_cycle) {
            Cycle::None => {}
            Cycle::Started => {
                log::trace!("{}.out | Operating Cycle - Active", self.id);
                for (input_name, input) in &self.inputs {
                    if let Some(value) = flow.map(input.borrow_mut().out())? {
                        if flow.is_new() {
                            if value.type_() == PointType::String {
                                // log::debug!("{}.out | '{}': {:?}", self.id, input_name, p.value);
                                // p.name = input_name.to_owned();
                                self.values.insert(input_name.to_owned(), value);
                            } else {
                                log::warn!("{}.out | Input '{}': unexpected type {:?}, string sql requared", self.id, input_name, value.type_());
                            }
                        }
                    }
                }
            }
            Cycle::Finished => {
                // Изолируем побочный эффект. Отправляем в очередь только если падение триггера произошло в этом такте
                if flow.is_new() {
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
        }
        // Возвращаем поток триггера, оборачивая его в текущий контекст заражения
        flow.wrap(op_cycle_point)
    }
    //
    fn reset(&mut self) {
        self.state.reset();
        self.op_cycle.borrow_mut().reset();
        for (_, input) in &self.inputs {
            input.borrow_mut().reset();
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
mod tests {
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