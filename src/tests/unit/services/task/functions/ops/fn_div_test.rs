#[cfg(test)]
use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, rc::Rc, cell::RefCell};
use debugging::session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef, 
    services::task::{EvalCycle, EvalCycleRef, FnDiv, FnInput, FnOut},
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
fn init_each(default: &str, typ: FnConfPointType, cycle: &EvalCycleRef) -> FnInOutRef {
    let mut conf = FnConfig { name: "test".to_owned(), type_: typ, options: FnConfOptions {default: Some(default.into()), ..Default::default()}, ..Default::default()};
    Rc::new(RefCell::new(
        FnInput::new("test", 0, &mut conf, &cycle)
    ))
}
///
/// Testing Mul Bool's
#[ignore = "Task FnDiv ignored for Bool's - not implemented, under discussion"]
#[test]
fn bool() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    log::info!("fn_div_bool");
    let mut value1_stored;
    let mut value2_stored = false.to_point(0, "bool");
    let mut target: bool;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("fasle", FnConfPointType::Bool, &cycle);
    let input2 = init_each("fasle", FnConfPointType::Bool, &cycle);
    let mut fn_div = FnDiv::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        (01, false, false),
        (02, false, true),
        (03, false, false),
        (04, true, false),
        (05, false, false),
        (06, true, true),
        (07, false, false),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_div.out().unwrap().unwrap().into_value();
        log::debug!("value1: {:?}   |   state: {:?}", value1, state);
        value1_stored = point1.clone();
        target = value1_stored.as_bool().value.0 && value2_stored.as_bool().value.0;
        let result = state.as_bool().value.0;
        assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        input2.borrow_mut().add(&point2);
        let state = fn_div.out().unwrap().unwrap().into_value();
        log::debug!("value2: {:?}   |   state: {:?}", value2, state);
        value2_stored = point2.clone();
        target = value1_stored.as_bool().value.0 && value2_stored.as_bool().value.0;
        let result = state.as_bool().value.0;
        assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
    }
}
///
/// Testing Mul Int's
#[test]
fn int() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    log::info!("fn_div_int");
    let mut value1_stored;
    let mut value2_stored = 1.to_point(0, "int");
    let mut target: i64;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0", FnConfPointType::Int, &cycle);
    let input2 = init_each("1", FnConfPointType::Int, &cycle);
    let mut fn_div = FnDiv::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        (01, 1, 1),
        (02, 2, 2),
        (03, 5, 5),
        (04, -1, 1),
        (05, -5, 1),
        (06, 1, -1),
        (07, 1, -5),
        (08, 0, 1),
        (09, i64::MIN, 1),
        (10, 0, i64::MIN),
        (11, i64::MAX, 1),
        (12, 0, i64::MAX),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_div.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.clone();
        target = value1_stored.as_int().value / value2_stored.as_int().value;
        let result = state.as_int().value;
        assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        input2.borrow_mut().add(&point2);
        let state = fn_div.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.clone();
        target = value1_stored.as_int().value / value2_stored.as_int().value;
        let result = state.as_int().value;
        assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
    }
}
///
/// Testing Mul Real's
#[test]
fn real() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    log::info!("fn_div_real");
    let mut value1_stored;
    let mut value2_stored = 1.0f32.to_point(0, "real");
    let mut target: f32;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0.0", FnConfPointType::Real, &cycle);
    let input2 = init_each("1.0", FnConfPointType::Real, &cycle);
    let mut fn_div = FnDiv::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        (01, 0.1, 0.1),
        (02, 0.2, 0.2),
        (03, 0.5, 0.5),
        (04, -0.1, 0.1),
        (05, -0.5, 0.1),
        (06, 0.1, -0.1),
        (07, 0.1, -0.5),
        (08, 0.0, 1.0),
        (09, f32::MIN, 1.0),
        (10, f32::MIN, 1.5),
        (11, f32::MIN, 2.0),
        (12, 0.0, f32::MIN),
        (13, 0.5, f32::MIN),
        (14, 1.0, f32::MIN),
        (15, f32::MAX, 1.0),
        (16, f32::MAX, 1.5),
        (17, f32::MAX, 2.0),
        (18, 0.0, f32::MAX),
        (19, 0.5, f32::MAX),
        (20, 1.0, f32::MAX),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_div.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.clone();
        target = value1_stored.as_real().value / value2_stored.as_real().value;
        let result = state.as_real().value;
        assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        input2.borrow_mut().add(&point2);
        let state = fn_div.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.clone();
        target = value1_stored.as_real().value / value2_stored.as_real().value;
        let result = state.as_real().value;
        assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
    }
}
///
/// Testing Mul Double's
#[test]
fn double() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    log::info!("fn_div_double");
    let mut value1_stored;
    let mut value2_stored = 1.0f64.to_point(0, "double");
    let mut target: f64;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0.0", FnConfPointType::Double, &cycle);
    let input2 = init_each("1.0", FnConfPointType::Double, &cycle);
    let mut fn_div = FnDiv::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        (01, 0.1, 0.1),
        (02, 0.2, 0.2),
        (03, 0.5, 0.5),
        (04, -0.1, 0.1),
        (05, -0.5, 0.1),
        (06, 0.1, -0.1),
        (07, 0.1, -0.5),
        (08, 0.0, 1.0),
        (09, f64::MIN, 1.0),
        (10, f64::MIN, 1.5),
        (11, f64::MIN, 2.0),
        (12, 0.0, f64::MIN),
        (13, 0.5, f64::MIN),
        (14, 1.0, f64::MIN),
        (15, f64::MAX, 1.0),
        (16, f64::MAX, 1.5),
        (17, f64::MAX, 2.0),
        (18, 0.0, f64::MAX),
        (19, 0.5, f64::MAX),
        (20, 1.0, f64::MAX),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_div.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.clone();
        target = value1_stored.as_double().value / value2_stored.as_double().value;
        let result = state.as_double().value;
        assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        input2.borrow_mut().add(&point2);
        let state = fn_div.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.clone();
        target = value1_stored.as_double().value / value2_stored.as_double().value;
        let result = state.as_double().value;
        assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
    }
}
