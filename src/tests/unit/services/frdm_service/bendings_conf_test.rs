#[cfg(test)]

use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDistance, ConfDistanceUnit};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::BendingsConf;

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
            vec![
                serde_yaml::from_str(r"D200mm 5.0 .. 5.15 m").unwrap(),
                serde_yaml::from_str(r"D0.2m 7.23 .. 7.30 mm").unwrap(),
            ],
            vec![
                (ConfDistance::new(200.0, ConfDistanceUnit::Millimeter), 5.0..5.15),
                (ConfDistance::new(0.200, ConfDistanceUnit::Meter), 7.23*0.001..7.3*0.001),
            ]
        ),
        (02,
            vec![
                serde_yaml::from_str(r"D150mm -5.0..-5.15m").unwrap(),
                serde_yaml::from_str(r"D170mm -7.23..-7.30km").unwrap(),
            ],
            vec![
                (ConfDistance::new(150.0, ConfDistanceUnit::Millimeter), -5.0..-5.15),
                (ConfDistance::new(170.0, ConfDistanceUnit::Millimeter), -7.23*1000.0..-7.3*1000.0),
            ]
        ),
    ];
    for (step, conf, target) in test_data {
        let result = BendingsConf::new(&dbg, conf);
        let result = result.bendings;
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
