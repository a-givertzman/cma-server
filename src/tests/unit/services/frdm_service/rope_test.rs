use std::sync::Arc;
#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use frdm_tools::{arena::{ChannelPacketSize, Exposure, ExposureAuto, FrameRate, PixelFormat}, camera::{CameraConf, CameraResolution}, conf::{Conf, DetectingContoursConf, FastScanConf, FineScanConf}};
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfDistanceUnit}, entity::Name, LinkName};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

use crate::{domain::RwLock, services::{DefectDetectionConf, Rope, TablesConf}};
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
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("RopePos-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let conf = DefectDetectionConf::new(
        &dbg,
        LinkName::new("MultiQueue", "in-queue"),
        TablesConf {
            defect: "".to_owned(),
            defect_image: "".to_owned(),
            deprecation: "".to_owned(),
        },
        0,
        ConfDistance::new(0.0, ConfDistanceUnit::Millimeter),
        CameraConf {
            name: Name::new(&dbg, ""),
            fps: FrameRate::Min,
            resolution: CameraResolution { width: 1920, height: 1200 },
            index: None,
            address: None,
            pixel_format: PixelFormat::Mono8,
            exposure: Exposure { auto: ExposureAuto::Continuous, time: 0.0 },
            auto_packet_size: false,
            channel_packet_size: ChannelPacketSize::Min,
            resend_packet: false,
        },
        Conf {
            segment: ConfDistance::new(100.0, ConfDistanceUnit::Millimeter),
            segment_threshold: ConfDistance::new(5.0, ConfDistanceUnit::Millimeter),
            detecting_contours: DetectingContoursConf::default(),
            fast_scan: FastScanConf::default(),
            fine_scan: FineScanConf::default(),
        },
    );
    let pos = Arc::new(RwLock::new(None::<f64>));
    let rope = Rope::new(&dbg, conf, pos.clone());
    let test_data = [
        //       rope-pos(m)
        (01,     0.000,          Some(0)),
        (02,     0.001,          Some(0)),
        (03,     0.004,          Some(0)),
        (04,     0.005,          None),
        (05,     0.010,          None),
        (06,     0.050,          None),
        (10,     0.095,          None),
        (11,     0.096,          Some(1)),
        (12,     0.100,          Some(1)),
        (13,     0.101,          Some(1)),
        (14,     0.104,          Some(1)),
        (15,     0.105,          Some(1)),
        (16,     0.106,          None),
        (17,     0.195,          None),
        (18,     0.196,          Some(2)),
        (19,     0.200,          Some(2)),
    ];
    for (step, rope_pos, segment_index) in test_data {
        pos.write().replace(rope_pos);
        let result = rope.pos();
        let target = Some(rope_pos);
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        let result = rope.segment_index();
        let target = segment_index;
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
