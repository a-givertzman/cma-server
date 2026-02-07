use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDuration, ConfDurationUnit};
#[cfg(test)]

use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use std::{sync::Once, time::Duration, rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
     domain::FnInOutRef, services::task::{FnInput, FnOut, FnTimerOnDelay},
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
    Rc::new(RefCell::new(Box::new(
        FnInput::new("test", 0, &mut conf)
    )))
}
///
/// `Task` `FnTimerOnDelay` | measuring simple elapsed seconds
#[test]
fn elapsed() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = Dbg::own("FnTimerOnDelay-test");
    log::info!("{dbg}");
    let input = init_each("false", FnConfPointType::Bool);
    let mut fn_timer = FnTimerOnDelay::new(
        &dbg,
        None,
        ConfDuration::new(250, ConfDurationUnit::Millis),
        input.clone(),
    );
    let test_data = vec![
        (100,
        vec![
            //   duration    value      target
            //   before, ms
            (01,  100,      false,     false),
            (02,  100,      false,     false),
            (03,  050,      true,      false),
            (04,  010,      false,     false),
            (05,  150,      true,     true),
            (06,  001,      false,     false),
            (07,  001,      false,     false),
            (08,  200,      true,     true),
            (09,  010,      false,     false),
            (10,  100,      false,     false),
            (11,  100,      false,     false),
            (12,  100,      false,     false),
            (13,  100,      false,     false),
            (14,  100,      false,     false),
            (15,  100,      false,     false),
        ]),
        (250,
        vec![
            //   duration    value      target
            //   before, ms
            (01,  100,      false,     false),
            (02,  100,      false,     false),
            (03,  200,      true,      false),
            (04,  010,      false,     false),
            (05,  100,      true,     false),
            (06,  160,      true,     true),
            (07,  001,      false,     false),
            (08,  500,      true,     true),
            (09,  010,      false,     false),
            (10,  100,      false,     false),
            (11,  100,      false,     false),
            (12,  100,      false,     false),
            (13,  100,      false,     false),
            (14,  100,      false,     false),
            (15,  100,      false,     false),
        ]),
        (500,
        vec![
            //   duration    value      target
            //   before, ms
            (01,  100,      false,     false),
            (02,  100,      false,     false),
            (03,  100,      true,      false),
            (04,  010,      false,     false),
            (05,  250,      true,     false),
            (06,  260,      true,     true),
            (07,  001,      false,     false),
            (08,  510,      true,     true),
            (09,  010,      false,     false),
            (10,  100,      false,     false),
            (11,  100,      false,     false),
            (12,  100,      false,     false),
            (13,  100,      false,     false),
            (14,  100,      false,     false),
            (15,  100,      false,     false),
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
