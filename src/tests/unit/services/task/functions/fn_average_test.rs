#[cfg(test)]
use testing::entities::test_value::Value;
use sal_sync::{math::AproxEq, services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}}};
use std::{cell::RefCell, rc::Rc, sync::Once};
use debugging::session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef,
    services::task::{EvalCycle, EvalCycleRef, FlowContext, FnAverage, FnFlow, FnInput, FnOut}
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    // Формат: (step, input_value, target_average, target_is_new)
    let test_data = vec![
        (00,    0i64,  0i64,  true),  // Холодный старт -> New
        (01,    0,     0,     false), // Среднее не изменилось -> Old
        (02,    3,     1,     true),  // 3/3 = 1 -> New
        (03,    0,     1,     false), // 3/4 = 0.75 (округление 1). Старое 1 -> Old
        (04,    0,     1,     false), // 3/5 = 0.60 (округление 1). -> Old
        (05,    1,     1,     false),
        (06,    0,     1,     false),
        (07,    7,     1,     false), // 11/8 = 1.375 (округление 1). -> Old
        (08,    0,     1,     false),
        (09,    0,     1,     false),
        (10,    2,     1,     false),
        (11,    8,     2,     true),  // 21/12 = 1.75 (округление 2). Изменилось -> New!
        (12,    1,     2,     false),
        (13,    0,     2,     false),
        (14,    0,     1,     true),  // 22/15 = 1.46 (округление 1). Изменилось -> New!
    ];
    for (step, value, target, target_is_new) in test_data {
        let mut flow = FlowContext::new();
        cycle.increment();
        let point = value.to_point(0, "input");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let result = flow.map(fn_average.out()).unwrap().unwrap();
        let flow_is_new = flow.is_new();
        // debug!("input: {:?}", &mut input);
        log::debug!("{dbg} | Step {step} | \t value: {:?}   |   result: {:?}", value, result.value());
        assert!(result.as_int().value == target, "{dbg} | Step {step} | \nresult: {:?}\ntarget: {:?}", result, target);
        assert_eq!(flow_is_new, target_is_new, "{dbg} | Step {step} | Taint tracking mismatch");
    }
}
///
///
#[test]
fn test_real() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
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
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
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
        (00,    true,  0.0,      Some(0.00)),
        (01,    true,  0.0,      Some(0.00)),
        (02,    true,  3.3,      Some(3.30)),
        (03,    false,  0.1,     Some(1.70)),
        (04,    false,  0.0,     Some(1.133333333333333)),
        (05,    false,  1.6,     Some(1.25)),
        (06,    false,  0.0,     Some(1.00)),
        (07,    false,  7.2,     Some(2.033333333333333)),
        (08,    false,  0.0,     Some(1.742857142857143)),
        (09,    false,  0.3,     Some(1.5625)),
        (10,    false,  2.2,     Some(1.633333333333333)),
        (11,    true,  8.1,     Some(8.1)),
        (12,    true,  1.9,     Some(1.9)),
        (13,    true,  0.1,     Some(0.1)),
        (14,    true,  0.0,     Some(0.0)),
        (15,    false,  0.1,     Some(0.05)),
        (16,    false,  0.0,     Some(0.03333333333333)),
        (17,    false,  1.6,     Some(0.425)),
        (18,    false,  0.0,     Some(0.340)),
        (19,    false,  7.2,     Some(1.48333333333333)),
        (20,    false,  0.0,     Some(1.271428571428571)),
        (21,    false,  0.3,     Some(1.15)),
        (22,    false,  2.2,     Some(1.266666666666667)),
        (23,    true,  0.0,     Some(0.0)),
        (24,    true,  0.0,     Some(0.0)),
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
                log::debug!("Step {step} \t value: {:?}   |   result: {:?}", value, result);
                assert!(result.as_double().value.aprox_eq(*target, 6), "Step {step} | \nresult: {:?}\ntarget: {:?}", result.as_double().value, target);
                results += 1;
            }
            (Ok(None), None) => {
                // log::debug!("step {} \t enable: {:?}  |  value: {:?}  |  result: {:?}", step, reset, value, result);
                results += 1;
            }
            (Err(err), _) => panic!("Step {step} \t value: {:?}   |   Error: {:?}", value, err),
            _ => panic!("step {step} | \nresult: {:?}\ntarget: {:?}", result, target),
        };
    }
    assert!(results == test_data.len(), "\nresult: {:?}\ntarget: {:?}", results, test_data.len());
}
///
/// Проверка поведения при отсутствии входных данных (обрыв связи)
#[test]
fn test_disconnect() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    let dbg = "FnAverage-test_disconnect";
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each(&dbg, Value::Double(0.0), &cycle);
    let mut fn_average = FnAverage::new(dbg, None, input.clone());
    // Такт 1: Нормальные данные
    cycle.increment();
    input.borrow_mut().add(&10.0.to_point(0, "input"));
    let result = fn_average.out().unwrap();
    assert!(matches!(result, Some(FnFlow::New(_))), "Должен вернуть новое посчитанное значение: \nresult: {:?}\ntarget: Some(New(_))", result);
    // Такт 2: Источник замолчал (None)
    cycle.increment();
    // Мы не вызываем input.add(), имитируя отсутствие данных в цикле опроса
    let result = fn_average.out().unwrap();
    assert!(matches!(result, Some(FnFlow::Old(_))), "Вход не поменялся (Old) узел должен вернуть новое посчитанное значение, но так как результат расчета прежний, то Old: \nresult: {:?}\ntarget: Some(New(_))", result);
}
