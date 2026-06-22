#[cfg(test)]
use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef, 
    services::task::{EvalCycle, EvalCycleRef, FnInput, FnOut, FnPow}
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
/// Testing FnPow Int's
#[test]
fn int() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    log::info!("fn_pow_int");
    let mut value1_stored;
    let mut value2_stored = 1;
    let mut target: Result<i64, String>;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0", FnConfPointType::Int, &cycle);
    let input2 = init_each("1", FnConfPointType::Int, &cycle);
    let mut fn_pow = FnPow::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        ("01", 1, 1),
        ("02", 2, 2),
        ("03", 5, 5),
        ("04", -1, 1),
        ("05", -5, 1),
        ("06", 1, -1),
        ("07", 1, -5),
        ("08", 0, 0),
        ("09", i64::MIN, 0),
        ("10", 0, i64::MIN),
        ("11", i64::MAX, 0),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_pow.out().transpose().unwrap().map(|v| v.into_value());
        log::debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.as_int().value;
        target = if value2_stored < 0 {
            Err(format!("Can't pow {value1_stored} ^ {value2_stored}"))
        } else {
            value1_stored.checked_pow(value2_stored as u32).ok_or(format!("Can't pow {value1_stored} ^ {value2_stored}"))
        };
        match (&target, &state) {
            (Ok(target), Ok(result)) => {
                let result = result.as_int().value;
                assert_eq!(result, *target, "step {} \n result: {} \n target: {}", step, result, target);
            }
            (Err(_), Err(_)) => {},
            _ => assert_eq!(state.is_ok(), target.is_ok(), "step {} \n result: {:?} \n target: {:?}", step, state, target),
        }
        input2.borrow_mut().add(&point2);
        let state = fn_pow.out().transpose().unwrap().map(|v| v.into_value());
        log::debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.as_int().value;
        target = if value2_stored < 0 {
            Err(format!("Can't pow {value1_stored} ^ {value2_stored}"))
        } else {
            value1_stored.checked_pow(value2_stored as u32).ok_or(format!("Can't pow {value1_stored} ^ {value2_stored}"))
        };
        match (&target, &state) {
            (Ok(target), Ok(result)) => {
                let result = result.as_int().value;
                assert_eq!(result, *target, "step {} \n result: {} \n target: {}", step, result, target);
            }
            (Err(_), Err(_)) => {},
            _ => assert_eq!(state.is_ok(), target.is_ok(), "step {} \n result: {:?} \n target: {:?}", step, state, target),
        }
    }
}
///
/// Testing FnPow Real's
#[test]
fn real() {
    DebugSession::new().filter(LogLevel::Info).init();
    fn checked_powf(base: f32, exp: f32) -> Result<f32, ()> {
        let result = base.powf(exp);
        if result.is_finite() { Ok(result) } else { Err(()) }
    }
    init_once();
    log::info!("fn_pow_real");
    let mut value1_stored;
    let mut value2_stored = 1.0f32;
    let mut target: Result<f32, ()>;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0.0", FnConfPointType::Real, &cycle);
    let input2 = init_each("1.0", FnConfPointType::Real, &cycle);
    let mut fn_pow = FnPow::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        ("01", 0.1, 0.1),
        ("02", 0.2, 0.2),
        ("03", 0.5, 0.5),
        // ("04", -0.1, 0.1),
        // ("05", -0.5, 0.1),
        ("06", 0.1, -0.1),
        ("07", 0.1, -0.5),
        ("08", 0.0, 0.0),
        ("09", f32::MIN, 0.0),
        // ("10", f32::MIN, 0.5),
        ("11", f32::MIN, 1.0),
        ("12", 0.0, f32::MIN),
        ("13", 0.5, f32::MIN),
        ("14", 1.0, f32::MIN),
        ("15", f32::MAX, 0.0),
        ("16", f32::MAX, 0.5),
        ("17", f32::MAX, 1.0),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_pow.out().transpose().unwrap().map(|v| v.into_value());
        log::debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.as_real().value;
        target = checked_powf(value1_stored, value2_stored);
        match (&target, &state) {
            (Ok(target), Ok(result)) => {
                let result = result.as_real().value;
                assert_eq!(result, *target, "step {} \n result: {} \n target: {}", step, result, target);
            }
            (Err(_), Err(_)) => {},
            _ => assert_eq!(state.is_ok(), target.is_ok(), "step {} \n result: {:?} \n target: {:?}", step, state, target),
        }
        input2.borrow_mut().add(&point2);
        let state = fn_pow.out().transpose().unwrap().map(|v| v.into_value());
        log::debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.as_real().value;
        target = checked_powf(value1_stored, value2_stored);
        match (&target, &state) {
            (Ok(target), Ok(result)) => {
                let result = result.as_real().value;
                assert_eq!(result, *target, "step {} \n result: {} \n target: {}", step, result, target);
            }
            (Err(_), Err(_)) => {},
            _ => assert_eq!(state.is_ok(), target.is_ok(), "step {} \n result: {:?} \n target: {:?}", step, state, target),
        }
    }
}
///
/// Testing FnPow Double's
#[test]
fn double() {
    DebugSession::new().filter(LogLevel::Info).init();
    fn checked_powf(base: f64, exp: f64) -> Result<f64, ()> {
        let result = base.powf(exp);
        if result.is_finite() { Ok(result) } else { Err(()) }
    }
    init_once();
    log::info!("fn_pow_double");
    let mut value1_stored;
    let mut value2_stored = 1.0f64;
    let mut target: Result<f64, ()>;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0.0", FnConfPointType::Double, &cycle);
    let input2 = init_each("1.0", FnConfPointType::Double, &cycle);
    let mut fn_pow = FnPow::new(
        "test",
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        ("01", 0.1, 0.1),
        ("02", 0.2, 0.2),
        ("03", 0.5, 0.5),
        // ("04", -0.1, 0.1),
        // ("05", -0.5, 0.1),
        ("06", 0.1, -0.1),
        ("07", 0.1, -0.5),
        ("08", 0.0, 0.0),
        ("09", f64::MIN, 0.0),
        // ("10", f64::MIN, 0.5),
        ("11", f64::MIN, 1.0),
        ("12", 0.0, f64::MIN),
        ("13", 0.5, f64::MIN),
        ("14", 1.0, f64::MIN),
        ("15", f64::MAX, 0.0),
        ("16", f64::MAX, 0.5),
        ("17", f64::MAX, 1.0),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = fn_pow.out().transpose().unwrap().map(|v| v.into_value());
        log::debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.as_double().value;
        target = checked_powf(value1_stored, value2_stored);
        match (&target, &state) {
            (Ok(target), Ok(result)) => {
                let result = result.as_double().value;
                assert_eq!(result, *target, "step {} \n result: {} \n target: {}", step, result, target);
            }
            (Err(_), Err(_)) => {},
            _ => assert_eq!(state.is_ok(), target.is_ok(), "step {} \n result: {:?} \n target: {:?}", step, state, target),
        }
        input2.borrow_mut().add(&point2);
        let state = fn_pow.out().transpose().unwrap().map(|v| v.into_value());
        log::debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.as_double().value;
        target = checked_powf(value1_stored, value2_stored);
        match (&target, &state) {
            (Ok(target), Ok(result)) => {
                let result = result.as_double().value;
                assert_eq!(result, *target, "step {} \n result: {} \n target: {}", step, result, target);
            }
            (Err(_), Err(_)) => {},
            _ => assert_eq!(state.is_ok(), target.is_ok(), "step {} \n result: {:?} \n target: {:?}", step, state, target),
        }
    }
}
