#[cfg(test)]
use log::{debug, info};
use std::{sync::Once, rc::Rc, cell::RefCell};

use crate::{
    core_::{debug::debug_session::{DebugSession, LogLevel, Backtrace},
    point::point_type::{PointType, ToPoint}, types::fn_in_out_ref::FnInOutRef},
    services::task::nested_function::{fn_::{FnInOut, FnOut},
    fn_count::FnCount, fn_input::FnInput},
};
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
fn init_each(initial: PointType) -> FnInOutRef {
    fn boxFnInput(input: FnInput) -> Box<(dyn FnInOut)> {
        Box::new(input)
    }
    Rc::new(RefCell::new(
        boxFnInput(
            FnInput::new("test", initial)
        )
    ))
}
///
///
#[test]
fn test_single() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    info!("test_single");
    let input = init_each(false.toPoint("bool"));
    let mut fnCount = FnCount::new(
        "test",
        0,
        input.clone(),
    );
    let test_data = vec![
        (false, 0),
        (false, 0),
        (true, 1),
        (false, 1),
        (false, 1),
        (true, 2),
        (false, 2),
        (true, 3),
        (false, 3),
        (false, 3),
        (true, 4),
        (true, 4),
        (false, 4),
        (false, 4),
    ];
    for (value, targetState) in test_data {
        let point = value.toPoint("test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fnCount.out();
        // debug!("input: {:?}", &mut input);
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state.asInt().value, targetState);
    }
}
//

#[test]
fn test_multiple() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    info!("test_multiple");
    let input = init_each(false.toPoint("bool"));
    let mut fnCount = FnCount::new(
        "test",
        0,
        input.clone(),
    );
    let test_data = vec![
        (false, 0),
        (false, 0),
        (true, 1),
        (false, 1),
        (false, 1),
        (true, 2),
        (false, 2),
        (true, 3),
        (false, 3),
        (false, 3),
        (true, 4),
        (true, 4),
        (false, 4),
        (false, 4),
    ];
    for (value, targetState) in test_data {
        let point = value.toPoint("test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fnCount.out();
        // debug!("input: {:?}", &mut input);
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state.asInt().value, targetState);
    }
}

#[test]
fn test_multiple_reset() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    info!("test_multiple_reset");
    let input = init_each(false.toPoint("bool"));
    let mut fnCount = FnCount::new(
        "test",
        0,
        input.clone(),
    );
    let test_data = vec![
        (false, 0, false),
        (false, 0, false),
        (true, 1, false),
        (false, 1, false),
        (false, 1, false),
        (true, 2, false),
        (false, 0, true),
        (true, 1, false),
        (false, 1, false),
        (false, 1, false),
        (true, 2, false),
        (true, 2, false),
        (false, 0, true),
        (false, 0, false),
    ];
    for (value, targetState, reset) in test_data {
        if reset {
            fnCount.reset();
        }
        let point = value.toPoint("test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fnCount.out();
        // debug!("input: {:?}", &mut input);
        debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state.asInt().value, targetState);
    }
}
