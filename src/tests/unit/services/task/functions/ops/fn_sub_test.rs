#[cfg(test)]
use sal_sync::services::{entity::{Point, ToPoint, Status}, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef, 
    services::task::{EvalCycle, EvalCycleRef, FnInput, FnOut, FnSub},
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
        FnInput::new("test", 0, &mut conf, cycle)
    ))
}
///
/// Testing Task Sub Bool's
#[ignore = "Task FnSub ignored for Bool's - not implemented, under discussion"]
#[test]
fn test_bool() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    log::info!("test_bool");
    let mut value1_stored;
    let mut value2_stored = false.to_point(0, "bool");
    let mut target: Point;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("false", FnConfPointType::Bool, &cycle);
    let input2 = init_each("false", FnConfPointType::Bool, &cycle);
    let mut fn_sub = FnSub::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        (false, false),
        (false, true),
        (false, false),
        (true, false),
        (false, false),
        (true, true),
        (false, false),
    ];
    for (value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_sub.out().unwrap().unwrap().into_value();
        log::debug!("value1: {:?}   |   state: {:?}", value1, state);
        value1_stored = point1.clone();
        target = Point::Bool(value1_stored.as_bool() + value2_stored.as_bool());
        assert_eq!(state, target);
        input2.borrow_mut().add(&point2);
        let state = fn_sub.out().unwrap().unwrap().into_value();
        log::debug!("value2: {:?}   |   state: {:?}", value2, state);
        value2_stored = point2.clone();
        target = Point::Bool(value1_stored.as_bool() + value2_stored.as_bool());
        assert_eq!(state, target);
    }
}
///
/// Testing Task Sub Int's
#[test]
fn test_int() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    log::info!("test_int");
    let mut value1_stored;
    let mut value2_stored = 0.to_point(0, "int");
    let mut target: Point;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0", FnConfPointType::Int, &cycle);
    let input2 = init_each("0", FnConfPointType::Int, &cycle);
    let mut fn_sub = FnSub::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        (01, 1, 1, Ok::<(),()>(()), Ok(())),
        (02, 2, 2, Ok(()), Ok(())),
        (03, 5, 5, Ok(()), Ok(())),
        (04, -1, 1, Ok(()), Ok(())),
        (05, -5, 1, Ok(()), Ok(())),
        (06, 1, -1, Ok(()), Ok(())),
        (07, 1, -5, Ok(()), Ok(())),
        (08, 0, 0, Ok(()), Ok(())),
        (09, i64::MIN, 0, Ok(()), Ok(())),
        (10, 0, i64::MIN, Ok(()), Err(())),
        (11, i64::MAX, 0, Err(()), Ok(())),
        (12, 0, i64::MAX, Ok(()), Ok(())),
    ];
    for (step, value1, value2, target1, target2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_sub.out().transpose().unwrap().map(|v| v.into_value());
        log::debug!("value1: {:?}   |   state: {:?}", value1, state);
        value1_stored = point1.clone();
        if target1.is_ok() {
            let state = state.unwrap();
            target = Point::Int(value1_stored.as_int() - value2_stored.as_int());
            assert_eq!(state.value(), target.value());
            assert_eq!(state.status(), Status::Ok);
        } else {
            assert!(state.is_err(), "Step {step}: \n result: {:?} \n target: Err(_)", state);
        }
        input2.borrow_mut().add(&point2);
        let state = fn_sub.out().transpose().unwrap().map(|v| v.into_value());
        log::debug!("value2: {:?}   |   state: {:?}", value2, state);
        value2_stored = point2.clone();
        if target2.is_ok() {
            let state = state.unwrap();
            target = Point::Int(value1_stored.as_int() - value2_stored.as_int());
            assert_eq!(state.value(), target.value());
            assert_eq!(state.status(), Status::Ok);
        } else {
            assert!(state.is_err(), "Step {step}: \n result: {:?} \n target: Err(_)", state);
        }
    }
}
///
/// Testing Task Sub Real's
#[test]
fn real() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    log::info!("fn_sub_real");
    let mut value1_stored;
    let mut value2_stored = 0.0f32.to_point(0, "real");
    let mut target: f32;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0.0", FnConfPointType::Real, &cycle);
    let input2 = init_each("0.0", FnConfPointType::Real, &cycle);
    let mut fn_sub = FnSub::new(
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
        (08, 0.0, 0.0),
        (09, f32::MIN, 0.0),
        (10, f32::MIN, 0.5),
        (11, f32::MIN, 1.0),
        (12, 0.0, f32::MIN),
        (13, 0.5, f32::MIN),
        (14, 1.0, f32::MIN),
        (15, f32::MAX, 0.0),
        (16, f32::MAX, 0.5),
        (17, f32::MAX, 1.0),
        (18, 0.0, f32::MAX),
        (19, 0.5, f32::MAX),
        (20, 1.0, f32::MAX),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_sub.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.clone();
        target = value1_stored.as_real().value - value2_stored.as_real().value;
        let result = state.as_real().value;
        assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        input2.borrow_mut().add(&point2);
        let state = fn_sub.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.clone();
        target = value1_stored.as_real().value - value2_stored.as_real().value;
        let result = state.as_real().value;
        assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
    }
}
///
/// Testing Task Sub Double's
#[test]
fn double() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    log::info!("fn_sub_double");
    let mut value1_stored;
    let mut value2_stored = 0.0f64.to_point(0, "double");
    let mut target: f64;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0.0", FnConfPointType::Double, &cycle);
    let input2 = init_each("0.0", FnConfPointType::Double, &cycle);
    let mut fn_sub = FnSub::new(
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
        (08, 0.0, 0.0),
        (09, f64::MIN, 0.0),
        (10, f64::MIN, 0.5),
        (11, f64::MIN, 1.0),
        (12, 0.0, f64::MIN),
        (13, 0.5, f64::MIN),
        (14, 1.0, f64::MIN),
        (15, f64::MAX, 0.0),
        (16, f64::MAX, 0.5),
        (17, f64::MAX, 1.0),
        (18, 0.0, f64::MAX),
        (19, 0.5, f64::MAX),
        (20, 1.0, f64::MAX),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_sub.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.clone();
        target = value1_stored.as_double().value - value2_stored.as_double().value;
        let result = state.as_double().value;
        assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        input2.borrow_mut().add(&point2);
        let state = fn_sub.out().unwrap().unwrap().into_value();
        log::debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.clone();
        target = value1_stored.as_double().value - value2_stored.as_double().value;
        let result = state.as_double().value;
        assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
    }
}
