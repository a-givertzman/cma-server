#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, math::AproxEq, services::conf::ConfTree};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::{Blocks, Booms, CraneConf, LooseRopeSections};

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
/// Testing [LooseRopeSection]
#[test]
fn new() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("LooseRopeSection-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01,  [
            // Input Events
            ("MainBoom.Angle",    69.71),
            ("RotaryBoom.Angle", 155.30)
        ], 
        // Targets
        [
            // rope_len_bck,     rope_len_fwd
            (    0.00,           11509.01),
            (11509.01,            1722.44),
            ( 1722.44,            5481.52),
            ( 5481.52,            1128.33),
            ( 1128.33,             389.89),
            (  389.89,            1000.00),
        ]),
        (02,  [
            // Input Events
            ("MainBoom.Angle",    74.00),
            ("RotaryBoom.Angle", 128.00)
        ], 
        // Targets
        [
            // rope_len_bck,     rope_len_fwd
            (    0.00,           11362.11),
            (11362.11,            2261.27),
            ( 2261.27,            5481.52),
            ( 5481.52,            1128.33),
            ( 1128.33,             389.89),
            (  389.89,            1000.00),
        ]),
    ];

    // Блоки 0 | Схема 1 | L_block=11509.02 | Alpha_rope=-65.34° | L_rope=11509.01
    // Блоки 1 | Схема 1 | L_block=1722.44 | Alpha_rope=-65.46° | L_rope=1722.44
    // Блоки 2 | Схема 1 | L_block=5481.52 | Alpha_rope=-37.07° | L_rope=5481.52
    // Блоки 3 | Схема 2 | L_block=1392.59 | Alpha_rope=-4.48° | L_rope=1128.33
    // Блоки 4 | Схема 3 | L_block=904.54 | Alpha_rope=-11.15° | L_rope=389.89
    // Блоки 5 | Схема 1 | L_block=1080.07 | Alpha_rope=90.00° | L_rope=1000.00

    // Блоки 0 | Схема 1 | L_block=11362.12 | Alpha_rope=-69.61° | L_rope=11362.11
    // Блоки 1 | Схема 1 | L_block=2261.27 | Alpha_rope=-42.99° | L_rope=2261.27
    // Блоки 2 | Схема 1 | L_block=5481.52 | Alpha_rope=-14.06° | L_rope=5481.52
    // Блоки 3 | Схема 2 | L_block=1392.59 | Alpha_rope=18.53° | L_rope=1128.33
    // Блоки 4 | Схема 3 | L_block=904.54 | Alpha_rope=11.86° | L_rope=389.89
    // Блоки 5 | Схема 1 | L_block=1080.07 | Alpha_rope=90.00° | L_rope=1000.00

    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        rope:
            width: 35 mm            # Diameter of the rome
            length: 3000 m          # Total working length of the rope
            winch-length: 2985 m    # Length of the rope on the winch drum in the parking position, when rope pos is zero
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
    let mut rope_sections = LooseRopeSections::new(
        &dbg,
        Blocks::new(
            &dbg,
            &conf.blocks,
            Booms::new(&dbg, &conf.booms, &mut vec![]),
        ),
    );
    let t = Instant::now();
    for (step, events, target) in test_data {
        for (key, val) in events {
            log::debug!("{dbg} | step {step}  Event '{}': {:?}", key, val);
            inputs.insert(key.to_owned(), val);
        }
        let result = rope_sections.eval(&inputs).unwrap();
        log::trace!("{dbg} | step {step}  result: {:#?}", result);
        for (i, (rope_len_bck, rope_len_fwd)) in target.into_iter().enumerate() {
            assert!(result[i].rope_len_bck.aprox_eq(rope_len_bck, 2), "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", result[i].rope_len_bck, rope_len_bck);
            assert!(result[i].rope_len_fwd.aprox_eq(rope_len_fwd, 2), "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", result[i].rope_len_fwd, rope_len_fwd);
        }
        log::debug!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
    }
    test_duration.exit();
}
