#[cfg(test)]

use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::ConfTree, entity::{Point, ToPoint}};
use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::domain::testing::ServiceTestPlanner;
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
fn run() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("ServiceTestPlanner-test");
    log::debug!("\n{dbg}");
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    let conf = ConfTree::new_root(
        serde_yaml::from_str(&format!(r"
            thread-pool: 12
            services:
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json
                        # api:
                        #     table: public.tags
                        #     address: 0.0.0.0:8080
                        #     auth_token: 123!@#
                        #     database: crane_data_server

            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:

            service SendService SendService0:
                send-to: /{dbg}/MultiQueue.in-queue

            service RecvService RecvService0:
                in queue in-queue:
                    max-length: 10000
            service RecvService RecvService1:
                in queue in-queue:
                    max-length: 10000

            service FrdmService:
                cycle: 100 ms
                send-to: /{dbg}/RecvService0.in-queue
                subscribe: /{dbg}/MultiQueue
                tables:
                    defect: public.frdm_defect
                    defect-image: public.frdm_defect_image
                    deprecation: public.frdm_deprecation
                crane:
                    bendings:
                        - D200mm 2.4..2.5 m
                        - D200mm 2.7..2.9 m
                        - D200mm 3.1..3.2 m
                    boom:
                        main-len: 5.3 m                                         # length of the main boom
                        main-angle: point real 'Load.MainBoomAngle'      # degrees, current angle of the main boom to vertical axis
                        rotary-len: 2.1 m                                       # length of the rotary boom
                        rotary-angle: point real 'Load.RotaryBoomAngle'  # degrees, current angle of the rotary boom (jib) to boom axis
                    rope:
                        width: 35 mm        # Diameter of the rome
                        length: 3000 m      # Total working length of the rope
                        segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
                        pos: point real 'Winch.EncoderBR2'      # meters, current rope position
                        load: point real 'Winch.Load'          # tonn, current rope load
                scan:
                    detecting-contours:
                        gamma:
                            no-param: not parameters implemented 
                        brightness-contrast:
                            histogram-clipping: 1     # optional histogram clipping, default = 0 %
                        gausian:
                            kernel-size:
                                width: 3
                                height: 3
                            sigma-x: 0.0
                            sigma-y: 0.0
                        sobel:
                            kernel-size: 3
                            scale: 1.0
                            delta: 0.0
                        overlay:
                            src1-weight: 0.5
                            src2-weight: 0.5
                            gamma: 0.0
                    fast-scan:
                        geometry-defect-threshold: 1.2      # 1.1..1.3, absolute threshold to detect the geometry deffects
                    fine-scan:
                        no-params: not implemented yet
                camera-offset: 5.5 m                        # camera position from the begin of the rope (hook side)
                camera Camera1:
                    fps: Max                    # Max / Min / 30.0
                    resolution: 
                        width: 1200
                        height: 800
                    index: 0
                    # address: 192.168.10.12:2020
                    # Mono8/10/12/16, Bayer8/10/12/16, RGB8, BGR8, YCbCr8, YCbCr411, YUV422, YUV411 | Default and fastest BayerRG8
                    # pixel-format:  Mono8
                    # pixel-format:  BayerRG8
                    pixel-format:  QOI_Mono8
                    # pixel-format:  QOI_BayerRG8
                    exposure:
                        auto: Off                   # Off / Continuous
                        time: 26000                 # microseconds
                    auto-packet-size: true          # StreamAutoNegotiatePacketSize
                    channel-packet-size: Max        # Maximizing packet size increases frame rate
                    resend-packet: true             # StreamPacketResendEnable
        ")).unwrap(),
    );
    let test_data = [
        (01, 111, 112),
        (02, 222, 223),
        (03, 333, 334),
    ];
    for (step, val, target) in test_data {
        let result = val + 1;
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    let builder_dbg = dbg.clone();
    let each_sent_dbg = dbg.clone();
    let each_received_dbg = dbg.clone();
    let all_received_dbg = dbg.clone();
    let planner = ServiceTestPlanner::new(
        &dbg,
        conf,
        move |txid, ix, name: &str, event: &Value| {
            let dbg = builder_dbg.clone();
            log::debug!("{dbg} | test event {ix}: '{name}'");
            event.to_point(txid, name)
        },
        vec![
            vec![   // SendService0
                ("Int0", Value::Int(0)),
                ("Int1", Value::Int(1)),
                ("Int2", Value::Int(2)),
                ("Int3", Value::Int(3)),
                ("Int4", Value::Int(4)),
                ("Int5", Value::Int(5)),
                ("Int6", Value::Int(6)),
            ],
        ],
        vec![
            move |event: &Point| {
                let dbg = each_sent_dbg.clone();
                log::debug!("{dbg} | Sent event: {:?}", event.name());
            },
        ],
        (0..1).map(|ix| {
            let dbg = each_received_dbg.clone();
            move |received: &Vec<Point>| {    // RecvService0
                log::debug!("{dbg} | Receiver{ix} received: {:?}", received);
            }
        }).collect(),
        move |received: Vec<Vec<Point>>| {
            let dbg = all_received_dbg.clone();
            log::debug!("{dbg} | All received:");
            for (ix, recvd) in received.iter().enumerate() {
                log::debug!("{dbg} | Received[{ix}]: {:?}", recvd);
            }
        },
    );
    planner.run().unwrap();
    planner.wait().unwrap();
    test_duration.exit();
}
