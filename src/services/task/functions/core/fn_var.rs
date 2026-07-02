use std::sync::atomic::{Ordering, AtomicUsize};
use crate::{domain::FnOutRef, services::task::{CycleIndex, EvalCycleRef, FnFlow}};
use super::{FnOut, FnKind, FnResult};
///
/// ### Variable | Specific kinde of function
/// - has reference to calculations corresponding to the variable name
#[derive(Debug, Clone)]
pub struct FnVar {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    /// Локальное значение отработанного вычислительного цикла
    cycle: CycleIndex,
    /// Значение текущего вычислительного цикла из `TaskNodes`
    eval_cycle: EvalCycleRef,
    /// Текущий результат вычислений
    state: FnResult<FnFlow, String>,
}
//
impl FnVar {
    pub fn new(parent: impl Into<String>, cycle: EvalCycleRef, input: FnOutRef) -> Self {
        Self {
            id: format!("{}/FnVar{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Var,
            input,
            cycle: CycleIndex::new(),
            eval_cycle: cycle,
            state: Ok(None),
        }
    }
}
//
impl FnOut for FnVar {
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
    ///
    /// - Evaluate calculations
    /// - Returns calculated value
    /// - Returns error if:
    ///   - Calculations fails
    ///   - Input not initialized
    /// - Returns None if:
    ///   - Point filtered by any kind of filtering function
    fn out(&mut self) -> FnResult<FnFlow, String> {
        if !self.cycle.update(&self.eval_cycle.get()) {
            return self.state.clone();
        }
        match self.input.borrow_mut().out() {
            Ok(Some(v)) => {
                self.state = FnResult::Ok(Some(v.clone()));
                Ok(Some(v))
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
    fn hard_reset(&mut self) {
        self.input.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnVar instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
