use std::{cell::Cell, rc::Rc};

///
/// Считает бесконечно по кругу
pub(crate) type EvalCycleRef = Rc<EvalCycle>;
///
/// Считает бесконечно по кругу
#[derive(Debug)]
pub(crate) struct EvalCycle {
    val: Cell<usize>,
}
impl EvalCycle {
    pub(crate) const START: usize = 0;
    const INIT: usize = 1;
    pub fn new() -> Self {
        Self { val: Cell::new(Self::INIT) }
    }
    pub fn increment(&self) {
        self.val.update(|v| {
            if v >= usize::MAX { return Self::INIT }
            v + 1
        })
    }
    pub fn get(&self) -> usize {
        self.val.get()
    }
}
