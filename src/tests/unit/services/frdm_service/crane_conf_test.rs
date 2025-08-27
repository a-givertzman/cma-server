#[cfg(test)]
use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::{ConfDistance, ConfDistanceUnit, ConfTree};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::{BlockBind, BlockConf, BlockScheme, BoomConf, InputKind, CraneConf, Offset, RopeConf};

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
                    main-angle: point real 'Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
                    rotary-len: 2.1 m                                      # length of the rotary boom
                    rotary-angle: point real 'Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
                booms:
                    - Main-Boom:
                        l1: 0.1 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                        l2: 0.2 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                        l3: 0.3 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                        l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                        len: 11200.0 mm                                         # length of the boom
                        angle: point real 'Load.MainBoomAngle'   # degrees, current angle of the boom (relative axis)
                    - Rotary-Boom:
                        l1: 0.1 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                        l2: 0.2 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                        l3: 0.3 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                        l4: 0.4 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                        len: 7984.0 mm                                          # length of the rotary boom
                        angle: point real 'Load.RotaryBoomAngle' # degrees, current angle of the boom (relative axis)
                blocks:
                    - '1':
                        lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                        d: 844.0 mm                 # Диаметры блоков, мм
                        scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                        bind: Fixed                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                    - '2':
                        lf: 308.0 mm, 1090.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
                        d: 816.0 mm                 # Диаметры блоков, мм
                        scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                        bind: Boom 0                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                    - '3':
                        lf: -6550.0 mm, 1730.0 mm   # Растояние (x, y) от **конца** стрелы до оси блока, мм
                        d: 816.0 mm                 # Диаметры блоков, мм
                        scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                        bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                    - '4':
                        lf: -1121.0 mm, 973.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                        d: 816.0 mm                 # Диаметры блоков, мм
                        scheme: TopBottom           # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                        bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                    - '5':
                        lf: 267.0 mm, 860.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                        d: 816.0 mm                 # Диаметры блоков, мм
                        scheme: BottomTop           # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                        bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                    - '6':
                        lf: 136.0 mm, -35.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                        d: 816.0 mm                 # Диаметры блоков, мм
                        scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                        bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                    - '7':
                        lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
                        d: 0.0 mm                   # Диаметры блоков, мм
                        scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                        bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе

                rope:
                    width: 35 mm        # Diameter of the rome
                    length: 3000 m      # Total working length of the rope
                    winch-length: 3000 m      # Total working length of the rope
                    segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
                    pos: point real 'Winch.EncoderBR2'      # meters, current rope position
                    load: point real 'Winch.Load'          # tonn, current rope load
            ").unwrap(),
            CraneConf {
                booms: vec![
                    ("Main-Boom".to_owned(), BoomConf {
                        l1: ConfDistance::new(0.1, ConfDistanceUnit::Millimeter),
                        l2: ConfDistance::new(0.2, ConfDistanceUnit::Millimeter),
                        l3: ConfDistance::new(0.3, ConfDistanceUnit::Millimeter),
                        l4: ConfDistance::new(10330.0, ConfDistanceUnit::Millimeter),
                        len: InputKind::Const(ConfDistance::new(11200.0, ConfDistanceUnit::Millimeter)),
                        angle: InputKind::Point("Load.MainBoomAngle".to_owned()),
                    }),
                    ("Rotary-Boom".to_owned(), BoomConf {
                        l1: ConfDistance::new(0.1, ConfDistanceUnit::Millimeter),
                        l2: ConfDistance::new(0.2, ConfDistanceUnit::Millimeter),
                        l3: ConfDistance::new(0.3, ConfDistanceUnit::Millimeter),
                        l4: ConfDistance::new(0.4, ConfDistanceUnit::Millimeter),
                        len: InputKind::Const(ConfDistance::new(7984.0, ConfDistanceUnit::Millimeter)),
                        angle: InputKind::Point("Load.RotaryBoomAngle".to_owned()),
                    }),
                ],
                blocks: vec![
                    ("1".to_owned(), BlockConf {
                        lf: Offset::new(ConfDistance::new(1830.0, ConfDistanceUnit::Millimeter), ConfDistance::new(710.0, ConfDistanceUnit::Millimeter)),
                        d: ConfDistance::new(844.0, ConfDistanceUnit::Millimeter),
                        scheme: BlockScheme::TopTop,
                        bind: BlockBind::Fixed,
                    }),
                    ("2".to_owned(), BlockConf {
                        lf: Offset::new(ConfDistance::new(308.0, ConfDistanceUnit::Millimeter), ConfDistance::new(1090.0, ConfDistanceUnit::Millimeter)),
                        d: ConfDistance::new(816.0, ConfDistanceUnit::Millimeter),
                        scheme: BlockScheme::TopTop,
                        bind: BlockBind::Boom(0),
                    }),
                    ("3".to_owned(), BlockConf {
                        lf: Offset::new(ConfDistance::new(-6550.0, ConfDistanceUnit::Millimeter), ConfDistance::new(1730.0, ConfDistanceUnit::Millimeter)),
                        d: ConfDistance::new(816.0, ConfDistanceUnit::Millimeter),
                        scheme: BlockScheme::TopTop,
                        bind: BlockBind::Boom(1),
                    }),
                    ("4".to_owned(), BlockConf {
                        lf: Offset::new(ConfDistance::new(-1121.0, ConfDistanceUnit::Millimeter), ConfDistance::new(973.0, ConfDistanceUnit::Millimeter)),
                        d: ConfDistance::new(816.0, ConfDistanceUnit::Millimeter),
                        scheme: BlockScheme::TopBottom,
                        bind: BlockBind::Boom(1),
                    }),
                    ("5".to_owned(), BlockConf {
                        lf: Offset::new(ConfDistance::new(267.0, ConfDistanceUnit::Millimeter), ConfDistance::new(860.0, ConfDistanceUnit::Millimeter)),
                        d: ConfDistance::new(816.0, ConfDistanceUnit::Millimeter),
                        scheme: BlockScheme::BottomTop,
                        bind: BlockBind::Boom(1),
                    }),
                    ("6".to_owned(), BlockConf {
                        lf: Offset::new(ConfDistance::new(136.0, ConfDistanceUnit::Millimeter), ConfDistance::new(-35.0, ConfDistanceUnit::Millimeter)),
                        d: ConfDistance::new(816.0, ConfDistanceUnit::Millimeter),
                        scheme: BlockScheme::TopTop,
                        bind: BlockBind::Boom(1),
                    }),
                    ("7".to_owned(), BlockConf {
                        lf: Offset::new(ConfDistance::new(0.0, ConfDistanceUnit::Millimeter), ConfDistance::new(0.0, ConfDistanceUnit::Millimeter)),
                        d: ConfDistance::new(0.0, ConfDistanceUnit::Millimeter),
                        scheme: BlockScheme::TopTop,
                        bind: BlockBind::Hook,
                    }),
                ],
                rope: RopeConf {
                    width: ConfDistance::new(35.0, ConfDistanceUnit::Millimeter),
                    length: ConfDistance::new(3000.0, ConfDistanceUnit::Meter),
                    winch_len: ConfDistance::new(2985.0, ConfDistanceUnit::Meter),
                    segment: ConfDistance::new(100.0, ConfDistanceUnit::Millimeter),
                    pos: "Winch.EncoderBR2".to_owned(),
                    load: "Winch.Load".to_owned(),
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
