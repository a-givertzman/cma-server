use super::switch_state::SwitchState;
///
/// Returns true once if inner SwitchState is just changed the state
pub struct SwitchStateChanged<TState, TInput> {
    switch_state: SwitchState<TState, TInput>,
    prev: TState,
}
//
// 
impl<TState: std::fmt::Debug + Eq + Ord + core::hash::Hash + Clone, TInput: Clone> SwitchStateChanged<TState, TInput> {
    ///
    /// Creates new instance of the SwitchStateChanged
    pub fn new(switch_state: SwitchState<TState, TInput>) -> Self {
        let prev = switch_state.state();
        Self {
            switch_state,
            prev,
        }
    }
    ///
    /// Adds new value to the current state
    pub fn add(& mut self, value: TInput) {
        self.switch_state.add(value)
    }
    ///
    /// Returns current state
    pub fn state(&self) -> TState {
        self.switch_state.state()
    }
    ///
    /// resets current state to initial
    pub fn reset(&mut self) {
        self.switch_state.reset();
    }
    ///
    /// Returns true if state just changed by last call of add method
    pub fn changed(&mut self) -> bool {
        let changed = self.switch_state.state() != self.prev;
        self.prev = self.switch_state.state();
        changed
    }
    ///
    /// Returns true if the last state riched
    pub fn is_max(&self) -> bool {
        self.switch_state.is_max()
    }
}