use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDuration, ConfDurationUnit};
#[cfg(test)]

use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, time::Duration, rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
     domain::FnInOutRef, services::task::{FnInput, FnOut, FnTimerOffDelay},
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
fn init_each(default: &str, type_: FnConfPointType) -> FnInOutRef {
    let mut conf = FnConfig { name: "test".to_owned(), type_, options: FnConfOptions {default: Some(default.into()), ..Default::default()}, ..Default::default()};
    Rc::new(RefCell::new(
        FnInput::new("test", 0, &mut conf)
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
    let input = init_each("false", FnConfPointType::Bool);
    let test_data = vec![
        (100,
        vec![
            //   duration    value      target
            //   before, ms
            (01,  001,      false,     false),
            (02,  001,      true,      true),
            (03,  001,      false,     true),
            (04,  050,      false,     true),
            (05,  001,      true,      true),
            (06,  001,      false,     true),
            (07,  150,      false,     false),
            (08,  001,      true,      true),
            (09,  001,      false,     true),
            (10,  090,      false,     true),
            (11,  050,      false,     false),
            (12,  001,      true,      true),
            (13,  001,      false,     true),
            (14,  200,      false,     false),
            (15,  010,      true,     true),
            (16,  100,      true,     true),
        ]),
        (250,
        vec![
            //   duration    value      target
            //   before, ms
            (21,  001,      false,     false),
            (22,  001,      true,     true),
            (23,  001,      false,      true),
            (24,  200,      false,      true),
            (25,  001,      true,     true),
            (26,  001,      false,     true),
            (27,  100,      false,     true),
            (28,  160,      false,     false),
            (29,  001,      true,     true),
            (30,  001,      false,     true),
            (31,  500,      false,     false),
            (32,  100,      true,     true),
            (33,  100,      true,     true),
        ]),
        (500,
        vec![
            //   duration    value      target
            //   before, ms
            (21,  001,      false,     false),
            (22,  001,      true,     true),
            (23,  001,      false,      true),
            (24,  300,      false,      true),
            (25,  001,      true,     true),
            (26,  001,      false,     true),
            (27,  200,      false,     true),
            (28,  360,      false,     false),
            (29,  001,      true,     true),
            (30,  001,      false,     true),
            (31,  510,      false,     false),
            (32,  100,      true,     true),
            (33,  100,      true,     true),
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
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            std::thread::sleep(Duration::from_millis(before));
            let result = fn_timer.out().unwrap().as_bool().value.0;
            // debug!("input: {:?}", &mut input);
            log::debug!("{step} | input: {:?}   |   result: {:?}", value, result);
            assert!(result == target, "{dbg} | step {step} \nresult: {} \ntarget: {}", result, target);
        }
    }
}
