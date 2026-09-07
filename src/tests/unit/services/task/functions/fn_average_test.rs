#[cfg(test)]
use testing::entities::test_value::Value;
use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{cell::RefCell, rc::Rc, sync::Once, time::Duration};
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
    let must = true; // Обязательно
    let opt = false;  // Может играть
    let test_data = vec![
        (00,    false,  0,  must), // холодны старт
        (01,    true,   0,  must), // 0 / 1 => 0
        (02,    true,   1,  opt),  // 1 / 2 => 0.5
        (03,    false,  1,  must), // 2 / 3 => 0.7
        (04,    false,  1,  opt),  // 2 / 4 => 0.5
        (05,    true,   0,  must), // 2 / 5 => 0.4
        (06,    true,   1,  opt),  // 3 / 6 => 0.5
        (07,    false,  1,  must), // 4 / 7 => 0.57
        (08,    false,  1,  opt),  // 4 / 8 => 0.5
        (09,    false,  0,  must), // 4 / 9 => 0.44
    ];
    let flow = FlowContext::new();
    for (step, value, target, must) in &test_data {
        cycle.increment();
        let point = value.to_point(0, "input");
        input.borrow_mut().add(&point);
        let result = flow.ignore(fn_average.out()).unwrap().unwrap();
        log::debug!("{dbg} | Step {step} | input: {:?} => result: {:?}", value, result);
        if *must {
            assert!(result.as_int().value == *target, "{dbg} | Step {step} | \nresult: {:?}\ntarget: {:?}", result, target);
        }
        std::thread::sleep(Duration::from_millis(50));
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
        (00,    0i64,  0i64,  true),   // холодный старт: total_t == 0 -> average = value -> New
        (01,    0,     0,     false),  // 0/1 = 0 -> Old
        (02,    3,     0,     false),  // 0/2 = 0 -> Old (3 еще не вошла в sum)
        (03,    0,     1,     true),   // 3/3 = 1.0 -> 1 -> New
        (04,    0,     1,     false),  // 3/4 = 0.75 -> 1
        (05,    2,     1,     false),  // 3/5 = 0.6 -> 1
        (06,    0,     1,     false),  // 5/6 = 0.83 -> 1
        (07,    1,     1,     false),  // 5/7 = 0.71 -> 1
        (08,    0,     1,     false),  // 6/8 = 0.75 -> 1
        (09,    0,     1,     false),  // 6/9 = 0.67 -> 1
        (10,    2,     1,     false),  // 6/10 = 0.6 -> 1 (см. примечание ниже)
        (11,    8,     1,     false),  // 8/11 = 0.73 -> 1
        (12,    1,     1,     false),  // 16/12 = 1.33 -> 1
        (13,    0,     1,     false),  // 17/13 = 1.31 -> 1
        (14,    0,     1,     false),  // 17/14 = 1.21 -> 1
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
        std::thread::sleep(Duration::from_millis(50));
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
        //      value       time       target
        //                  (millis)
        (00,    0.0f32,     000,       0.0),      // холодный старт: total_t == 0 -> value
        (01,    0.0,        000,       0.0),      // dt ~ 0 (sleep еще не выполнялся)
        (02,    3.3,        050,       0.0),      // dt ~ 0, sum ~ 0
        (03,    0.1,        050,       3.3),      // 3.3*0.05/0.05
        (04,    0.0,        050,       1.7),      // (3.3+0.1)/2
        (05,    1.6,        050,       1.13333),  // 3.4/3
        (06,    0.0,        050,       1.25),     // 5.0/4
        (07,    7.2,        050,       1.0),      // 5.0/5
        (08,    0.0,        050,       2.0333),   // 12.2/6
        (09,    0.3,        050,       1.74286),  // 12.2/7
        (10,    2.2,        050,       1.5625),   // 12.5/8
        (11,    8.1,        050,       1.63333),  // 14.7/9
        (12,    1.9,        050,       2.28),     // 22.8/10
        (13,    0.1,        050,       2.24545),  // 24.7/11
        (14,    0.0,        050,       2.06667),  // 24.8/12
    ];
    for (step, value, t, target) in test_data {
        cycle.increment();
        let point = value.to_point(0, "input");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let result = fn_average.out().unwrap().unwrap().into_value();
        // debug!("input: {:?}", &mut input);
        log::debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result.value());
        assert!((result.as_real().value - target as f32) < 0.05, "step {step} \nresult: {:?}\ntarget: {:?}", result, target);
        if t > 0 { std::thread::sleep(Duration::from_millis(t)); }
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
        (00,    true,   0.0,      Some(0.0)),
        (01,    true,   0.0,      Some(0.0)),
        (02,    true,   3.3,      Some(3.3)),
        (03,    false,  0.1,      Some(3.3)),         // окно [3.3]
        (04,    false,  0.0,      Some(1.7)),         // (3.3+0.1)/2
        (05,    false,  1.6,      Some(1.13333)),     // (3.3+0.1+0)/3
        (06,    false,  0.0,      Some(1.25)),        // 5/4
        (07,    false,  7.2,      Some(1.0)),         // 5/5
        (08,    false,  0.0,      Some(2.0333)),      // 12.2/6
        (09,    false,  0.3,      Some(1.74286)),     // 12.2/7
        (10,    false,  2.2,      Some(1.5625)),      // 12.5/8
        (11,    true,   8.1,      Some(8.1)),         // сброс -> value
        (12,    true,   1.9,      Some(1.9)),         // сброс
        (13,    true,   0.1,      Some(0.1)),         // сброс
        (14,    true,   0.0,      Some(0.0)),         // сброс
        (15,    false,  0.1,      Some(0.0)),         // окно [0.0]
        (16,    false,  0.0,      Some(0.05)),        // (0+0.1)/2
        (17,    false,  1.6,      Some(0.03333)),     // 0.1/3
        (18,    false,  0.0,      Some(0.425)),       // 1.7/4
        (19,    false,  7.2,      Some(0.34)),        // 1.7/5
        (20,    false,  0.0,      Some(1.48333)),     // 8.9/6
        (21,    false,  0.3,      Some(1.27143)),     // 8.9/7
        (22,    false,  2.2,      Some(1.15)),        // 9.2/8
        (23,    true,   0.0,      Some(0.0)),         // сброс
        (24,    true,   0.0,      Some(0.0)),         // сброс
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
                assert!((result.as_double().value - *target).abs() < 0.05, "Step {step} | \nresult: {:?}\ntarget: {:?}", result.as_double().value, target);
                results += 1;
            }
            (Ok(None), None) => {
                // log::debug!("step {} \t enable: {:?}  |  value: {:?}  |  result: {:?}", step, reset, value, result);
                results += 1;
            }
            (Err(err), _) => panic!("Step {step} \t value: {:?}   |   Error: {:?}", value, err),
            _ => panic!("step {step} | \nresult: {:?}\ntarget: {:?}", result, target),
        };
        std::thread::sleep(Duration::from_millis(50));
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
