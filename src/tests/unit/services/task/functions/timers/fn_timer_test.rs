#[cfg(test)]

use sal_sync::{math::AproxEq, services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}}};
use std::{sync::Once, time::{Instant, Duration}, thread,rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
     domain::FnInOutRef, services::task::{EvalCycle, EvalCycleRef, FnFlow, FnInput, FnOut, FnTimer},
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
/// Testing Task FnTimer measuring simple elapsed
#[test]
fn total_elapsed() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnTimer-test_total_elapsed";
    log::info!("{dbg}");
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each("false", FnConfPointType::Bool, &cycle);
    let mut fn_timer = FnTimer::new(
        dbg,
        None,
        None,
        input.clone(),
    );
    let test_data = vec![
        (00, false, 0),
        (01, false, 0),
        (02, true, 1),
        (03, true, 1),
        (04, false, 1),
        (05, false, 1),
        (06, true, 2),
        (07, false, 2),
        (08, true, 3),
        (09, false, 3),
        (10, false, 3),
        (11, true, 4),
        (12, true, 4),
        (13, false, 4),
        (14, false, 4),
    ];
    let mut start: Option<Instant> = None;
    let mut target: f64;
    let mut elapsed: f64 = 0.0;
    let mut elapsed_total: f64 = 0.0;
    for (step, value, _) in test_data {
        cycle.increment();
        if value {
            if start.is_none() {
                start = Some(Instant::now());
            } else {
                elapsed = start.unwrap().elapsed().as_secs_f64();
            }
        } else {
            if start.is_some() {
                elapsed_total += start.unwrap().elapsed().as_secs_f64();
                elapsed = 0.0;
                start = None;
            }
        }
        target = elapsed_total + elapsed;
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let fn_timer_elapsed = fn_timer.out().unwrap().unwrap().into_value().as_double().value;
        // debug!("input: {:?}", &mut input);
        log::debug!("{dbg} | step {step}:  value: {:?}   |   state: {:?}", value, fn_timer_elapsed);
        assert!(fn_timer_elapsed.aprox_eq(target, 2), "{dbg} | step {step}:  \n current '{}' \n target '{}'", fn_timer_elapsed, target);
        thread::sleep(Duration::from_millis(1));
    }
}
///
/// Testing Task FnTimer elapsed having reset
#[test]
fn elapsed_reset() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnTimer-test_elapsed_reset";
    log::info!("{dbg}");
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each("false", FnConfPointType::Bool, &cycle);
    // let initial = init_each("0.0", FnConfPointType::Double, &cycle);
    let reset = init_each("false", FnConfPointType::Bool, &cycle);
    let mut fn_timer = FnTimer::new(
        dbg,
        None, //Some(initial),
        Some(reset.clone()),
        input.clone(),
    );
    let is_new = true; let is_old = false;
    let test_data = vec![
        (00, false, is_old, false),
        (01, false, is_old, false),
        (02, true,  is_old, false),
        (03, false, is_new, false),
        (04, false, is_old, false),
        (05, true,  is_old, false),
        (06, true,  is_new, false),
        (07, true,  is_new, true),
        (08, true,  is_old, false),
        (09, false, is_new, false),
        (10, true,  is_old, false),
        (11, false, is_new, false),
        (12, false, is_old, false),
        (13, true,  is_old, false),
        (14, true,  is_new, false),
        (15, false, is_new, false),
        (16, false, is_old, false),
        (17, true,  is_new, true),
        (18, true,  is_old, false),
        (19, false, is_new, false),
        (20, false, is_old, false),
        (21, false, is_new, true),
        (22, false, is_old, false),
    ];
    let mut start: Option<Instant> = None;
    let mut target: f64;
    let mut elapsed: f64 = 0.0;
    let mut elapsed_total: f64 = 0.0;
    for (step, value, flow, rst) in test_data {
        cycle.increment();
        if rst {
            start = None;
            elapsed = 0.0;
            elapsed_total = 0.0;
        }
        if value && !rst {
            if start.is_none() {
                start = Some(Instant::now());
            } else {
                elapsed = start.unwrap().elapsed().as_secs_f64();
            }
        } else {
            if start.is_some() {
                elapsed = 0.0;
                elapsed_total += start.unwrap().elapsed().as_secs_f64();
                start = None;
            }
        }
        target = elapsed_total + elapsed;
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        reset.borrow_mut().add(&rst.to_point(0, "reset"));
        // debug!("input: {:?}", &input);
        let fn_timer_result = fn_timer.out().unwrap().unwrap();
        if flow {
            assert!(matches!(fn_timer_result, FnFlow::New(_)), "{dbg} | step {step}:  \n current {:?} \n target FnFlow::New(_)", fn_timer_result);
        } else {
            assert!(matches!(fn_timer_result, FnFlow::Old(_)), "{dbg} | step {step}:  \n current {:?} \n target FnFlow::Old(_)", fn_timer_result);
        }
        let fn_timer_elapsed = fn_timer_result.into_value().as_double().value;
        // debug!("input: {:?}", &mut input);
        log::debug!("{dbg} | step {step}:  value: {:?}   |   state: {:?}", value, fn_timer_elapsed);
        assert!(fn_timer_elapsed.aprox_eq(target, 2), "{dbg} | step {step}: \n current '{}' \n target '{}'", fn_timer_elapsed, target);
        thread::sleep(Duration::from_millis(1));
    }
}
///
/// Testing Task FnTimer with initial value
#[test]
fn initial() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "FnTimer-test_initial";
    log::info!("{dbg}");
    let initial = 123.1234f64;
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each("false", FnConfPointType::Bool, &cycle);
    let initial_input = init_each(initial.to_string().as_str(), FnConfPointType::Double, &cycle);
    let mut fn_timer = FnTimer::new(
        dbg,
        Some(initial_input),
        None,
        input.clone(),
    );
    let test_data = vec![
        (00, false),
        (01, false),
        (02, true),
        (03, false),
        (04, false),
        (05, true),
        (06, false),
        (07, true),
        (08, false),
        (09, false),
        (10, true),
        (11, true),
        (12, false),
        (13, false),
    ];
    let mut start: Option<Instant> = None;
    let mut target: f64;
    let mut elapsed: f64 = 0.0;
    let mut elapsed_total: f64 = initial;
    for (step, value) in test_data {
        cycle.increment();
        if value {
            if start.is_none() {
                start = Some(Instant::now());
            } else {
                elapsed = start.unwrap().elapsed().as_secs_f64();
            }
        } else {
            if start.is_some() {
                elapsed = 0.0;
                elapsed_total += start.unwrap().elapsed().as_secs_f64();
                start = None;
            }
        }
        target = elapsed_total + elapsed;
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let fn_timer_elapsed = fn_timer.out().unwrap().unwrap().into_value().as_double().value;
        // debug!("input: {:?}", &mut input);
        log::debug!("{dbg} | step {step}:  value: {:?}   |   state: {:?}", value, fn_timer_elapsed);
        assert!(fn_timer_elapsed.aprox_eq(target, 2), "{dbg} | step: {} | \n current '{}' \n target '{}'", step, fn_timer_elapsed, target);
        thread::sleep(Duration::from_millis(30));
    }
}
