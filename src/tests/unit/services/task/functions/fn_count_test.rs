#[cfg(test)]
use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, rc::Rc, cell::RefCell};
use debugging::session::{DebugSession, LogLevel};
use crate::{
     domain::{FnInOutRef, FnOutRef},
    services::task::{
        FnCount, FnInput, FnOut
    }
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
fn init_each(default: &str, type_: FnConfPointType) -> (FnOutRef, FnInOutRef) {
    let mut conf = FnConfig { name: "test".to_owned(), type_, options: FnConfOptions {default: Some(default.into()), ..Default::default()}, ..Default::default()};
    let input = Rc::new(RefCell::new(
        FnInput::new("test", 0, &mut conf)
    ));
    (input.clone(), input)
}
///
///
#[test]
fn test_single() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    log::info!("test_single");
    let (initial, _) = init_each("0", FnConfPointType::Int);
    let (input1, input) = init_each("false", FnConfPointType::Bool);
    let mut fn_count = FnCount::new(
        "test",
        Some(initial),
        input1,
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
    for (value, target) in test_data {
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fn_count.out().unwrap();
        // debug!("input: {:?}", &mut input);
        log::debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state.as_int().value, target);
    }
}
//

#[test]
fn test_multiple() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    log::info!("test_multiple");
    let (initial, _) = init_each("0", FnConfPointType::Int);
    let (input1, input) = init_each("false", FnConfPointType::Bool);
    let mut fn_count = FnCount::new(
        "test",
        Some(initial),
        input1,
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
    for (value, target) in test_data {
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fn_count.out().unwrap();
        // debug!("input: {:?}", &mut input);
        log::debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state.as_int().value, target);
    }
}

#[test]
fn test_multiple_reset() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    log::info!("test_multiple_reset");
    let (initial, _) = init_each("0", FnConfPointType::Int);
    let (input1, input) = init_each("false", FnConfPointType::Bool);
    let mut fn_count = FnCount::new(
        "test",
        Some(initial),
        input1,
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
    for (value, target, reset) in test_data {
        if reset {
            fn_count.reset();
        }
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fn_count.out().unwrap();
        // debug!("input: {:?}", &mut input);
        log::debug!("value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state.as_int().value, target);
    }
}
