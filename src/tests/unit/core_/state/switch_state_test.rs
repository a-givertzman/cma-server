#[cfg(test)]

mod tests {
    use log::{debug, info};
    use std::sync::Once;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::core_::state::switch_state::{Switch, SwitchCondition, SwitchState};
    ///
    ///
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    enum ProcessState {
        Off,
        Start,
        Progress,
        Stop,
    }
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
    /// returns tuple(
    ///     - initialState: ProcessState
    ///     - switches: Vec<Switch<ProcessState, u8>>
    /// )
    fn init_each() -> (ProcessState, Vec<Switch<ProcessState, i8>>) {
        (
            ProcessState::Off,
            vec![
                Switch{
                    state: ProcessState::Off,
                    conditions: vec![
                        SwitchCondition {
                            condition: Box::new(|value| {value >= 5}),
                            target: ProcessState::Start,
                        },
                    ],
                },
                Switch{
                    state: ProcessState::Stop,
                    conditions: vec![
                        SwitchCondition {
                            condition: Box::new(|value| {value >= 5}),
                            target: ProcessState::Start,
                        },
                        SwitchCondition {
                            condition: Box::new(|value| {value < 5}),
                            target: ProcessState::Off,
                        },
                    ],
                },
                Switch{
                    state: ProcessState::Start,
                    conditions: vec![
                        SwitchCondition {
                            condition: Box::new(|value| {value >= 5}),
                            target: ProcessState::Progress,
                        },
                        SwitchCondition {
                            condition: Box::new(|value| {value < 5}),
                            target: ProcessState::Stop,
                        },
                    ],

                },
                Switch{
                    state: ProcessState::Progress,
                    conditions: vec![
                        SwitchCondition {
                            condition: Box::new(|value| {value < 5}),
                            target: ProcessState::Stop,
                        },
                    ],

                },
            ]
        )
    }
    ///
    ///
    #[test]
    fn test_single() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test_single");
        let (initial, switches) = init_each();
        let mut switch_state: SwitchState<ProcessState, i8> = SwitchState::new(
            initial,
            switches,
        );
        let test_data = vec![
            (0, ProcessState::Off),
            (0, ProcessState::Off),
            (1, ProcessState::Off),
            (1, ProcessState::Off),
            (2, ProcessState::Off),
            (2, ProcessState::Off),
            (5, ProcessState::Start),
            (5, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (7, ProcessState::Progress),
            (7, ProcessState::Progress),
            (7, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (5, ProcessState::Progress),
            (5, ProcessState::Progress),
            (2, ProcessState::Stop),
            (2, ProcessState::Off),
            (1, ProcessState::Off),
            (1, ProcessState::Off),
        ];
        for (value, target_state) in test_data {
            switch_state.add(value);
            let state = switch_state.state();
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state, target_state);
            if state == ProcessState::Stop {
                assert_eq!(switch_state.is_max(), true);
            } else {
                assert_eq!(switch_state.is_max(), false);
            }
        }
    }
    ///
    ///
    #[test]
    fn test_start_step_back() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test_start_step_back");
        let (initial, switches) = init_each();
        let mut switch_state: SwitchState<ProcessState, i8> = SwitchState::new(
            initial,
            switches,
        );
        let test_data = vec![
            (0, ProcessState::Off),
            (0, ProcessState::Off),
            (1, ProcessState::Off),
            (1, ProcessState::Off),
            (2, ProcessState::Off),
            (2, ProcessState::Off),
            (5, ProcessState::Start),
            (0, ProcessState::Stop),
            (6, ProcessState::Start),
            (0, ProcessState::Stop),
            (6, ProcessState::Start),
            (7, ProcessState::Progress),
            (7, ProcessState::Progress),
            (7, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (5, ProcessState::Progress),
            (5, ProcessState::Progress),
            (2, ProcessState::Stop),
            (2, ProcessState::Off),
            (1, ProcessState::Off),
            (1, ProcessState::Off),
        ];
        for (value, target_state) in test_data {
            switch_state.add(value);
            let state = switch_state.state();
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state, target_state);
        }
    }
    ///
    ///
    #[test]
    fn test_stot_step_back() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test_stot_step_back");
        let (initial, switches) = init_each();
        let mut switch_state: SwitchState<ProcessState, i8> = SwitchState::new(
            initial,
            switches,
        );
        let test_data = vec![
            (0, ProcessState::Off),
            (0, ProcessState::Off),
            (1, ProcessState::Off),
            (1, ProcessState::Off),
            (2, ProcessState::Off),
            (2, ProcessState::Off),
            (5, ProcessState::Start),
            (5, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (7, ProcessState::Progress),
            (7, ProcessState::Progress),
            (7, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (6, ProcessState::Progress),
            (5, ProcessState::Progress),
            (2, ProcessState::Stop),
            (7, ProcessState::Start),
            (2, ProcessState::Stop),
            (1, ProcessState::Off),
            (1, ProcessState::Off),
        ];
        for (value, target_state) in test_data {
            switch_state.add(value);
            let state = switch_state.state();
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state, target_state);
        }
    }
}
