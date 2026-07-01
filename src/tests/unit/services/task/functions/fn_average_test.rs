#[cfg(test)]
use testing::entities::test_value::Value;
use sal_sync::{math::AproxEq, services::{entity::{Point, ToPoint}, task::functions::{FnConfOptions, FnConfPointType, FnConfig}}};
use std::{cell::RefCell, rc::Rc, sync::Once};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef,
    services::task::{EvalCycle, EvalCycleRef, FlowContext, FnAverage, FnInput, FnOut}
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
fn init_each(parent: &str, initial: Value, cycle: &EvalCycleRef) -> FnInOutRef {
    let mut conf = FnConfig {
        name: "test".to_owned(),
        type_: match initial {
            Value::Bool(_) => FnConfPointType::Bool,
            Value::Int(_) => FnConfPointType::Int,
            Value::Real(_) => FnConfPointType::Real,
            Value::Double(_) => FnConfPointType::Double,
            Value::String(_) => FnConfPointType::String,
            Value::Bytes(_) => panic!("{parent} Initial of type 'Bytes' - is not supported"),
        },
        options: FnConfOptions {default: Some(match initial {
            Value::Bool(v) => v.to_string(),
            Value::Int(v) => v.to_string(),
            Value::Real(v) => v.to_string(),
            Value::Double(v) => v.to_string(),
            Value::String(v) => v.to_string(),
            Value::Bytes(v) => String::from_utf8_lossy(&v).into_owned(),
        }),
            ..Default::default()}, ..Default::default()
    };
    Rc::new(RefCell::new(
        FnInput::new(parent, 0, &mut conf, cycle)
    ))
}
///
///
#[test]
fn test_bool() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    let dbg = "FnAverage-test_bool";
    log::info!("{}", dbg);
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each(&dbg, Value::Bool(false), &cycle);
    let mut fn_average = FnAverage::new(
        dbg,
        None,
        input.clone(),
    );
    let test_data = vec![
        (00,    false),
        (01,    false),
        (02,    true),
        (03,    false),
        (04,    false),
        (05,    true),
    ];
    let flow = FlowContext::new();
    for (step, value) in &test_data {
        cycle.increment();
        let point = value.to_point(0, "input");
        input.borrow_mut().add(&point);
        let result = flow.ignore(fn_average.out());
        log::debug!("{dbg} | Step {step} | input: {:?} => result: {:?}", value, result);
        assert!(result.is_err(), "{dbg} | Step {step} | \nresult: {:?}\ntarget: Err(_)", result);
    }
}
///
///
#[test]
fn test_int() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    let dbg = "FnAverage-test_int";
    log::info!("{}", dbg);
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each(&dbg, Value::Int(0), &cycle);
    let mut fn_average = FnAverage::new(
        dbg,
        None,
        input.clone(),
    );
    let test_data = vec![
        (00,    0i64,     0i64),
        (01,    0,     0),
        (02,    3,     1),
        (03,    0,     1),
        (04,    0,     1),
        (05,    1,     1),
        (06,    0,     1),
        (07,    7,     1),
        (08,    0,     1),
        (09,    0,     1),
        (10,    2,     1),
        (11,    8,     2),
        (12,    1,     2),
        (13,    0,     2),
        (14,    0,     1),
    ];
    for (step, value, target) in test_data {
        cycle.increment();
        let point = value.to_point(0, "input");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let result = fn_average.out().unwrap().unwrap().into_value();
        // debug!("input: {:?}", &mut input);
        log::debug!("{dbg} | Step {step} | \t value: {:?}   |   result: {:?}", value, result);
        assert!(result.as_int().value == target, "{dbg} | Step {step} | \nresult: {:?}\ntarget: {:?}", result, target);
    }
}
///
///
#[test]
fn test_real() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAverage-test_real";
    log::info!("{}", dbg);
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each(&dbg, Value::Real(0.0), &cycle);
    let mut fn_average = FnAverage::new(
        dbg,
        None,
        input.clone(),
    );
    let test_data = vec![
        (00,    0.0f32,     0.0),
        (01,    0.0,     0.0),
        (02,    3.3,     1.09999),
        (03,    0.1,     0.84999),
        (04,    0.0,     0.67999),
        (05,    1.6,     0.83333),
        (06,    0.0,     0.71428),
        (07,    7.2,     1.52499),
        (08,    0.0,     1.35555),
        (09,    0.3,     1.24999),
        (10,    2.2,     1.33636),
        (11,    8.1,     1.9),
        (12,    1.9,     1.9),
        (13,    0.1,     1.77142),
        (14,    0.0,     1.65333),
    ];
    for (step, value, target) in test_data {
        cycle.increment();
        let point = value.to_point(0, "input");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let result = fn_average.out().unwrap().unwrap().into_value();
        // debug!("input: {:?}", &mut input);
        log::debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
        assert!(result.as_real().value.aprox_eq(target, 3), "\nresult: {:?}\ntarget: {:?}", result, target);
    }
}
///
/// Double points on input, enable - is variable during the test
#[test]
fn test_double_reset() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAverage-test_double_reset";
    log::info!("{}", dbg);
    let cycle = Rc::new(EvalCycle::new());
    let reset = init_each(&dbg, Value::Bool(false), &cycle);
    let input = init_each(&dbg, Value::Double(0.0), &cycle);
    let mut fn_average = FnAverage::new(
        dbg,
        Some(reset.clone()),
        input.clone(),
    );
    let test_data = vec![
        (00,    true,  0.0,      None),
        (01,    true,  0.0,      None),
        (02,    true,  3.3,      None),
        (03,    false,  0.1,     Some(0.1)),
        (04,    false,  0.0,     Some(0.05)),
        (05,    false,  1.6,     Some(0.566666666666667)),
        (06,    false,  0.0,     Some(0.425)),
        (07,    false,  7.2,     Some(1.78)),
        (08,    false,  0.0,     Some(1.48333333333333)),
        (09,    false,  0.3,     Some(1.31428571428571)),
        (10,    false,  2.2,     Some(1.425)),
        (11,    true,  8.1,     None),
        (12,    true,  1.9,     None),
        (13,    true,  0.1,     None),
        (14,    true,  0.0,     None),
        (15,    false,  0.1,     Some(0.1)),
        (16,    false,  0.0,     Some(0.05)),
        (17,    false,  1.6,     Some(0.566666666666667)),
        (18,    false,  0.0,     Some(0.425)),
        (19,    false,  7.2,     Some(1.78)),
        (20,    false,  0.0,     Some(1.48333333333333)),
        (21,    false,  0.3,     Some(1.31428571428571)),
        (22,    false,  2.2,     Some(1.425)),
        (23,    true,  0.0,     None),
        (24,    true,  0.0,     None),
    ];
    let mut results = 0;
    let flow = FlowContext::new();
    for (step, rst, value, target) in &test_data {
        cycle.increment();
        let rst = rst.to_point(0, "reset");
        let point = value.to_point(0, "input");
        reset.borrow_mut().add(&rst);
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let result = flow.ignore(fn_average.out());
        match (&result, &target) {
            (Ok(Some(result)), Some(target)) => {
                log::debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
                assert!(result.as_double().value.aprox_eq(*target, 3), "\nresult: {:?}\ntarget: {:?}", result.as_real().value, target);
                results += 1;
            }
            (Ok(None), None) => {
                // log::debug!("step {} \t enable: {:?}  |  value: {:?}  |  result: {:?}", step, reset, value, result);
                results += 1;
            }
            (Err(err), _) => panic!("step {} \t value: {:?}   |   Error: {:?}", step, value, err),
            _ => panic!("step {step} \nresult: {:?}\ntarget: {:?}", result, target),
        };
    }
    assert!(results == test_data.len(), "\nresult: {:?}\ntarget: {:?}", results, test_data.len());
}
