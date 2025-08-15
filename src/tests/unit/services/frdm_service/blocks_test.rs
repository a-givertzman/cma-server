#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::{collections::FxIndexMap, services::conf::ConfTree};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::{Blocks, Booms, CraneConf};

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
/// Testing [RopeSlices]
#[test]
fn new() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Blocks-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        (01,  [
            // Input Events
            ("Load.MainBoomAngle",    69.71),
            ("Load.RotaryBoomAngle", 155.30)
        ], 
        // Targets
        [
            // block.x,           block.y
            (-1829.9999999999993, 11040.0),
            (2958.9072250002687, 21505.37167318569),
            (3674.151798369415, 23072.283377861302),
            (8047.737692695379, 26376.6496446291),
            (9108.947602987579, 27278.396020445285),
            (9649.303797749028, 26552.998761846295),
            (10057.303797749028, 25552.998761846295),
        ]),
        (01,  [
            // Input Events
            ("Load.MainBoomAngle",    74.00),
            ("Load.RotaryBoomAngle", 128.00)
        ], 
        // Targets
        [
            // block.x,           block.y
            (-1829.9999999999993, 11040.0),
            (2114.646825209876, 21695.400688256872),
            (3768.650625989636, 23237.344917868133),
            (9085.90896364857, 24569.20593561606),
            (10415.170698843269, 24984.388111711298),
            (10628.98251500226, 24105.485098136538),
            (11036.98251500226, 23105.485098136538),
        ]),
    ];
    //  Число стрел: 2
    //  alpha_boom: [69.71, 45.01]
    //  Стрела 1: D=(6.32530071759608e-13, 10330.0), G=(3883.8458824556724, 20835.034086633517)
    //  Стрела 2: D=(3883.8458824556724, 20835.034086633517), G=(9528.40100476262, 26481.55987434036)
    //  Блок 1: (-1829.9999999999993, 11040.0)
    //  Блок 2: (2958.9072250002687, 21505.37167318569)
    //  Блок 3: (3674.151798369415, 23072.283377861302)
    //  Блок 4: (8047.737692695379, 26376.6496446291)
    //  Блок 5: (9108.947602987579, 27278.396020445285)
    //  Блок 6: (9649.303797749028, 26552.998761846295)
    //  Блок 7: (10057.303797749028, 25552.998761846295)
    //  Блоки (1, 2) | Схема 1 | L_block=11509.02 | Alpha_rope=-65.34° | L_rope=11509.02
    //  Блоки (2, 3) | Схема 1 | L_block=1722.44 | Alpha_rope=-65.46° | L_rope=1722.44
    //  Блоки (3, 4) | Схема 1 | L_block=5481.52 | Alpha_rope=-37.07° | L_rope=5481.52
    //  Блоки (4, 5) | Схема 2 | L_block=1392.59 | Alpha_rope=-4.49° | L_rope=1128.48
    //  Блоки (5, 6) | Схема 3 | L_block=904.54 | Alpha_rope=-11.12° | L_rope=390.29
    //  Блоки (6, 7) | Схема 1 | L_block=1080.03 | Alpha_rope=90.00° | L_rope=1000.00

    //  Число стрел: 2
    //  alpha_boom: [74.0, 22.0]
    //  Стрела 1: D=(6.32530071759608e-13, 10330.0), G=(3087.138385150391, 21096.13099450917)
    //  Стрела 2: D=(3087.138385150391, 21096.13099450917), G=(10489.774280011621, 24086.990036341813)
    //  Блок 1: (-1829.9999999999993, 11040.0)
    //  Блок 2: (2114.646825209876, 21695.400688256872)
    //  Блок 3: (3768.650625989636, 23237.344917868133)
    //  Блок 4: (9085.90896364857, 24569.20593561606)
    //  Блок 5: (10415.170698843269, 24984.388111711298)
    //  Блок 6: (10628.98251500226, 24105.485098136538)
    //  Блок 7: (11036.98251500226, 23105.485098136538)
    //  Блоки (1, 2) | Схема 1 | L_block=11362.12 | Alpha_rope=-69.61° | L_rope=11362.11
    //  Блоки (2, 3) | Схема 1 | L_block=2261.27 | Alpha_rope=-42.99° | L_rope=2261.27
    //  Блоки (3, 4) | Схема 1 | L_block=5481.52 | Alpha_rope=-14.06° | L_rope=5481.52
    //  Блоки (4, 5) | Схема 2 | L_block=1392.59 | Alpha_rope=18.52° | L_rope=1128.48
    //  Блоки (5, 6) | Схема 3 | L_block=904.54 | Alpha_rope=11.89° | L_rope=390.29
    //  Блоки (6, 7) | Схема 1 | L_block=1080.03 | Alpha_rope=90.00° | L_rope=1000.00
    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        bendings:           # Rope bloks with diameter, inter and exit
            # Block Diameter   inter   exit
            - D200mm           5.0  .. 5.15 m
            - D300mm           7.23 .. 7.30 mm
        boom:
            main-len: 5.3 m                                        # length of the main boom
            main-angle: point real '/App/Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
            rotary-len: 2.1 m                                      # length of the rotary boom
            rotary-angle: point real '/App/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
        booms:
            - Main-Boom:
                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                len: 11200.0 mm                                         # length of the boom
                angle: point real 'Load.MainBoomAngle'   # degrees, current angle of the boom (relative axis)
            - Rotary-Boom:
                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                l4: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                len: 7984.0 mm                                          # length of the rotary boom
                angle: point real 'Load.RotaryBoomAngle' # degrees, current angle of the boom (relative axis)
        blocks:
            - '1':
                lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 844.0 mm                 # Диаметры блоков, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Fixed                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - '2':
                lf: 308.0 mm, 1100.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
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
            segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
            pos: point real '/App/Winch.EncoderBR2'      # meters, current rope position
            load: point real '/App/Winch.Load'          # tonn, current rope load
    ").unwrap());
    let conf = CraneConf::new(&dbg, conf);
    let mut inputs = FxIndexMap::default();
    let mut blocks = Blocks::new(
        &dbg,
        &conf.blocks,
        Booms::new(&dbg, &conf.booms, &mut inputs),
    );
    let t = Instant::now();
    for (step, events, target) in test_data {
        for (key, val) in events {
            log::debug!("{dbg} | step {step}  Event '{}': {:?}", key, val);
            inputs.insert(key.to_owned(), val);
        }
        let result = blocks.eval(&inputs).unwrap();
        log::trace!("{dbg} | step {step}  result: {:#?}", result);
        for (i, (target_x, target_y)) in target.into_iter().enumerate() {
            assert!(result[i].pos.x == target_x, "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", result[i].pos.x, target_x);
            assert!(result[i].pos.y == target_y, "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", result[i].pos.y, target_y);
        }
        log::debug!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
    }
    test_duration.exit();
}
