#[cfg(test)]
use std::cell::RefCell;
use std::{rc::Rc, sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::ConfTree, entity::ToPoint};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, CraneConf, Deprecation, LooseRopeSections};

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
    let dbg = Dbg::own("Deprication-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    //
    // rope pos                   blk[0] blk[1]
    //                             .5     .7  
    //                             ◯     ◯
    // 0.5                        ⎯  ⎯  ⎯  ⎯  ⎯  ⎯  ⎯  ⎯
    //                            0   1   2  3   4   5   6  7
    //                             ◯     ◯
    // 0.6                           ⎯  ⎯  ⎯  ⎯  ⎯  ⎯  ⎯  ⎯
    //                               0   1   2  3   4   5   6  7
    // 
    //                             ◯     ◯
    // 0.7                              ⎯  ⎯  ⎯  ⎯  ⎯  ⎯  ⎯  ⎯
    //                                  0   1   2  3   4   5   6  7
    // 
    //                             ◯     ◯
    // 0.71                               ⎯  ⎯  ⎯  ⎯  ⎯  ⎯  ⎯  ⎯
    //                                    0   1   2  3   4   5   6  7
    // 
    let test_data = [
        //    input values                            rope slices deprecetion
        //    pos, m                                  slice[0]  slice[1]  slice[2]  count of dep's
        //    load, tonn
        //    angle, degree
        (01,  "MainBoom.Angle",           69.71,      vec![ 0.00,     0.00,     0.00],     0),
        (02,  "RotaryBoom.Angle",        155.30,      vec![ 0.00,     0.00,     0.00],     0),
        (03,  "Winch.Pos",                 0.00,      vec![ 0.00,     0.00,     0.00],     0),
        (04,  "Winch.Load",                1.00,      vec![ 3.33,     0.00,     3.33],     2),
        (05,  "Winch.Pos",                 0.01,      vec![ 0.00,     0.00,     0.00],     0),
        // (03,  "          ", 0.51,      vec![ 3.33,     0.00,     3.33],     2),
        // (04,  "          ", 0.52,      vec![ 3.33,     0.00,     3.33],     2),
        // (05,  "          ", 0.53,      vec![ 3.33,     0.00,     3.33],     2),
        // (06,  "          ", 0.54,      vec![ 3.33,     0.00,     3.33],     2),
        // (07,  "          ", 0.55,      vec![ 3.33,     0.00,     3.33],     2),
        // (08,  "          ", 0.56,      vec![ 3.33,     0.00,     3.33],     2),
        // (09,  "          ", 0.57,      vec![ 3.33,     0.00,     3.33],     2),
        // (10,  "          ", 0.58,      vec![ 3.33,     0.00,     3.33],     2),
        // (11,  "          ", 0.59,      vec![ 3.33,     0.00,     3.33],     2),
        // (12,  "          ", 0.60,      vec![ 6.66,     3.33,     6.66],     5),
        // (13,  "          ", 0.61,      vec![ 6.66,     3.33,     6.66],     5),
        // (14,  "          ", 0.62,      vec![ 6.66,     3.33,     6.66],     5),
        // (15,  "          ", 0.63,      vec![ 6.66,     3.33,     6.66],     5),
        // (16,  "          ", 0.64,      vec![ 6.66,     3.33,     6.66],     5),
        // (17,  "          ", 0.65,      vec![ 6.66,     3.33,     6.66],     5),
        // (18,  "          ", 0.66,      vec![ 6.66,     3.33,     6.66],     5),
        // (19,  "          ", 0.67,      vec![ 6.66,     3.33,     6.66],     5),
        // (20,  "          ", 0.68,      vec![ 6.66,     3.33,     6.66],     5),
        // (21,  "          ", 0.69,      vec![ 6.66,     3.33,     6.66],     5),
        // (22,  "          ", 0.70,      vec![ 9.99,     3.33,     6.66],     6),
        // (23,  "          ", 0.71,      vec![ 9.99,     6.66,     6.66],     7),
        // (24,  "          ", 0.72,      vec![ 9.99,     6.66,     6.66],     7),
    ];
    let mut target: Vec<f64> = vec![];
    let mut target_count = 0;
    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        bendings:           # Rope bloks with diameter, inter and exit
            # Block Diameter   inter   exit
            - D200mm           5.0  .. 5.15 m
            - D300mm           7.23 .. 7.30 mm
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
                len: 7984.1 mm                                          # length of the rotary boom
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
                bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе    ").unwrap());
    let conf = CraneConf::new(&dbg, conf);
    log::trace!("{dbg} | conf: {:#?}", conf);
    let result = Rc::new(RefCell::new(vec![0.00, 0.00, 0.00]));
    let result_count = Rc::new(RefCell::new(0));
    let mut subscriptions = vec![];
    let mut deprecation = Deprecation::new(
        &dbg,
        &conf,
        Bendings::new(
            &dbg,
            conf.rope.pos.clone(),
            &conf.rope,
            BlockArcs::new(
                &dbg,
                LooseRopeSections::new(
                    &dbg,
                    Blocks::new(
                        &dbg,
                        &conf.blocks,
                        Booms::new(&dbg, &conf.booms, &mut subscriptions),
                    ),
                ),
            ),
        ),
        subscriptions,
        |slice_ix, deprecation| {
            let dbg = &dbg.clone();
            log::debug!("{dbg} | Deprication slice[{slice_ix}]: {:?}", deprecation);
            result.replace_with(|r| {
                r[slice_ix] += deprecation;
                r.to_owned()
            });
            result_count.replace_with(|r| {
                *r + 1
            });
        },
    );
    for (step, event_name, event_value, target_i, target_count_i) in test_data {
        target = target_i;
        target_count = target_count_i;
        log::debug!("{dbg} | step {step}  Event '{}': {:.4}", event_name, event_value);
        let time = Instant::now();
        deprecation.eval(&event_value.to_point(0, event_name));
        log::debug!("{dbg} | step {step} elapsed: {:?}", time.elapsed());
        // assert!(
        //     result.borrow().iter().enumerate().all(|(ix, r)| {
        //         log::trace!("{dbg} | step {step} result: {}, target: {},  test: {}", r.round(), target[ix].round(), r.round() == target[ix].round());
        //         r.round() == target[ix].round()
        //     }),
        //     "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result.borrow().to_vec(), target,
        // );
    }
    // assert!(*result_count.borrow() == target_count, "{dbg} | \nresult: {:?}\ntarget: {:?}", result_count.borrow(), target_count);
    // let result = result.borrow().to_vec();
    // assert!(
    //     result.iter().enumerate().all(|(ix, r)| {
    //         log::debug!("{dbg} | result: {}, target: {},  test: {}", r.round(), target[ix].round(), r.round() == target[ix].round());
    //         r.round() == target[ix].round()
    //     }),
    //     "{dbg} | \nresult: {:?}\ntarget: {:?}", result, target,
    // );
    test_duration.exit();
}
