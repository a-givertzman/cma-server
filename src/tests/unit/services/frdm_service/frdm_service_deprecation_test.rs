#[cfg(test)]
use std::{sync::Once, time::Duration};
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
    let dbg = Dbg::own("FrdmService-test");
    log::debug!("\n{dbg}");
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(40));
    test_duration.run().unwrap();
    let events = vec![
        vec![   // SendService0
            ("Winch.RopePos", Value::Real(0.500)),  // 55          // EncoderBR0
            ("Winch.Load", Value::Real(1.0)),
            ("Winch.RopePos", Value::Real(0.510)),  // 55
            ("Winch.RopePos", Value::Real(0.520)),  // 55
            ("Winch.RopePos", Value::Real(0.530)),  // 55
            ("Winch.RopePos", Value::Real(0.540)),
            ("Winch.RopePos", Value::Real(0.550)),
            ("Winch.RopePos", Value::Real(0.560)),
            ("Winch.RopePos", Value::Real(0.570)),  // 56
            ("Winch.RopePos", Value::Real(0.580)),  // 56
            ("Winch.RopePos", Value::Real(0.590)),  // 56
            ("Winch.RopePos", Value::Real(0.600)),  // 56
            ("Winch.RopePos", Value::Real(0.610)),  // 56
            ("Winch.RopePos", Value::Real(0.620)),  // 56
            ("Winch.RopePos", Value::Real(0.630)),
            ("Winch.RopePos", Value::Real(0.640)),
            ("Winch.RopePos", Value::Real(0.650)),  // 57
            ("Winch.RopePos", Value::Real(0.660)),  // 57
            ("Winch.RopePos", Value::Real(0.670)),  // 57
            ("Winch.RopePos", Value::Real(0.680)),  // 57
            ("Winch.RopePos", Value::Real(0.690)),  // 57
            ("Winch.RopePos", Value::Real(0.700)),  // 57
            ("Winch.RopePos", Value::Real(0.710)),  // 57
            ("Winch.RopePos", Value::Real(0.720)),  // 57
            ("Winch.Load", Value::Real(1.1)),
            // ("Load.MainBoomAngle", Value::Int(2)),
            // ("Load.RotaryBoomAngle", Value::Int(3)),
            // ("Int4", Value::Int(4)),
            // ("Int5", Value::Int(5)),
            // ("Int6", Value::Int(6)),
        ],
    ];
    let recv_limit0 = events[0].len();    //events[0].len();
    let conf = ConfTree::new_root(
        serde_yaml::from_str(&format!(r#"
            thread-pool: 12
            services:
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json

            service MultiQueue:
                wait-started: 10 ms
                in queue in-queue:
                    max-length: 10000
                send-to:
                    - /{dbg}/RecvService0.in-queue
                    # - /{dbg}/RecvService1.in-queue

            service RecvService RecvService0:
                in queue in-queue:
                    max-length: 10000
                recv-limit: {recv_limit0}

            service FrdmService:
                wait-started: 10 ms
                cycle: 100 ms
                api-client:
                    wait-started: 10 ms
                    address: 0.0.0.0:8081
                    auth-token: "123!@#"
                    database: cma
                table-settings: 'public.frdm_settings'
                rope-defect:
                    wait-started: 10 ms
                    tables:
                        defect: 'public.frdm_defect'
                        defect-image: 'public.frdm_defect_image'
                    segment: 100 mm                       # Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
                    segment-threshold: 5 mm               # Acceptable camera position error in relation to exact segment position 
                    camera-offset: 0.0 m                  # camera position from the begin of the rope (hook side)
                    defect-detection:
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
                rope-deprecation:
                    wait-started: 10 ms
                    table: public.frdm_deprecation
                    subscribe: /{dbg}/MultiQueue    # Service name, to subscribe for rope positin and crane angles event's
                    crane:
                        bendings:
                        # Block Diameter   inter    exit
                        - D300mm           0.500 .. 0.600 m
                        - D300mm           0.700 .. 0.800 m
                        boom:
                            main-len: 5.3 m                                         # length of the main boom
                            main-angle: point real 'Load.MainBoomAngle'      # degrees, current angle of the main boom to vertical axis
                            rotary-len: 2.1 m                                       # length of the rotary boom
                            rotary-angle: point real 'Load.RotaryBoomAngle'  # degrees, current angle of the rotary boom (jib) to boom axis
                        rope:
                            width: 35 mm        # Diameter of the rome
                            length: 10 m      # Total working length of the rope
                            segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
                            pos: point real 'Winch.RopePos'      # meters, current rope position
                            load: point real 'Winch.Load'        # tonn, current rope load

            service SendService SendService0:
                send-to: /{dbg}/MultiQueue.in-queue
        "#)).unwrap(),
    );
    // for (step, val, target) in test_data {
    //     let result = val + 1;
    //     assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    // }
    let builder_dbg = dbg.clone();
    let each_sent_dbg = dbg.clone();
    let each_received_dbg = dbg.clone();
    let all_received_dbg = dbg.clone();
    let planner = ServiceTestPlanner::new(
        &dbg,
        conf,
        move |txid, ix, name: &str, event: &Value| {
            let dbg = builder_dbg.clone();
            log::debug!("{dbg}.event_builder | test event {ix}: '{name}': {:?}", event);
            std::thread::sleep(Duration::from_millis(300));
            event.to_point(txid, name)
        },
        events.clone(),
        vec![
            move |event: &Point| {
                let dbg = each_sent_dbg.clone();
                log::debug!("{dbg} | Sent event: {:?}", event.name());
            },
        ],
        (0..1).map(|ix| {
            let dbg = each_received_dbg.clone();
            let events = events.first().unwrap().clone();
            move |received: &Vec<Point>| {
                let result: Vec<(String, Value)> = received.iter().map(|p| (p.name(), p.value())).collect();
                log::debug!("{dbg} | Receiver{ix} result: {:?}", result.len());
                let target: Vec<(String, Value)> = events.iter().map(|(name, val)| (name.to_string(), val.to_owned())).collect();
                // assert!(result == target, "{dbg} | Receiver{} \nresult: {:?}\ntarget: {:?}", ix, result, target);
            }
        }).collect(),
        move |received: Vec<Vec<Point>>| {
            let dbg = all_received_dbg.clone();
            log::debug!("{dbg} | All received");
            for (ix, recvd) in received.iter().enumerate() {
                log::debug!("{dbg} | Received[{ix}]: {:?}", recvd.len());
            }
        },
    );
    planner.run().unwrap();
    std::thread::sleep(Duration::from_millis(2_000));
    planner.exit();
    planner.wait().unwrap();
    test_duration.exit();
}
