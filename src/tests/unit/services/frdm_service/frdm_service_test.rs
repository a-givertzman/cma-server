use std::sync::Arc;
#[cfg(test)]

use std::{sync::Once, time::Duration};
use indexmap::IndexMap;
use sal_core::dbg::Dbg;
use sal_sync::{services::{conf::{ConfDistance, ConfDistanceUnit, ConfTree, ServicesConf}, task::functions::{FnConfKind, FnConfOptions, FnConfPointType, FnConfig}, Service, Services}, thread_pool::ThreadPool};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::{services::{FrdmService, FrdmServiceConf, RopeConf}, tests::unit::services::mock::mock_recv_service::MockRecvService};

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
/// Testing [FrdmService].run
#[test]
fn run() {
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
                pos: point real '/App/Winch.EncoderBR2'      # in meters
                load: point real '/App/Winch.Load'             # in tonn
            ").unwrap(),
            RopeConf {
                width: ConfDistance::new(35.0, ConfDistanceUnit::Millimeter),
                length: ConfDistance::new(3000.0, ConfDistanceUnit::Meter),
                segment: ConfDistance::new(100.0, ConfDistanceUnit::Millimeter),
                pos: FnConfKind::Point(
                    FnConfig {
                        name: format!("/App/Winch.EncoderBR2"),
                        inputs: IndexMap::new(),
                        type_: FnConfPointType::Real,
                        options: FnConfOptions::default(),
                    }
                ),
                load: FnConfKind::Point(
                    FnConfig {
                        name: format!("/App/Winch.Load"),
                        inputs: IndexMap::new(),
                        type_: FnConfPointType::Real,
                        options: FnConfOptions::default(),
                    },
                )
            }
        ),
    ];
    for (step, conf, target) in test_data {
        let result = RopeConf::new(&dbg, ConfTree::new("rope", conf));
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    let conf = FrdmServiceConf::from_yaml(&dbg,
        &serde_yaml::from_str(&format!(r"
            service FrdmService:
                cycle: 100 ms
                send-to: /{dbg}/MockRecvService.in-queue
                rope:
                    width: 35 mm        # Diameter of the rome
                    length: 3000 m      # Total working length of the rope
                    segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
                    pos: point real 'App/Winch.EncoderBR2'      # meters, current rope position
                    load: point real '/App/Winch.Load'          # tonn, current rope load 
                camera:
                    fps: Max                    # Max / Min / 30.0
                    resolution: 
                        width: 1200
                        height: 800
                    index: 0
                    # address: 192.168.10.12:2020
                    # Mono8/10/12/16, Bayer8/10/12/16, RGB8, BGR8, YCbCr8, YCbCr411, YUV422, YUV411 | Default and fastest BayerRG8
                    # pixel-format:  Mono8
                    # pixel-format:  BayerRG8
                    # pixel-format:  QOI_Mono8
                    pixel-format:  QOI_BayerRG8
                    exposure:
                        auto: Off                   # Off / Continuous
                        time: 26000                   # microseconds
                    auto-packet-size: true          # StreamAutoNegotiatePacketSize
                    channel-packet-size: Max        # Maximizing packet size increases frame rate
                    resend-packet: true             # StreamPacketResendEnable
        ")).unwrap(),
    );
    log::trace!("config: {:?}", &conf);
    let tp = ThreadPool::new(&dbg, Some(8));
    let services = Arc::new(Services::new(&dbg, ServicesConf::new(
        &dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
        retain:
        "#).unwrap()),
    ), Some(tp.scheduler())));
    let frdm = Arc::new(FrdmService::new(conf, services.clone(), tp.scheduler()));
    services.insert(frdm.clone());
    let receiver = Arc::new(MockRecvService::new(&dbg, &format!("in-queue"), None));
    services.insert(receiver.clone());
    services.run().unwrap();
    receiver.run().unwrap();
    frdm.run().unwrap();

    frdm.exit();
    services.exit();
    frdm.wait().unwrap();
    services.wait().unwrap();

    test_duration.exit();
}
