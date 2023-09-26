#![allow(non_snake_case)]
#[cfg(test)]
use std::env;
use std::sync::Once;
use log::{debug, info};
use crate::core::state::switch_state::{Switch, SwitchCondition, SwitchState};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum ProcessState {
    Off,
    Start,
    Progress,
    Stop,
}

// Note this useful idiom: importing names from outer (for mod tests) scope.
// use super::*;

static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
            env::set_var("RUST_LOG", "debug");  // off / error / warn / info / debug / trace
            // env::set_var("RUST_BACKTRACE", "1");
            env::set_var("RUST_BACKTRACE", "full");
            env_logger::init();
        }
    )
}


///
/// returns tuple(
///     - initialState: ProcessState
///     - switches: Vec<Switch<ProcessState, u8>>
/// )
fn initEach() -> (ProcessState, Vec<Switch<ProcessState, i8>>) {
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

#[test]
fn test_single() {
    init();
    info!("test_single");

    let (initial, switches) = initEach();
    let mut switchState: SwitchState<ProcessState, i8> = SwitchState::new(
        initial,
        switches,
    );
    let testData = vec![
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
    for (value, targetState) in testData {
        switchState.add(value);
        let state = switchState.state();
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state, targetState);
    }        
}

#[test]
fn test_start_step_back() {
    init();
    info!("test_start_step_back");

    let (initial, switches) = initEach();
    let mut switchState: SwitchState<ProcessState, i8> = SwitchState::new(
        initial,
        switches,
    );
    let testData = vec![
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
    for (value, targetState) in testData {
        switchState.add(value);
        let state = switchState.state();
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state, targetState);
    }        
}

#[test]
fn test_stot_step_back() {
    init();
    info!("test_stot_step_back");

    let (initial, switches) = initEach();
    let mut switchState: SwitchState<ProcessState, i8> = SwitchState::new(
        initial,
        switches,
    );
    let testData = vec![
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
    // let sequence = vec![0,0,1,1,2,2,5,0,6,0,6,7,7,7,6,6,6,5,2,7,2,1,1];
    for (value, targetState) in testData {
        switchState.add(value);
        let state = switchState.state();
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state, targetState);
    }        
}
