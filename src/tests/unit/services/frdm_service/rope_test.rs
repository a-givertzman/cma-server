#[cfg(test)]
use std::sync::Arc;
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDistance, ConfDistanceUnit};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{domain::RwLock, services::frdm_service::Rope};
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
/// Testing [Rope].pos()
#[test]
fn rope_pos() {
    DebugSession::new().filter(LogLevel::Trace).init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("RopePos-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let pos = Arc::new(RwLock::new(None::<f64>));
    // Camera position from the begin of the rope (hook side), meters
    let camera_offset = 3.5;
    let rope = Rope::new(
        &dbg,
        ConfDistance::new(camera_offset, ConfDistanceUnit::Meter),
        ConfDistance::new(100.0, ConfDistanceUnit::Millimeter),
        ConfDistance::new(5.0, ConfDistanceUnit::Millimeter),
        pos.clone(),
    );
    let test_data = [
        //       rope-pos(m)
        (01,     0.000,          Some(35)),
        (02,     0.001,          Some(35)),
        (03,     0.004,          Some(35)),
        (04,     0.005,          Some(35)),
        (05,     0.006,          None),
        (06,     0.050,          None),
        (06,     0.094,          None),
        (10,     0.095,          Some(36)),
        (11,     0.096,          Some(36)),
        (12,     0.100,          Some(36)),
        (13,     0.101,          Some(36)),
        (14,     0.104,          Some(36)),
        (15,     0.105,          Some(36)),
        (16,     0.106,          None),
        (16,     0.194,          None),
        (17,     0.195,          Some(37)),
        (18,     0.196,          Some(37)),
        (19,     0.200,          Some(37)),
    ];
    for (step, rope_pos, segment_index) in test_data {
        let time = Instant::now();
        pos.write().replace(rope_pos);
        // let result = rope.pos();
        // let target = Some(rope_pos + camera_offset);
        // log::debug!("{dbg} | step {step}  rope pos: {:?}", rope_pos);
        // assert!(result == target, "{dbg} | step {step} \nresult: {:?}\ntarget: {:?}", result, target);
        let result = rope.segment_index();
        let target = segment_index;
        assert!(result == target, "{dbg} | step {step} \nresult: {:?}\ntarget: {:?}", result, target);
        log::debug!("{dbg} | step {step}  elapsed: {:?}", time.elapsed());
    }
    test_duration.exit();
}
