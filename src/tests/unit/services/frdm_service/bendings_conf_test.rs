#[cfg(test)]

use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfTree;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::BendingsConf;

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
fn init_each() -> () {}
///
/// Testing such functionality / behavior
#[test]
fn new() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("new");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01,
            serde_yaml::from_str(r"
                bendings:
                    - 5.0 .. 5.15 m
                    - 7.23 .. 7.30 mm
            ").unwrap(),
            vec![
                5.0..5.15,
                7.23*0.001..7.3*0.001,
            ]
        ),
        (02,
            serde_yaml::from_str(r"
                bendings:
                    - -5.0..-5.15m
                    - -7.23..-7.30km
            ").unwrap(),
            vec![
                -5.0..-5.15,
                -7.23*1000.0..-7.3*1000.0,
            ]
        ),
    ];
    for (step, conf, target) in test_data {
        let result = BendingsConf::new(&dbg, ConfTree::new("bindings", conf));
        let result = result.bendings;
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
