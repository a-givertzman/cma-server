use sal_sync::services::entity::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{
        EvalCycle, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// ### Function | Evaluates Once per coaclulation cycle & Transparent for calculations
///
/// Реализует запуск вычисления eval() только один раз за вычислительный цикл
/// - Получает номер текущего вычислительного цикла через `task_nodes_cycle: Rc<Cell<usize>>`
/// - Если `task_nodes_cycle` отличен от локального номер цикла, то:
///     - Вычисления выполняются, `licl_cycle =  task_nodes_cycle`
/// - Очередной запуск вычислений в том же вычислительном цикле ни к чему не приведет
/// - Таким образом мы получаем защиту от повторных вычислений
#[derive(Debug)]
pub struct FnEvalOnce {
    id: String,
    /// Локальное значение отработанного вычислительного цикла
    cycle: usize,
    /// Значение текущего вычислительного цикла из `TaskNodes`
    eval_cycle: EvalCycle,
    /// Вычисления
    input: FnOutRef,
    /// Текущий результат вычислений
    state: FnResult<FnFlow, String>,
}
//
// 
impl FnEvalOnce {
    ///
    /// Creates new instance of the FnEvalOnce
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, eval_cycle: EvalCycle, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnEvalOnce{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            cycle: usize::MAX,
            eval_cycle,
            input,
            state: Ok(None),
        }
    }
}
//
// 
impl FnOut for FnEvalOnce {
    //
    fn id(&self) -> String {
        self.input.borrow().id()
    }
    //
    fn kind(&self) -> FnKind {
        self.input.borrow().kind()
    }
    //
    fn inputs(&self) -> Vec<String> {
        self.input.borrow().inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let eval_cycle = self.eval_cycle.get();
        if self.eval_cycle.get() == self.cycle {
            return self.state.clone();
        }
        self.cycle = eval_cycle;
        match self.input.borrow_mut().out() {
            Ok(Some(v)) => {
                self.state = FnResult::Ok(Some(v.clone()));
                FnResult::Ok(Some(v))
            }
            Ok(None) => {
                self.state = Ok(None);
                Ok(None)
            }
            Err(err) => {
                let err = FnResult::Err(format!("{}.out | Error: {}", self.id, err));
                self.state = err.clone();
                err
            }
        }
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
//
// 
// impl FnInOut for FnEvalOnce {}
///
/// Global static counter of FnEvalOnce instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
