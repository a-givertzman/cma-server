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
    Rc::new(RefCell::new(
        FnInput::new("test", 0, &mut conf)
    ))
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
    let test_data = vec![
        (100,
        vec![
            //   duration    value      target
            //   before, ms
            (01,  100,      false,     false),
            (02,  001,      true,     false),
            (03,  050,      true,      false),
            (04,  001,      false,     false),
            (05,  001,      true,     false),
            (06,  150,      true,     true),
            (07,  001,      false,     false),
            (08,  001,      true,     false),
            (09,  090,      true,     false),
            (10,  050,      true,     true),
            (11,  001,      false,     false),
            (12,  001,      true,     false),
            (13,  200,      true,     true),
            (14,  010,      false,     false),
            (15,  100,      false,     false),
        ]),
        (250,
        vec![
            //   duration    value      target
            //   before, ms
            (21,  100,      false,     false),
            (22,  001,      true,      false),
            (23,  200,      true,      false),
            (24,  001,      false,     false),
            (25,  001,      true,     false),
            (26,  100,      true,     false),
            (27,  160,      true,     true),
            (28,  001,      false,     false),
            (29,  001,      true,     false),
            (30,  500,      true,     true),
            (31,  100,      false,     false),
            (32,  100,      false,     false),
            (33,  100,      false,     false),
            (34,  100,      false,     false),
            (35,  100,      false,     false),
        ]),
        (500,
        vec![
            //   duration    value      target
            //   before, ms
            (41,  100,      false,     false),
            (42,  001,      true,      false),
            (43,  300,      true,      false),
            (44,  001,      false,     false),
            (45,  001,      true,     false),
            (46,  250,      true,     false),
            (47,  260,      true,     true),
            (48,  001,      false,     false),
            (49,  001,      true,     false),
            (50,  510,      true,     true),
            (51,  100,      false,     false),
            (52,  100,      false,     false),
            (53,  100,      false,     false),
            (54,  100,      false,     false),
            (55,  100,      false,     false),
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
