use std::sync::{Arc, atomic::AtomicBool};
#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::{math::AproxEq, services::conf::ConfTree};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, CraneConf, FrdmServiceConf, Inputs, RopeSections, RopeDeprecationConf};

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
/// Testing [Bendings]
#[test]
fn new() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Bendings-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01,  [
            // Input Events
            ("Winch.Pos",          0.00),
            ("MainBoom.Angle",    69.71),
            ("RotaryBoom.Angle", 155.30)
        ], 
        // Targets
        [
            // enter .. exit, mm
                0.0000 .. 65565.5016,   // start from the end of the ropr on the winch
            77074.5163 .. 77075.4196,
            78797.8560 .. 79000.0876,
            84481.6102 .. 84713.7866,
            85842.1211 .. 85889.6608,
            86279.5476 .. 87000.0000,
    // F12: 88000.0000
        ]),
        (02,  [
            // Input Events
            ("Winch.Pos",          0.00),
            ("MainBoom.Angle",    74.00),
            ("RotaryBoom.Angle", 128.00)
        ], 
        // Targets
        [
            // enter .. exit, mm
                0.0000 .. 65144.9469,   // start from the end of the ropr on the winch
            76507.0587 .. 76696.6577,
            78957.9230 .. 79163.9797,
            84645.5023 .. 84877.6787,
            86006.0132 .. 86053.5528,
            86443.4396 .. 87000.0000,
    // F12: 88000.0000
        ]),
    ];

    // Опорные точки
    // F01: 65565.5016
    // F02: 77074.5163
    // F03: 77075.4196
    // F04: 78797.8560
    // F05: 79000.0876
    // F06: 84481.6102
    // F07: 84713.7866
    // F08: 85842.1211
    // F09: 85889.6608
    // F10: 86279.5476
    // F11: 87000.0000
    // F12: 88000.0000

    // Опорные точки
    // F01: 65144.9469
    // F02: 76507.0587
    // F03: 76696.6577
    // F04: 78957.9230
    // F05: 79163.9797
    // F06: 84645.5023
    // F07: 84877.6787
    // F08: 86006.0132
    // F09: 86053.5528
    // F10: 86443.4396
    // F11: 87000.0000
    // F12: 88000.0000

    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        rope:
            width: 35 mm            # Diameter of the rome
            length: 88 m          # Total working length of the rope
            winch-length: 65.5655 m    # Length of the rope on the winch drum in the parking position, when rope pos is zero
            segment: 100 mm         # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
            pos: point real 'Winch.Pos'         # meters, current rope position (длина каната размотанного с барабана считая от парковочного)
            load: point real 'Winch.Load'       # tonn, current rope load 
        booms:
            - Main-Boom:
                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                len: 11200.0 mm                                         # length of the boom
                angle: point real 'MainBoom.Angle'   # degrees, current angle of the boom (relative axis)
            - Rotary-Boom:
                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                l4: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                len: 7984.0 mm                                          # length of the rotary boom
                angle: point real 'RotaryBoom.Angle' # degrees, current angle of the boom (relative axis)
        blocks:
            - 1:
                lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 845.670 mm               # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Fixed                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 2:
                lf: 308.0 mm, 1100.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.195 mm               # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 0                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 3:
                lf: -6550.0 mm, 1730.0 mm   # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.195 mm               # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 4:
                lf: -1121.0 mm, 973.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.195 mm               # Диаметр блока, мм
                scheme: TopBottom           # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 5:
                lf: 267.0 mm, 860.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.195 mm               # Диаметр блока, мм
                scheme: BottomTop           # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 6:
                lf: 136.0 mm, -35.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.195 mm               # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 7:
                lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 0.0 mm                   # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе

    ").unwrap());
    let conf = CraneConf::new(&dbg, conf);
    let inputs = Arc::new(Inputs::fake(
        &dbg,
        &FrdmServiceConf {
            rope_deprecation: RopeDeprecationConf {
                crane: conf.clone(),
                ..Default::default()
            },
            ..Default::default()
        },
        [("", 0.0)],
        Arc::new(AtomicBool::new(false)),
    ));
    let mut bendings = Bendings::new(
        &dbg,
        &conf.rope,
        BlockArcs::new(
            &dbg,
            RopeSections::new(
                &dbg,
                Blocks::new(
                    &dbg,
                    1200.0,        // TODO: replace with config or calculated value
                    &conf.blocks,
                    Booms::new(&dbg, &conf.booms, inputs.clone()),
                ),
            ),
        ),
    );
    let t = Instant::now();
    for (step, events, target) in test_data {
        for (key, val) in events {
            log::debug!("{dbg} | step {step}  Event '{}': {:?}", key, val);
            inputs.insert(key.to_owned(), val);
        }
        let result = bendings.eval(&inputs).unwrap();
        log::trace!("{dbg} | step {step}  result: {:#?}", result);
        log::debug!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
        for (i, bending) in target.into_iter().enumerate() {
            assert!(result[i].bending.start.aprox_eq(bending.start, 1), "{dbg} | step {step} Bending {i}  \nresult: {:?}\ntarget: {:?}", result[i].bending.start, bending.start);
            assert!(result[i].bending.end.aprox_eq(bending.end, 1), "{dbg} | step {step} Bending {i}  \nresult: {:?}\ntarget: {:?}", result[i].bending.end, bending.end);
        }
    }
    test_duration.exit();
}
