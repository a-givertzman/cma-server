use log::error;
use sal_sync::collections::map::IndexMapFxHasher;
///
/// Provides callback on connection status changes
pub struct ChangeNotify<S, T> {
    id: String,
    state: S,
    states: IndexMapFxHasher<S, Box<dyn Fn(T)>>
}
//
//
impl<S: Clone + std::cmp::PartialEq + std::cmp::Eq + std::hash::Hash + std::fmt::Debug, T> ChangeNotify<S, T> {
    //
    //
    pub fn new(parent: impl Into<String>, initial: S, states: Vec<(S, Box<dyn Fn(T)>)>) -> Self {
        // fn callback<T>(c: impl Fn(T) + 'static) -> Box<dyn Fn(T)> {
        //     Box::new(c)
        // }
        let states = IndexMapFxHasher::from_iter(states);
        Self {
            id: format!("{}/ChangeNotify<{}>", parent.into(), std::any::type_name::<S>()),
            state: initial,
            states,
        }
    }
    ///
    /// Add new state
    pub fn add(&mut self, state: S, message: T) {
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