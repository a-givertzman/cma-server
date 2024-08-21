use log::error;
use crate::core_::types::map::IndexMapFxHasher;
///
/// Provides callback on connection status changes
pub struct ChangeNotify<T> {
    id: String,
    state: T,
    states: IndexMapFxHasher<T, Box<dyn Fn(&str)>>
}
//
//
impl<T: Clone + std::cmp::PartialEq + std::cmp::Eq + std::hash::Hash + std::fmt::Debug> ChangeNotify<T> {
    //
    //
    pub fn new(parent: impl Into<String>, initial: T, states: Vec<(T, Box<dyn Fn(&str)>)>) -> Self {
        let states = IndexMapFxHasher::from_iter(states);
        Self {
            id: format!("{}/ChangeNotify<{}>", parent.into(), std::any::type_name::<T>()),
            state: initial,
            states,
        }
    }
    ///
    /// Add new state
    pub fn add(&mut self, state: T, message: &str) {
        if state != self.state {
            match self.states.get(&state) {
                Some(callback) => {
                    (callback)(message)
                },
                None => error!("{}.add | State `{:?}` is not found", self.id, state),
            }
            self.state = state;
        }
    }
}