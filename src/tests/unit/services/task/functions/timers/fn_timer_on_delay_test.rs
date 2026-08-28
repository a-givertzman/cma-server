use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDuration, ConfDurationUnit};
#[cfg(test)]

use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, time::Duration, rc::Rc, cell::RefCell};
use debugging::session::{DebugSession, LogLevel};
use crate::{
     domain::FnInOutRef, services::task::{EvalCycle, EvalCycleRef, FnInput, FnOut, FnTimerOnDelay},
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
/// `Task` `FnTimerOnDelay` | measuring simple elapsed seconds
#[test]
fn elapsed() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    let dbg = Dbg::own("FnTimerOnDelay-test");
    log::info!("{dbg}");
    let cycle = Rc::new(EvalCycle::new());
    let input = init_each("false", FnConfPointType::Bool, &cycle);
    let test_data = vec![
        (10,
        vec![
            //   duration    value      target
            //   before, ms
            (01,  10,      false,     false),
            (02,  01,      true,     false),
            (03,  05,      true,      false),
            (04,  01,      false,     false),
            (05,  01,      true,     false),
            (06,  15,      true,     true),
            (07,  01,      false,     false),
            (08,  01,      true,     false),
            (09,  09,      true,     false),
            (10,  05,      true,     true),
            (11,  01,      false,     false),
            (12,  01,      true,     false),
            (13,  20,      true,     true),
            (14,  01,      false,     false),
            (15,  10,      false,     false),
        ]),
        (25,
        vec![
            //   duration    value      target
            //   before, ms
            (21,  10,      false,     false),
            (22,  01,      true,      false),
            (23,  20,      true,      false),
            (24,  01,      false,     false),
            (25,  01,      true,     false),
            (26,  10,      true,     false),
            (27,  16,      true,     true),
            (28,  01,      false,     false),
            (29,  01,      true,     false),
            (30,  50,      true,     true),
            (31,  10,      false,     false),
            (32,  10,      false,     false),
            (33,  10,      false,     false),
            (34,  10,      false,     false),
            (35,  10,      false,     false),
        ]),
        (50,
        vec![
            //   duration    value      target
            //   before, ms
            (41,  10,      false,     false),
            (42,  01,      true,      false),
            (43,  30,      true,      false),
            (44,  01,      false,     false),
            (45,  01,      true,     false),
            (46,  25,      true,     false),
            (47,  26,      true,     true),
            (48,  01,      false,     false),
            (49,  01,      true,     false),
            (50,  51,      true,     true),
            (51,  10,      false,     false),
            (52,  10,      false,     false),
            (53,  10,      false,     false),
            (54,  10,      false,     false),
            (55,  10,      false,     false),
        ]),
    ];
    for (delay, test_data) in test_data {
        let mut fn_timer = FnTimerOnDelay::new(
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
