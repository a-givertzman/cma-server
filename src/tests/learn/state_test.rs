#![allow(non_snake_case)]
#[cfg(test)]

mod tests {
    use log::info;
    use std::sync::Once;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::core_::state::{switch_state::{SwitchState, Switch, SwitchCondition}, switch_state_changed::SwitchStateChanged};
    ///
    ///
    static INIT: Once = Once::new();
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        })
    }
    ///
    /// returns:
    ///  - ...
    fn init_each<T: std::cmp::PartialOrd + Clone + 'static>(initial: u8, steps: Vec<T>) -> SwitchState<u8, T> {
        fn switch<T: std::cmp::PartialOrd + Clone + 'static>(state: &mut u8, input: Option<T>) -> Switch<u8, T> {
            let state_ = *state;
            *state = *state + 1;
            let target = state;
            Switch{
                state: state_,
                conditions: vec![
                    SwitchCondition {
                        condition: Box::new(move |value| {
                            match input.clone() {
                                Some(input) => value >= input,
                                None => false,
                            }
                        }),
                        target: *target,
                    },
                ],
            }
        }
        let mut state: u8 = initial;
        let mut switches: Vec<Switch<u8, T>> = steps.into_iter().map(|input| {switch(&mut state, Some(input))}).collect();
        switches.push(switch(&mut state, None));
        let switchState: SwitchState<u8, T> = SwitchState::new(
            initial,
            switches,
        );
        switchState
    }
    ///
    ///
    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn test_state() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        println!();
        println!("test SwitchState");
        let steps: Vec<f64> = vec![0.25, 0.50, 0.75];
        let mut switchState = SwitchStateChanged::new(
            init_each(1, steps),
        );
        for value in 0..=100 {
            let value = 0.01 * (value as f64);
            switchState.add(value);
            let state = switchState.state();
            let changed = switchState.changed();
            info!("state: {},\t changed: {},\t value: {}", state, changed, value);
        }
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
}

