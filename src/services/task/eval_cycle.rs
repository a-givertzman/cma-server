use std::{cell::Cell, rc::Rc};

///
/// Считает бесконечно по кругу
pub(crate) type EvalCycleRef = Rc<EvalCycle>;
///
/// Считает бесконечно по кругу
#[derive(Debug)]
pub(crate) struct EvalCycle {
    val: Cell<CycleIndex>,
}
impl EvalCycle {
    ///
    /// Returns `EvalCycle` new instance
    pub fn new() -> Self {
        Self {
            val: Cell::new(
                CycleIndex::restart()
            )
        }
    }
    ///
    /// Increments current cycle index
    pub fn increment(&self) {
        self.val.update(|mut v| {
            if v.0 >= u64::MAX { return CycleIndex::restart() }
            v.0 = v.0 + 1;
            v
        })
    }
    pub fn get(&self) -> CycleIndex {
        self.val.get()
    }
}
///
/// ### EvalCycle Value
/// 
/// Current number of the cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CycleIndex(u64);
//
impl CycleIndex {
    ///
    /// Returns initial state for the cycle
    pub(crate) fn new() -> Self {
        Self(1)
    }
    ///
    /// Updates current Cycle Index
    /// - Returns true if `cycle` was updated
    /// - Returns false if `cycle` is same
    pub(crate) fn update(&mut self, cycle: &Self) -> bool {
        if self.0 != cycle.0 {
            self.0 = cycle.0;
            return true;
        }
        false
    }
    fn restart() -> Self {
        Self(1)
    }
}
