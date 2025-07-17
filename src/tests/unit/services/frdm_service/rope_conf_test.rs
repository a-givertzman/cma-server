use std::str::FromStr;
#[cfg(test)]

use std::{sync::Once, time::Duration};
use indexmap::IndexMap;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfDistanceUnit, ConfTree}, task::functions::{FnConfKind, FnConfOptions, FnConfPointType, FnConfig}, LinkName};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::{BendingsConf, RopeConf};

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
                segment: 100 mm
                bendings:
                    - D200mm 5.0 .. 5.15 m
                    - D0.2m 7.23 .. 7.30 mm
                pos: point real '/App/Winch.EncoderBR2'      # in meters
                load: point real '/App/Winch.Load'             # in tonn
            ").unwrap(),
            RopeConf {
                width: ConfDistance::new(35.0, ConfDistanceUnit::Millimeter),
                length: ConfDistance::new(3000.0, ConfDistanceUnit::Meter),
                segment: ConfDistance::new(100.0, ConfDistanceUnit::Millimeter),
                bendings: BendingsConf {
                    bendings: vec![
                        (ConfDistance::new(200.0, ConfDistanceUnit::Millimeter), 5.0..5.15),
                        (ConfDistance::new(0.200, ConfDistanceUnit::Meter), 7.23*0.001..7.3*0.001),
                    ],
                },
                pos: LinkName::from_str("/App/Winch.EncoderBR2").unwrap(),
                load: LinkName::from_str("/App/Winch.Load").unwrap(),
            }
        ),
    ];
    for (step, conf, target) in test_data {
        let result = RopeConf::new(&dbg, ConfTree::new("rope", conf));
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
