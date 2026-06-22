#[cfg(test)]
use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef, 
    services::task::{EvalCycle, EvalCycleRef, FnInput, FnLt, FnOut},
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
/// Testing Task Lt Bool's
#[test]
fn test_bool() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let self_id = "test_bool";
    log::info!("{}", self_id);
    let mut target: bool;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("false", FnConfPointType::Bool, &cycle);
    let input2 = init_each("false", FnConfPointType::Bool, &cycle);
    let mut fn_lt = FnLt::new(
        self_id,
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        (00, false, false),
        (01, false, true),
        (02, true,  false),
        (03, true,  true),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        input2.borrow_mut().add(&point2);
        let result = fn_lt.out().unwrap().unwrap().into_value().as_bool().value.0;
        log::debug!("step {}  |  value1: {:?} < value2: {:?} | result: {:?}", step, value1, value2, result);
        target = value1 < value2;
        assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
}
///
/// Testing Task Lt Int's
#[test]
fn test_int() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let self_id = "test_int";
    log::info!("{}", self_id);
    let mut target: bool;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0", FnConfPointType::Int, &cycle);
    let input2 = init_each("0", FnConfPointType::Int, &cycle);
    let mut fn_lt = FnLt::new(
        self_id,
        vec![input1.clone(), input2.clone()],
    ).unwrap();
    let test_data = vec![
        (00, 1, 5),
        (01, 5, 1),
        (02, 3,  3),
        (03, -1,  -5),
        (04, -5,  -1),
        (05, -4,  -4),
        (06, 4,  0),
        (07, 0,  4),
        (08, 0,  0),
        (09, -4,  0),
        (10, 0,  -4),
    ];
    for (step, value1, value2) in test_data {
        cycle.increment();
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        input2.borrow_mut().add(&point2);
        let result = fn_lt.out().unwrap().unwrap().into_value().as_bool().value.0;
        log::debug!("step {}  |  value1: {:?} < value2: {:?} | result: {:?}", step, value1, value2, result);
        target = value1 < value2;
        assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
}
///
/// Testing Lt Real's
#[test]
fn test_real() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let self_id = "test_real";
    log::info!("{}", self_id);
    let mut target: bool;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0.0", FnConfPointType::Real, &cycle);
    let input2 = init_each("0.0", FnConfPointType::Real, &cycle);
    let mut fn_lt = FnLt::new(
        self_id,
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
        input2.borrow_mut().add(&point2);
        let result = fn_lt.out().unwrap().unwrap().into_value().as_bool().value.0;
        log::debug!("step {}  |  value1: {:?} < value2: {:?} | result: {:?}", step, value1, value2, result);
        target = value1 < value2;
        assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
}
///
/// Testing Lt Double's
#[test]
fn test_double() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let self_id = "test_double";
    log::info!("{}", self_id);
    let mut target: bool;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each("0.0", FnConfPointType::Double, &cycle);
    let input2 = init_each("0.0", FnConfPointType::Double, &cycle);
    let mut fn_lt = FnLt::new(
        self_id,
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
        input2.borrow_mut().add(&point2);
        let result = fn_lt.out().unwrap().unwrap().into_value().as_bool().value.0;
        log::debug!("step {}  |  value1: {:?} < value2: {:?} | result: {:?}", step, value1, value2, result);
        target = value1 < value2;
        assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
}
