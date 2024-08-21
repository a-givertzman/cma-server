use log::trace;
use std::{collections::HashMap, fmt::Debug, hash::Hash};
///
/// FSM switch
///  - holds the state 
///  - contains the conditions that lead to the state
pub struct Switch<TState, TInput> {
    pub state: TState,
    pub conditions: Vec<SwitchCondition<TState, TInput>>,
}
//
//
impl<TState: Debug, TInput> Debug for Switch<TState, TInput> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
        .field("state", &self.state)
        .field("conditions", &self.conditions)
        .finish()
    }
}
///
/// FSM switch state condition
///  - holds condition when to switch
///  - holds target - where to switch
pub struct SwitchCondition<TState, TInput> {
    pub condition: Box<dyn Fn(TInput) -> bool>,
    pub target: TState,
}
//
//
impl<TState: Debug, TInput> Debug for SwitchCondition<TState, TInput> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwitchCondition")
        .field("target", &self.target)
        .finish()
    }
}
///
/// Finit State Machine (FSM) implementation
#[derive(Debug)]
pub struct SwitchState<TState, TInput> {
    initial: TState,
    state: TState,
    switches: HashMap<TState, Switch<TState, TInput>>,
}
//
//
impl<TState: Debug + Eq + Ord + Hash + Clone, TInput: Clone> SwitchState<TState, TInput> {
    pub fn new(initial: TState, switches: Vec<Switch<TState, TInput>>) -> Self {
        let mut switches_set = HashMap::new();
        for switch in switches {
            // let key = format!("{:?}", switch.state);
            switches_set.insert(switch.state.clone(), switch);
        }
        trace!("SwitchState{{switches: {:?}}}", &switches_set);
        Self { 
            initial: initial.clone(),
            state: initial,
            switches: switches_set,
        }
    }
    ///
    /// Adds new value to the current state
    pub fn add(& mut self, value: TInput) {
        let key = self.state.clone(); 
        let switch_ref = &self.switches[&key];
        // let switch: Switch<T, U> = switchRef.clone().to_owned();
        for switch_condition in &switch_ref.conditions {            
            let cond = (switch_condition.condition)(value.clone());
            if cond {
                self.state = switch_condition.target.clone();
            }
        };
    }
    ///
    /// Returns current state
    pub fn state(&self) -> TState {
        self.state.clone()
    }
    ///
    /// resets current state to initial
    pub fn reset(&mut self) {
        self.state = self.initial.clone();
    }
    ///
    /// Returns true if the last state is riched
    pub fn is_max(&self) -> bool {
        match self.switches.keys().max() {
            Some(max) => {
                self.state == *max
            }
            None => panic!("SwitchState.isMax | switches collection is empty"),
        }
    }
}