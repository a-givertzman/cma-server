#[cfg(test)]
use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDistance, ConfDistanceUnit, ConfTree};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::{BendingsConf, BoomConf, CraneConf, RopeConf};

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
                bendings:           # Rope bloks with diameter, inter and exit
                    # Block Diameter   inter   exit
                    - D200mm           5.0  .. 5.15 m
                    - D300mm           7.23 .. 7.30 mm
                boom:
                    main-len: 5.3 m                                        # length of the main boom
                    main-angle: point real 'App/Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
                    rotary-len: 2.1 m                                      # length of the rotary boom
                    rotary-angle: point real 'App/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
                booms:
                    - Main-Boom:
                        l1: 0.1 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                        l2: 0.2 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                        l3: 0.3 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                        l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                        len: 11200.0 mm                                         # length of the boom
                        angle: point real 'App/Load.MainBoomAngle'   # degrees, current angle of the boom (relative axis)
                    - Rotary-Boom:
                        l1: 0.1 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                        l2: 0.2 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                        l3: 0.3 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                        l4: 0.4 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                        len: 7984.0 mm                                          # length of the rotary boom
                        angle: point real 'App/Load.RotaryBoomAngle' # degrees, current angle of the boom (relative axis)
                rope:
                    width: 35 mm        # Diameter of the rome
                    length: 3000 m      # Total working length of the rope
                    segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
                    pos: point real '/App/Winch.EncoderBR2'      # meters, current rope position
                    load: point real '/App/Winch.Load'          # tonn, current rope load 
            ").unwrap(),
            CraneConf {
                bendings: BendingsConf {
                    bendings: vec![
                        (ConfDistance::new(200.0, ConfDistanceUnit::Millimeter), 5.0..5.15),
                        (ConfDistance::new(300.0, ConfDistanceUnit::Millimeter), 7.23*0.001..7.3*0.001),
                    ],
                },
                booms: vec![
                    ("Main-Boom".to_owned(), BoomConf {
                        l1: ConfDistance::new(0.1, ConfDistanceUnit::Millimeter),
                        l2: ConfDistance::new(0.2, ConfDistanceUnit::Millimeter),
                        l3: ConfDistance::new(0.3, ConfDistanceUnit::Millimeter),
                        l4: ConfDistance::new(10330.0, ConfDistanceUnit::Millimeter),
                        len: ConfDistance::new(11200.0, ConfDistanceUnit::Millimeter),
                        angle: "App/Load.MainBoomAngle".to_owned(),
                        // main_len: ConfDistance::new(5.3, ConfDistanceUnit::Meter),
                        // main_angle: "App/Load.MainBoomAngle".to_owned(),
                        // rotary_len: ConfDistance::new(2.1, ConfDistanceUnit::Meter),
                        // rotary_angle: "App/Load.RotaryBoomAngle".to_owned(),
                    }),
                    ("Rotary-Boom".to_owned(), BoomConf {
                        l1: ConfDistance::new(0.1, ConfDistanceUnit::Millimeter),
                        l2: ConfDistance::new(0.2, ConfDistanceUnit::Millimeter),
                        l3: ConfDistance::new(0.3, ConfDistanceUnit::Millimeter),
                        l4: ConfDistance::new(0.4, ConfDistanceUnit::Millimeter),
                        len: ConfDistance::new(7984.0, ConfDistanceUnit::Millimeter),
                        angle: "App/Load.RotaryBoomAngle".to_owned(),
                        // main_len: ConfDistance::new(5.3, ConfDistanceUnit::Meter),
                        // main_angle: "App/Load.MainBoomAngle".to_owned(),
                        // rotary_len: ConfDistance::new(2.1, ConfDistanceUnit::Meter),
                        // rotary_angle: "App/Load.RotaryBoomAngle".to_owned(),
                    }),
                ],
                rope: RopeConf {
                    width: ConfDistance::new(35.0, ConfDistanceUnit::Millimeter),
                    length: ConfDistance::new(3000.0, ConfDistanceUnit::Meter),
                    segment: ConfDistance::new(100.0, ConfDistanceUnit::Millimeter),
                    pos: "/App/Winch.EncoderBR2".to_owned(),
                    load: "/App/Winch.Load".to_owned(),
                },
            },
        ),
    ];
    for (step, conf, target) in test_data {
        let result = CraneConf::new(&dbg, ConfTree::new_root(conf));
        assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
