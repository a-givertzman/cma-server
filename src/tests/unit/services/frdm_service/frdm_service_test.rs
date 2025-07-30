#[cfg(test)]
use std::sync::Arc;
use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::Point, MultiQueue, MultiQueueConf, Service, Services}, thread_pool::ThreadPool};
use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::{domain::testing::{RecvService, RecvServiceConf, SendService, SendServiceConf}, services::{FrdmService, FrdmServiceConf}};

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
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(30));
    test_duration.run().unwrap();
    let test_data: Vec<(&str, Value)> = vec![
    //     (01, ),
    ];
    // for (step, conf, target) in test_data {
    //     let result = RopeConf::new(&dbg, ConfTree::new("rope", conf));
    //     assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    // }
    let conf = FrdmServiceConf::from_yaml(&dbg,
        &serde_yaml::from_str(&format!(r"
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
                        length: 300 m      # Total working length of the rope
                        segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
                        pos: point real 'Winch.EncoderBR2'      # meters, current rope position
                        load: point real 'Winch.Load'          # tonn, current rope load
                scan:
                    segment: 100 mm     # Whole rope will divided by the segments for the Camera defect detection, recomended: `segment length = camera.width * 0.10..0.20`
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
                    pixel-format:  QOI_Mono8
                    exposure:
                        auto: Off                   # Off / Continuous
                        time: 26000                 # microseconds
                    auto-packet-size: true          # StreamAutoNegotiatePacketSize
                    channel-packet-size: Max        # Maximizing packet size increases frame rate
                    resend-packet: true             # StreamPacketResendEnable
                camera Camera2:
                    fps: Max                    # Max / Min / 30.0
                    resolution: 
                        width: 1200
                        height: 800
                    index: 0
                    pixel-format:  QOI_Mono8
                    exposure:
                        auto: Off                   # Off / Continuous
                        time: 26000                 # microseconds
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
    let conf = serde_yaml::from_str(&format!(r#"
        service MultiQueue:
            in queue in-queue:
                max-length: 10000
            send-to:
    "#)).unwrap();
    let mq_conf = MultiQueueConf::from_yaml(&dbg, &conf);
    let mq = Arc::new(MultiQueue::new(mq_conf, services.clone(), Some(tp.scheduler())));
    services.insert(mq.clone());
    let conf = serde_yaml::from_str(&format!(r#"
        service SendService:
            send-to: /{dbg}/MultiQueue.in-queue
    "#)).unwrap();
    let conf = SendServiceConf::from_yaml(&dbg, &conf);
    let producer = Arc::new(SendService::new(
        &dbg,
        conf,
        // &format!("/{dbg}/MultiQueue.in-queue"),
        None::<Box<dyn Fn(usize, usize, &str, &Value) -> Point + Send + Sync + 'static>>,
        test_data,
        services.clone(),
        tp.scheduler(),
    ));
    services.insert(producer.clone());
    let conf = serde_yaml::from_str(&format!(r#"
        service RecvService RecvService:
            in queue in-queue:
                max-length: 10000
    "#)).unwrap();
    let receiver = Arc::new(RecvService::new(
        &dbg,
        RecvServiceConf::from_yaml(&dbg, &conf),
        tp.scheduler(),
    ));
    services.insert(receiver.clone());
    services.run().unwrap();
    receiver.run().unwrap();
    frdm.run().unwrap();
    std::thread::sleep(Duration::from_secs(10));
    frdm.exit();
    producer.exit();
    mq.exit();
    services.exit();
    frdm.wait().unwrap();
    producer.wait().unwrap();
    mq.wait().unwrap();
    services.wait().unwrap();

    test_duration.exit();
}
