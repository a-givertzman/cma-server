#[cfg(test)]
use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::ConfTree, entity::{Point, ToPoint}};
use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
use debugging::session::{DebugSession, LogLevel};
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
#[ignore = "Isn't implemented yet."]
#[test]
fn run() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("FrdmService-test");
    log::debug!("\n{dbg}");
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(40));
    test_duration.run().unwrap();
    let events = vec![
        vec![   // SendService0
            ("Load.MainBoomAngle", Value::Real(68.0)),
            ("Load.RotaryBoomAngle", Value::Real(155.3)),
            ("Winch.RopePos", Value::Real(0.000)),  // 55          // EncoderBR0
            ("Winch.RopePos", Value::Real(0.050)),  // 55
            ("Winch.RopePos", Value::Real(0.100)),  // 55
            ("Winch.RopePos", Value::Real(0.150)),  // 55
            ("Winch.RopePos", Value::Real(0.200)),  // 55
            ("Winch.RopePos", Value::Real(0.250)),  // 55
            ("Winch.RopePos", Value::Real(0.300)),  // 55
            ("Winch.RopePos", Value::Real(0.350)),  // 55
            ("Winch.RopePos", Value::Real(0.400)),
            ("Winch.RopePos", Value::Real(0.450)),
            ("Winch.RopePos", Value::Real(0.500)),
            ("Winch.RopePos", Value::Real(0.550)),
            ("Winch.RopePos", Value::Real(0.600)),
            ("Winch.RopePos", Value::Real(0.650)),
            ("Winch.RopePos", Value::Real(0.700)),  // 56
            ("Winch.RopePos", Value::Real(0.750)),  // 56
            ("Winch.RopePos", Value::Real(0.800)),  // 56
            ("Winch.RopePos", Value::Real(0.850)),  // 56
            ("Winch.RopePos", Value::Real(0.900)),  // 56
            ("Winch.RopePos", Value::Real(0.950)),  // 56
            ("Winch.RopePos", Value::Real(1.000)),  // 56
            ("Winch.RopePos", Value::Real(1.050)),  // 56
            ("Winch.RopePos", Value::Real(1.100)),  // 56
            ("Winch.RopePos", Value::Real(1.150)),  // 56
            ("Winch.RopePos", Value::Real(1.200)),  // 56
            ("Winch.RopePos", Value::Real(1.250)),  // 56
            ("Winch.RopePos", Value::Real(1.300)),
            ("Winch.RopePos", Value::Real(1.350)),
            ("Winch.RopePos", Value::Real(1.400)),
            ("Winch.RopePos", Value::Real(1.450)),
            ("Winch.RopePos", Value::Real(1.500)),  // 57
            ("Winch.RopePos", Value::Real(1.550)),  // 57
            ("Winch.RopePos", Value::Real(1.600)),  // 57
            ("Winch.RopePos", Value::Real(1.650)),  // 57
            ("Winch.RopePos", Value::Real(1.700)),  // 57
            ("Winch.RopePos", Value::Real(1.750)),  // 57
            ("Winch.RopePos", Value::Real(1.800)),  // 57
            ("Winch.RopePos", Value::Real(1.850)),  // 57
            ("Winch.RopePos", Value::Real(1.900)),  // 57
            ("Winch.RopePos", Value::Real(1.950)),  // 57
            ("Winch.RopePos", Value::Real(2.000)),  // 57
            // ("Winch.Load", Value::Int(1)),
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
            thread-pool: 24
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
                subscribe: /{dbg}/MultiQueue       # Service name, to subscribe for event's required for the calculations like rope positin and crane angles
                api-client:
                    wait-started: 10 ms
                    address: 127.0.0.1:8081
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
                        normalize:
                            cropping:
                                x: 230              # New left edge
                                y: 300              # New top edge
                                width: 1410         # New image width
                                height: 1000        # New image height
                            gamma:
                                factor: 120.0       # Percent of influence of [AutoGamma] algorythm bigger the value more the effect of [AutoGamma] algorythm, %

                        fast-scan:
                            fast-contours:
                                otsu-tune: 0.40
                            temporal-filter:
                                gaussian:
                                    kernel: [11, 11]    # Gausian blur kernel size, must be odd
                                    sigma: [0.0, 0.0]   # Standard deviation in [X, Y] direction, The higher the value, the more pixels are used to count each pixel and the smoother blur will be
                                open-kernel: [3, 3]     # Morphology open operation kernel size [w, h], default [5, 5]
                                erode-kernel: [3, 3]    # Morphology erode operation kernel size [w, h], default [5, 5]
                                threshold: 12.0         # Threshold to detect the pixel whas changed or not in the each next frame
                            fast-edges:
                                otsu-tune: 1.40         # Multiplier to otsu auto threshold, 1.0 - do nothing, just use otsu auto threshold, default 1.0
                                # threshold: 128        # 0...255, used if otsu-tune is not specified
                                smooth: 36              # Smoothing of edge line factor. The higher the factor the smoother the line.
                            union:
                                add-weighted:
                                    weight1: 1.0            # Weight of the first array elements.
                                    weight2: 1.0            # Weight of the second array elements.
                            rope-dimensions:        # Verifaing the rope dimensions
                                rope-width: 380               # Standart rope width, px
                                width-tolerance: 50.0         # Tolerance for rope width, %
                                square-tolerance: 100.0       # Tolerance for rope square, %
                            distortion-threshold: 1.2    # 1.1..1.3, absolute threshold to detect the geometry deffects

                        fine-scan:
                            fine-contours:
                                otsu-tune: 0.40         # Auto threshold factor, 1 - no correction, 0..1 - more, 1.. - less sensitive
                                merge-distance: 24.0    # Maximum distance between contours to be merged
                            temporal-filter:
                                gaussian:
                                    kernel: [11, 11]    # Gausian blur kernel size, must be odd
                                    sigma: [0.0, 0.0]   # Standard deviation in [X, Y] direction, The higher the value, the more pixels are used to count each pixel and the smoother blur will be
                                open-kernel: [3, 3]     # Morphology open operation kernel size [w, h], default [5, 5]
                                erode-kernel: [3, 3]    # Morphology erode operation kernel size [w, h], default [5, 5]
                                threshold: 12.0         # Threshold to detect the pixel whas changed or not in the each next frame
                            fine-edges:
                                # otsu-tune: 1.40       # Multiplier to otsu auto threshold, 1.0 - do nothing, just use otsu auto threshold, default 1.0
                                threshold: 16           # 0...255, used if otsu-tune is not specified
                                smooth: 16              # Smoothing of edge line factor. The higher the factor the smoother the line.
                            union:
                                # add-weighted:
                                #     weight1: 1.0            # Weight of the first array elements.
                                #     weight2: 1.0            # Weight of the second array elements.
                                bitwise-and:
                                    no-params: ~
                            rope-dimensions:        # Verifaing the rope dimensions
                                rope-width: 380               # Standart rope width, px
                                width-tolerance: 30.0         # Tolerance for rope width, %
                                square-tolerance: 100.0       # Tolerance for rope square, %
                            distortion-threshold: 1.4    # 1.1..1.3, absolute threshold to detect the geometry deffects
                            defect-threshold: 2.5        # 1.1..1.3, absolute threshold to detect the geometry deffects
                    camera Camera1:
                        from-path: src/tests/unit/services/frdm_service/frames
                        fps: Max                    # Max / Min / 30.0
                        resolution:
                            width: 1200
                            height: 800
                        index: 0
                        # address: 192.168.10.12:2020
                        pixel-format:  QOI_Mono8    # QOI_Mono8, QOI_BayerRG8, Mono8/10/12/16, BayerGG8/10/12/16, RGB8, BGR8, YCbCr8, YCbCr411, YUV422, YUV411 | Default and fastest BayerRG8
                        exposure:
                            auto: Off                   # Off / Continuous
                            time: 26000                 # microseconds
                        auto-packet-size: true          # StreamAutoNegotiatePacketSize
                        channel-packet-size: Max        # Maximizing packet size increases frame rate
                        resend-packet: true             # StreamPacketResendEnable
                rope-deprecation:
                    wait-started: 10 ms
                    table: public.frdm_deprecation
                    subscribe: /{dbg}/MultiQueue    # Service name, to subscribe for rope positin and crane angles event's
                    crane:
                        rope:
                            width: 35 mm            # Diameter of the rome
                            length: 3000 m          # Total working length of the rope
                            aux-length: 1.200 m     # Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
                            segment: 100 mm         # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
                            pos: point real 'Winch.RopePos'      # meters, current rope position
                            load: point real 'Winch.Load'        # tonn, current rope load
                        booms:
                            - Main-Boom:
                                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                                l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                                len: 11200.0 mm                                         # length of the boom
                                angle: point real 'Load.MainBoomAngle'   # degrees, current angle of the boom (relative axis)
                                parking: 0.0 deg            # Угол в парковочном положении, град
                            - Rotary-Boom:
                                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                                l4: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                                len: 7984.1 mm                                          # length of the rotary boom
                                angle: point real 'Load.RotaryBoomAngle' # degrees, current angle of the boom (relative axis)
                                parking: 23.78 deg          # Угол в парковочном положении, град
                        blocks:
                            - 1:
                                lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                                d: 844.0 mm                 # Диаметры блоков, мм
                                scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                                bind: Fixed                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                            - 2:
                                lf: 308.0 mm, 1090.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
                                d: 816.0 mm                 # Диаметры блоков, мм
                                scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                                bind: Boom 0                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                            - 3:
                                lf: -6550.0 mm, 1730.0 mm   # Растояние (x, y) от **конца** стрелы до оси блока, мм
                                d: 816.0 mm                 # Диаметры блоков, мм
                                scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                            - 4:
                                lf: -1121.0 mm, 973.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                                d: 816.0 mm                 # Диаметры блоков, мм
                                scheme: TopBottom          # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                                bind: Boom 2                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                            - 5:
                                lf: 267.0 mm, 860.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                                d: 816.0 mm                 # Диаметры блоков, мм
                                scheme: BottomTop          # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                                bind: Boom 3                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                            - 6:
                                lf: 136.0 mm, -35.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                                d: 816.0 mm                 # Диаметры блоков, мм
                                scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                                bind: Boom 4                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                            - 7:
                                lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
                                d: 0.0 mm                   # Диаметры блоков, мм
                                scheme: TopTop             # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                                bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе

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
            std::thread::sleep(Duration::from_millis(50));
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
                let _ = target;
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
    ).unwrap();
    planner.run().unwrap();
    std::thread::sleep(Duration::from_millis(3_000));
    planner.exit();
    planner.wait().unwrap();
    test_duration.exit();
}
