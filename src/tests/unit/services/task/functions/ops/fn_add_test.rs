use sal_core::error::Error;
use sal_sync::services::entity::Point;
#[cfg(test)]
use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef, 
    services::task::{EvalCycle, EvalCycleRef, FlowContext, FnAdd, FnInput, FnOut}
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
fn init_each(default: Option<impl ToString>, type_: FnConfPointType, cycle: &EvalCycleRef) -> FnInOutRef {
    let mut conf = FnConfig { name: "test".to_owned(), type_, options: FnConfOptions {default: default.map(|d| d.to_string()), ..Default::default()}, ..Default::default()};
    Rc::new(RefCell::new(
        FnInput::new("test", 0, &mut conf, cycle)
    ))
}
/// Testing Add Overflow
#[test]
fn overflow() {
    DebugSession::new().filter(LogLevel::Info).init();
    let dbg = "FnAdd-overflow";
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each(Some(i64::MAX), FnConfPointType::Int, &cycle);
    let input2 = init_each(Some("1"), FnConfPointType::Int, &cycle);
    let mut fn_add = FnAdd::new(dbg, vec![input1.clone(), input2.clone()]).unwrap();
    let flow = FlowContext::new();
    cycle.increment();
    input1.borrow_mut().add(&i64::MAX.to_point(0, "test"));
    input2.borrow_mut().add(&1i64.to_point(0, "test"));
    let state = flow.ignore(fn_add.out());
    // Должен вернуть Err, а не паниковать и не уходить в отрицательный диапазон
    assert!(matches!(state, Err(_)), "Переполнение должно возвращать ошибку");
}
///
/// Testing Type Promotion
#[test]
fn promotion() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAdd-promotion";
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each(Some("0"), FnConfPointType::Int, &cycle);
    let input2 = init_each(Some("0.0"), FnConfPointType::Double, &cycle);
    let mut fn_add = FnAdd::new(dbg, vec![input1.clone(), input2.clone()]).unwrap();
    let flow = FlowContext::new();
    cycle.increment();
    input1.borrow_mut().add(&5i64.to_point(0, "test"));
    input2.borrow_mut().add(&2.5f64.to_point(0, "test"));
    let state = flow.ignore(fn_add.out()).unwrap().unwrap();
    // При сложении Int и Double результат обязан стать Double
    assert!(matches!(state, Point::Double(_)));
    assert_eq!(state.as_double().value, 7.5);
}
///
/// Testing NaN Guard
#[test]
fn nan_guard() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAdd-nan";
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each(Some("0.0"), FnConfPointType::Real, &cycle);
    let input2 = init_each(Some("0.0"), FnConfPointType::Real, &cycle);
    let mut fn_add = FnAdd::new(dbg, vec![input1.clone(), input2.clone()]).unwrap();
    let flow = FlowContext::new();
    cycle.increment();
    input1.borrow_mut().add(&1.0f32.to_point(0, "test"));
    input2.borrow_mut().add(&f32::NAN.to_point(0, "test"));
    let state = flow.ignore(fn_add.out());
    assert!(matches!(state, Err(_)), "Вредоносный NaN должен немедленно вызывать ошибку");
}
///
/// Testing Cold Mode
#[test]
fn cold_mode() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAdd-cold";
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each(None::<i64>, FnConfPointType::Int, &cycle);
    let input2 = init_each(None::<i64>, FnConfPointType::Int, &cycle);
    let mut fn_add = FnAdd::new(dbg, vec![input1.clone(), input2.clone()]).unwrap();
    // Сигнал не подан, входы вернут Ok(None)
    let state = fn_add.out();
    assert!(matches!(state, Ok(None)), "Узел должен спать при отсутствии сигнала \nresult: {:?} target: {:?}", state, Ok::<_, Error>(None::<i64>));
}
///
/// Testing Task Add Bool's
#[test]
fn bool() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAdd-bool";
    log::info!("{dbg}");
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each(Some("false"), FnConfPointType::Bool, &cycle);
    let input2 = init_each(Some("false"), FnConfPointType::Bool, &cycle);
    let mut fn_add = FnAdd::new(
        dbg,
        vec![
            input1.clone(),
            input2.clone(),
        ]
    ).unwrap();
    let flow = FlowContext::new();
    cycle.increment();
    let point = false.to_point(0, "test");
    input1.borrow_mut().add(&point);
    let state = flow.ignore(fn_add.out());
    log::debug!("{dbg} | value: {:?}   |   state: {:?}", point.value(), state);
    assert!(matches!(state, Err(_)));
    let point = true.to_point(0, "test");
    input2.borrow_mut().add(&point);
    let state = flow.ignore(fn_add.out());
    log::debug!("{dbg} | value2: {:?}   |   state: {:?}", point.value(), state);
    assert!(matches!(state, Err(_)));
}
///
/// Testing Task Add Int's
#[test]
fn int() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAdd-int";
    log::info!("{dbg}");
    let mut value1_stored;
    let mut value2_stored = 0.to_point(0, "int");
    let mut target: i64;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each(Some("0"), FnConfPointType::Int, &cycle);
    let input2 = init_each(Some("0"), FnConfPointType::Int, &cycle);
    let mut fn_add = FnAdd::new(
        dbg,
        vec![
            input1.clone(),
            input2.clone(),
        ]
    ).unwrap();
    let test_data = vec![
        (01, 1, 1, Ok(())),
        (02, 2, 2, Ok(())),
        (03, 5, 5, Ok(())),
        (04, -1, 1, Ok(())),
        (05, -5, 1, Ok(())),
        (06, 1, -1, Ok(())),
        (07, 1, -5, Ok(())),
        (08, 0, 0, Ok(())),
        (09, i64::MIN, 0, Ok(())),
        (10, 0, i64::MIN, Ok(())),
        (11, i64::MAX, 0, Ok(())),
        (12, 0, i64::MAX, Ok(())),
        (13, 1, i64::MAX, Err(())),
    ];
    let flow = FlowContext::new();
    for (step, value1, value2, target_kind) in test_data {
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        if target_kind.is_ok() {
            input1.borrow_mut().add(&point1);
            let state = flow.ignore(fn_add.out()).unwrap().unwrap();
            log::debug!("{dbg} | step {step}: value1: {:?}   |   state: {:?}", value1, state);
            value1_stored = point1.clone();
            target = value1_stored.as_int().value + value2_stored.as_int().value;
            let result = state.as_int().value;
            assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
            input2.borrow_mut().add(&point2);
            let state = flow.ignore(fn_add.out()).unwrap().unwrap();
            log::debug!("{dbg} | step {step}: value2: {:?}   |   state: {:?}", value2, state);
            value2_stored = point2.clone();
            target = value1_stored.as_int().value + value2_stored.as_int().value;
            let result = state.as_int().value;
            assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        } else {
            input1.borrow_mut().add(&point1);
            input2.borrow_mut().add(&point2);
            let result = flow.ignore(fn_add.out());
            assert!(matches!(result, Err(_)), "\n result: {:?} \n target: Err(_)", result);
        }
    }
}
///
/// Testing Add Real's
#[test]
fn real() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAdd-real";
    log::info!("dbg");
    let mut value1_stored;
    let mut value2_stored = 0.0f32.to_point(0, "real");
    let mut target: f32;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each(Some("0.0"), FnConfPointType::Real, &cycle);
    let input2 = init_each(Some("0.0"), FnConfPointType::Real, &cycle);
    let mut fn_mul = FnAdd::new(
        dbg,
        vec![
            input1.clone(),
            input2.clone(),
        ]
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
    let flow = FlowContext::new();
    for (step, value1, value2) in test_data {
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = flow.ignore(fn_mul.out()).unwrap().unwrap();
        log::debug!("{dbg} | step {}:  value1: {:?}   |   state: {:?}", step, value1, state);
        value1_stored = point1.clone();
        target = value1_stored.as_real().value + value2_stored.as_real().value;
        let result = state.as_real().value;
        assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        input2.borrow_mut().add(&point2);
        let state = flow.ignore(fn_mul.out()).unwrap().unwrap();
        log::debug!("{dbg} | step {}:  value2: {:?}   |   state: {:?}", step, value2, state);
        value2_stored = point2.clone();
        target = value1_stored.as_real().value + value2_stored.as_real().value;
        let result = state.as_real().value;
        assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
    }
}
///
/// Testing Add Double's
#[test]
fn double() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnAdd-double";
    log::info!("{dbg}");
    let mut value1_stored;
    let mut value2_stored = 0.0f64.to_point(0, "double");
    let mut target: f64;
    let cycle = Rc::new(EvalCycle::new());
    let input1 = init_each(Some("0.0"), FnConfPointType::Double, &cycle);
    let input2 = init_each(Some("0.0"), FnConfPointType::Double, &cycle);
    let mut fn_mul = FnAdd::new(
        dbg,
        vec![
            input1.clone(),
            input2.clone(),
        ]
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
    let flow = FlowContext::new();
    for (step, value1, value2) in test_data {
        let point1 = value1.to_point(0, "test");
        let point2 = value2.to_point(0, "test");
        input1.borrow_mut().add(&point1);
        let state = flow.ignore(fn_mul.out()).unwrap().unwrap();
        log::debug!("{dbg} | step {step}:  value1: {:?}   |   state: {:?}", value1, state);
        value1_stored = point1.clone();
        target = value1_stored.as_double().value + value2_stored.as_double().value;
        let result = state.as_double().value;
        assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
        input2.borrow_mut().add(&point2);
        let state = flow.ignore(fn_mul.out()).unwrap().unwrap();
        log::debug!("{dbg} | step {step}:  value2: {:?}   |   state: {:?}", value2, state);
        value2_stored = point2.clone();
        target = value1_stored.as_double().value + value2_stored.as_double().value;
        let result = state.as_double().value;
        assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
    }
}
