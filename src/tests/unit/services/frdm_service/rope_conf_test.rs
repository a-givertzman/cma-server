#[cfg(test)]

use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDistance, ConfDistanceUnit, ConfTree};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::RopeConf;

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
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
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
                width: 35 mm
                length: 3000m
                winch-length: 2985m
                segment: 100 mm
                pos: point real 'Winch.EncoderBR2'      # in meters
                load: point real 'Winch.Load'             # in tonn
            ").unwrap(),
            RopeConf {
                width: ConfDistance::new(35.0, ConfDistanceUnit::Millimeter),
                length: ConfDistance::new(3000.0, ConfDistanceUnit::Meter),
                winch_len: ConfDistance::new(2985.0, ConfDistanceUnit::Meter),
                segment: ConfDistance::new(100.0, ConfDistanceUnit::Millimeter),
                pos: "Winch.EncoderBR2".to_owned(),
                load: "Winch.Load".to_owned(),
            }
        ),
    ];
    for (step, conf, target) in test_data {
        let result = RopeConf::new(&dbg, ConfTree::new("rope", conf));
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
