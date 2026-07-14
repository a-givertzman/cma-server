use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDuration, ConfDurationUnit};
#[cfg(test)]

use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, time::Duration, rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
     domain::FnInOutRef, services::task::{EvalCycle, EvalCycleRef, FnInput, FnOut, FnTimerOffDelay},
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
/// `Task` `FnTimerOffDelay` | measuring simple elapsed seconds
#[test]
fn elapsed() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = Dbg::own("FnTimerOffDelay-test");
    log::info!("{dbg}");
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each("false", FnConfPointType::Bool, &cycle);
    let test_data = vec![
        (10,
        vec![
            //   duration    value      target
            //   before, ms
            (01,  01,      false,     false),
            (02,  01,      true,      true),
            (03,  00,      false,     true),
            (04,  05,      false,     true),
            (05,  01,      true,      true),
            (06,  01,      false,     true),
            (07,  15,      false,     false),
            (08,  01,      true,      true),
            (09,  01,      false,     true),
            (10,  09,      false,     true),
            (11,  05,      false,     false),
            (12,  01,      true,      true),
            (13,  01,      false,     true),
            (14,  20,      false,     false),
            (15,  01,      true,     true),
            (16,  10,      true,     true),
        ]),
        (25,
        vec![
            //   duration    value      target
            //   before, ms
            (21,  01,      false,     false),
            (22,  01,      true,     true),
            (23,  01,      false,      true),
            (24,  20,      false,      true),
            (25,  01,      true,     true),
            (26,  01,      false,     true),
            (27,  10,      false,     true),
            (28,  16,      false,     false),
            (29,  01,      true,     true),
            (30,  01,      false,     true),
            (31,  50,      false,     false),
            (32,  10,      true,     true),
            (33,  10,      true,     true),
        ]),
        (50,
        vec![
            //   duration    value      target
            //   before, ms
            (21,  01,      false,     false),
            (22,  01,      true,     true),
            (23,  01,      false,      true),
            (24,  30,      false,      true),
            (25,  01,      true,     true),
            (26,  01,      false,     true),
            (27,  20,      false,     true),
            (28,  36,      false,     false),
            (29,  01,      true,     true),
            (30,  01,      false,     true),
            (31,  51,      false,     false),
            (32,  10,      true,     true),
            (33,  10,      true,     true),
        ]),
    ];
    for (delay, test_data) in test_data {
        let mut fn_timer = FnTimerOffDelay::new(
            &dbg,
            None,
            ConfDuration::new(delay, ConfDurationUnit::Millis),
            input.clone(),
        );
        for (step, before, value, target) in test_data {
            cycle.increment();
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            std::thread::sleep(Duration::from_millis(before));
            let result = fn_timer.out().unwrap().unwrap().into_value().as_bool().value.0;
            // debug!("input: {:?}", &mut input);
            log::debug!("{step} | input: {:?}   |   result: {:?}", value, result);
            assert!(result == target, "{dbg} | step {step} \nresult: {} \ntarget: {}", result, target);
        }
    }
}
