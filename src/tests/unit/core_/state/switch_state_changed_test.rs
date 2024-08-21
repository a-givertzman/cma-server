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
        let switch_state: SwitchState<u8, T> = SwitchState::new(
            initial,
            switches,
        );
        switch_state
    }
    ///
    ///
    #[test]
    fn test_state() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        println!();
        println!("test SwitchState");

        let steps: Vec<f64> = vec![0.25, 0.50, 0.75];
        let initial = 1;
        let mut switch_state = SwitchStateChanged::new(
            init_each(initial, steps),
        );

        let mut prev_state = initial;
        for value in 0..=100 {
            let value = 0.01 * (value as f64);
            switch_state.add(value);
            let state = switch_state.state();
            let changed = switch_state.changed();
            info!("state: {},\t changed: {},\t isMax: {},\t value: {}", state, changed, switch_state.is_max(), value);
            if state != prev_state {
                assert!(changed == true, "\nresult: {:?}\ntarget: {:?}", changed, true);
                prev_state = state;
            } else {
                assert!(changed == false, "\nresult: {:?}\ntarget: {:?}", changed, false);
            }
        }
    }
    ///
    /// 
    #[test]
    fn test_state_empty_steps() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        println!();
        println!("test SwitchState empty steps");

        let steps: Vec<f64> = vec![];
        let initial = 1;
        let mut switch_state = SwitchStateChanged::new(
            init_each(initial, steps),
        );

        for value in 0..=100 {
            let value = 0.01 * (value as f64);
            switch_state.add(value);
            let state = switch_state.state();
            let changed = switch_state.changed();
            info!("state: {},\t changed: {},\t isMax: {},\t value: {}", state, changed, switch_state.is_max(), value);
            assert!(changed == false, "\nresult: {:?}\ntarget: {:?}", changed, false);
        }
    }
}

