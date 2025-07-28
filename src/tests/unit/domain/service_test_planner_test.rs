#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
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
fn functionality() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("functionality");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01, 111, 112),
        (02, 222, 223),
        (03, 333, 334),
    ];
    for (step, val, target) in test_data {
        let result = val + 1;
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
