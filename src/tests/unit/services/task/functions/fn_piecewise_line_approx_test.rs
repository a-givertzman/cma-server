#[cfg(test)]
use sal_sync::{math::AproxEq, services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}}};
use std::{cell::{Cell, RefCell}, rc::Rc, sync::Once};
use debugging::session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef, 
    services::task::{EvalCycleRef, FnInput, FnOut, FnPiecewiseLineApprox, PiecewiseLinear},
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
fn init_each(default: &str, type_: FnConfPointType, cycle: &EvalCycleRef) -> FnInOutRef {
    let mut conf = FnConfig { name: "test".to_owned(), type_, options: FnConfOptions {default: Some(default.into()), ..Default::default()}, ..Default::default()};
    Rc::new(RefCell::new(
        FnInput::new("test", 0, &mut conf, cycle)
    ))
}
///
/// Testing FnPiecewiseLineApprox with Int's
#[test]
fn line_approx_int() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    let dbg = "line_approx_int";
    log::info!("{dbg}");
    let cycle = Rc::new(Cell::new(0));
    let input = init_each("0", FnConfPointType::Int, &cycle);
    let mut fn_line_approx = FnPiecewiseLineApprox::new(
        "test",
        input.clone(),
        PiecewiseLinear::from_yaml(dbg, &serde_yaml::from_str("
            0: 0
            5: 0
            10: 3
        ").unwrap()).unwrap(),
    );
    log::info!("fn: {:#?}", fn_line_approx);
    let test_data = vec![
        (00, -1, 0),
        (01, 0, 0),
        (02, 1, 0),
        (03, 2, 0),
        (04, 3, 0),
        (05, 4, 0),
        (06, 5, 0),
        (07, 6, 1),
        (08, 7, 1),
        (09, 8, 2),
        (10, 9, 2),
        (11, 10, 3),
        (12, 11, 3),
        (13, 12, 3),
        (14, 13, 3),
    ];
    for (step, value, target) in test_data {
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fn_line_approx.out().unwrap().unwrap().into_value();
        // debug!("input: {:?}", &mut input);
        log::debug!("{dbg} | value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state.as_int().value, target, "{dbg} | step: {}\n result: {:?} \ntarget: {}", step, state.as_int().value, target);
    }
}
///
/// Testing FnPiecewiseLineApprox with Real's
#[test]
fn line_approx_real() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    let dbg = "line_approx_real";
    log::info!("{dbg}");
    let cycle = Rc::new(Cell::new(0));
    let input = init_each("0.0", FnConfPointType::Real, &cycle);
    let mut fn_line_approx = FnPiecewiseLineApprox::new(
        "test",
        input.clone(),
        PiecewiseLinear::from_yaml(dbg, &serde_yaml::from_str("
            0: 0
            5: 0
            10: 3
            20: 1
        ").unwrap()).unwrap(),
    );
    log::info!("fn: {:#?}", fn_line_approx);
    let test_data = vec![
        (00, -1.0, 0.0),
        (01, 0.0, 0.0),
        (02, 1.0, 0.0),
        (03, 2.0, 0.0),
        (04, 3.0, 0.0),
        (05, 4.0, 0.0),
        (06, 5.0, 0.0),
        (07, 6.0, 0.6),
        (08, 7.0, 1.2),
        (09, 8.0, 1.8),
        (10, 9.0, 2.4),
        (11, 10.0, 3.0),
        (12, 11.0, 2.8),
        (13, 12.0, 2.6),
        (14, 13.0, 2.4),
    ];
    for (step, value, target) in test_data {
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fn_line_approx.out().unwrap().unwrap().into_value();
        // debug!("input: {:?}", &mut input);
        log::debug!("{dbg} | value: {:?}   |   state: {:?}", value, state);
        assert_eq!(state.as_real().value, target, "{dbg} | step: {}\n result: {:?} \ntarget: {}", step, state.as_real().value, target);
    }
}
///
/// Testing FnPiecewiseLineApprox with Double's
#[test]
fn line_approx_double() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    let dbg = "line_approx_double";
    log::info!("{dbg}");
    let cycle = Rc::new(Cell::new(0));
    let input = init_each("0.0", FnConfPointType::Double, &cycle);
    let mut fn_line_approx = FnPiecewiseLineApprox::new(
        "test",
        input.clone(),
        PiecewiseLinear::from_yaml(dbg, &serde_yaml::from_str(r"
            0: 0
            5: 0
            10: 3
            20: 1
        ").unwrap()).unwrap(),
    );
    log::info!("fn: {:#?}", fn_line_approx);
    let test_data = vec![
        (00, -1.0, 0.0),
        (01, 0.0, 0.0),
        (02, 1.0, 0.0),
        (03, 2.0, 0.0),
        (04, 3.0, 0.0),
        (05, 4.0, 0.0),
        (06, 5.0, 0.0),
        (07, 6.0, 0.6),
        (08, 7.0, 1.2),
        (09, 8.0, 1.8),
        (10, 9.0, 2.4),
        (11, 10.0, 3.0),
        (12, 11.0, 2.8),
        (13, 12.0, 2.6),
        (14, 13.0, 2.4),
    ];
    for (step, value, target) in test_data {
        let point = value.to_point(0, "test");
        input.borrow_mut().add(&point);
        // debug!("input: {:?}", &input);
        let state = fn_line_approx.out().unwrap().unwrap().into_value();
        // debug!("input: {:?}", &mut input);
        log::debug!("{dbg} | value: {:?}   |   state: {:?}", value, state);
        assert!(state.as_double().value.aprox_eq(target, 4), "{dbg} | step: {}\n result: {:?} \ntarget: {}", step, state.as_double().value, target);
    }
}
