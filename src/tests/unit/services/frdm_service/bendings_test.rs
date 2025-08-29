#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, math::AproxEq, services::conf::ConfTree};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, CraneConf, LooseRopeSections};

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
/// Testing [BlockArcs]
#[test]
fn new() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("BlockArcs-test");
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
            65566.0 .. 65566.0,
            77075.0 .. 77075.0,
            78798.0 .. 79000.0,
            84482.0 .. 84714.0,
            85842.0 .. 85890.0,
            86280.0 .. 87000.0,
            // 88000.0 .. 88000.0,
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
            65145.0 .. 65145.0,
            76507.0 .. 76697.0,
            78958.0 .. 79164.0,
            84646.0 .. 84878.0,
            86006.0 .. 86054.0,
            86443.0 .. 87000.0,
            // 88000.0 .. 88000.0,
        ]),
    ];

    // Опорные точки
    // F01:   65.566
    // F02:   77.075
    // F03:   77.075
    // F04:   78.798
    // F05:   79.000
    // F06:   84.482
    // F07:   84.714
    // F08:   85.842
    // F09:   85.890
    // F10:   86.280
    // F11:   87.000
    // F12:   88.000

    // Опорные точки
    // F01:   65.145
    // F02:   76.507
    // F03:   76.697
    // F04:   78.958
    // F05:   79.164
    // F06:   84.646
    // F07:   84.878
    // F08:   86.006
    // F09:   86.054
    // F10:   86.443
    // F11:   87.000
    // F12:   88.000

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
    let mut inputs = FxIndexMap::default();
    let mut bendings = Bendings::new(
        &dbg,
        conf.rope.pos.clone(),
        conf.rope.winch_len,
        BlockArcs::new(
            &dbg,
            LooseRopeSections::new(
                &dbg,
                Blocks::new(
                    &dbg,
                    &conf.blocks,
                    Booms::new(&dbg, &conf.booms, &mut vec![]),
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
        log::debug!("{dbg} | step {step}  result: {:#?}", result);
        log::debug!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
        for (i, bending) in target.into_iter().enumerate() {
            assert!(result[i].bending == bending, "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", result[i].bending, bending);
            // assert!(result[i].wrap_length == wrap_length, 2), "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", result[i].wrap_length, wrap_length);
        }
    }
    test_duration.exit();
}
