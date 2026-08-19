#[cfg(test)]
use std::cell::RefCell;
use std::{fs::OpenOptions, rc::Rc, sync::{Arc, Once, atomic::AtomicBool}, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfTree;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{services::{Bendings, BlockArcs, Blocks, Booms, CraneConf, Deprecation, FrdmServiceConf, Inputs, RopeDeprecationConf, RopeSections}, tests::{tools::{SeriesKind, plot}, unit::services::frdm_service::CsvRecord}};

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
/// Testing [Deprecation]
#[test]
fn eval() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    _ = std::process::Command::new("clear").status();
    log::debug!("");
    let dbg = Dbg::own("Deprecation-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(30));
    test_duration.run().unwrap();
    let path = "src/tests/unit/services/frdm_service/ysz-deprecation_test.csv";
    log::debug!("{dbg} | reading csv: '{}'", path);
    let rdr = OpenOptions::new().read(true).open(path).unwrap();
    let mut rdr = csv::Reader::from_reader(rdr);
    log::debug!("{dbg} | Parse csv data...");
    let csv: csv::DeserializeRecordsIter<'_, _, CsvRecord> = rdr.deserialize();
    // log::debug!("{dbg} | csv header: '{:?}'", csv.next().unwrap());
    // for result in csv {
    //     // log::debug!("{dbg} | csv record: '{}'", path);
    //     // Notice that we need to provide a type hint for automatic
    //     // deserialization.
    //     let record: CsvRecord = result.unwrap();
    //     println!("{:?}", record);
    // }
    // let csv_data =
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
    let test_data: &[(i32, &str, f64, Vec<f64>, i32)] = &[
    //     //    input values                            rope slices deprecetion
    //     //    pos, m                                  slice[0]  slice[1]  slice[2]  count of dep's
    //     //    load, tonn
    //     //    angle, degree
    //     (01,  "MainBoom.Angle",           69.710,     vec![ 0.00,     0.00,     0.00],     0),
    //     (02,  "RotaryBoom.Angle",        155.300,     vec![ 0.00,     0.00,     0.00],     0),
    //     (03,  "Winch.Pos",                 0.000,     vec![ 0.00,     0.00,     0.00],     0),
        (04,  "Winch.Load",                1.000,     vec![ 3.33,     0.00,     3.33],     2),
        (05,  "Winch.Pos",                 0.02,     vec![ 0.00,     0.00,     0.00],     0),
        (06,  "Winch.Pos",                 0.03,     vec![ 0.00,     0.00,     0.00],     0),
        (06,  "Winch.Pos",                 0.06,     vec![ 0.00,     0.00,     0.00],     0),
        (06,  "Winch.Pos",                 0.08,     vec![ 0.00,     0.00,     0.00],     0),
        (07,  "Winch.Pos",                 0.10,     vec![ 0.00,     0.00,     0.00],     0),
        (08,  "Winch.Pos",                 0.12,     vec![ 0.00,     0.00,     0.00],     0),
        (08,  "Winch.Pos",                 0.14,     vec![ 0.00,     0.00,     0.00],     0),
        (08,  "Winch.Pos",                 0.16,     vec![ 0.00,     0.00,     0.00],     0),
        (08,  "Winch.Pos",                 0.18,     vec![ 0.00,     0.00,     0.00],     0),
        (09,  "Winch.Pos",                 0.20,     vec![ 0.00,     0.00,     0.00],     0),
        (10,  "Winch.Pos",                 0.22,     vec![ 0.00,     0.00,     0.00],     0),
        (10,  "Winch.Pos",                 0.24,     vec![ 0.00,     0.00,     0.00],     0),
        (10,  "Winch.Pos",                 0.26,     vec![ 0.00,     0.00,     0.00],     0),
        (10,  "Winch.Pos",                 0.28,     vec![ 0.00,     0.00,     0.00],     0),
        (11,  "Winch.Pos",                 0.30,     vec![ 0.00,     0.00,     0.00],     0),
        (12,  "Winch.Pos",                 0.32,     vec![ 0.00,     0.00,     0.00],     0),
        (12,  "Winch.Pos",                 0.34,     vec![ 0.00,     0.00,     0.00],     0),
        (12,  "Winch.Pos",                 0.36,     vec![ 0.00,     0.00,     0.00],     0),
        (12,  "Winch.Pos",                 0.38,     vec![ 0.00,     0.00,     0.00],     0),
        (13,  "Winch.Pos",                 0.40,     vec![ 0.00,     0.00,     0.00],     0),
        (14,  "Winch.Pos",                 0.42,     vec![ 0.00,     0.00,     0.00],     0),
        (14,  "Winch.Pos",                 0.44,     vec![ 0.00,     0.00,     0.00],     0),
        (14,  "Winch.Pos",                 0.46,     vec![ 0.00,     0.00,     0.00],     0),
        (14,  "Winch.Pos",                 0.48,     vec![ 0.00,     0.00,     0.00],     0),
        (15,  "Winch.Pos",                 0.50,     vec![ 0.00,     0.00,     0.00],     0),
        (16,  "Winch.Pos",                 0.52,     vec![ 0.00,     0.00,     0.00],     0),
        (16,  "Winch.Pos",                 0.54,     vec![ 0.00,     0.00,     0.00],     0),
        (16,  "Winch.Pos",                 0.56,     vec![ 0.00,     0.00,     0.00],     0),
        (16,  "Winch.Pos",                 0.58,     vec![ 0.00,     0.00,     0.00],     0),
        (17,  "Winch.Pos",                 0.60,     vec![ 0.00,     0.00,     0.00],     0),
        (18,  "Winch.Pos",                 0.62,     vec![ 0.00,     0.00,     0.00],     0),
        (18,  "Winch.Pos",                 0.64,     vec![ 0.00,     0.00,     0.00],     0),
        (18,  "Winch.Pos",                 0.66,     vec![ 0.00,     0.00,     0.00],     0),
        (18,  "Winch.Pos",                 0.68,     vec![ 0.00,     0.00,     0.00],     0),
        (19,  "Winch.Pos",                 0.70,     vec![ 0.00,     0.00,     0.00],     0),
    ];
    let mut target: Vec<f64> = vec![];
    let mut target_count = 0;
    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        rope:
            width: 35 mm               # Diameter of the rome
            length: 85.045 m           # Total working length of the rope
            aux-length: 1.200 m        # Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
            segment: 100 mm            # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
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
                parking: 0.0 deg            # Угол в парковочном положении, град
            - Rotary-Boom:
                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                l4: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                len: 7984.0 mm                                          # length of the rotary boom
                angle: point real 'RotaryBoom.Angle' # degrees, current angle of the boom (relative axis)
                parking: 23.78 deg          # Угол в парковочном положении, град
        blocks:
            - 1:
                lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 845.000 mm               # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Drum                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 2:
                lf: 308.0 mm, 1100.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.000 mm               # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 0                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 3:
                lf: -6549.0 mm, 1730.0 mm   # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.000 mm               # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 4:
                lf: -1121.0 mm, 973.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.000 mm               # Диаметр блока, мм
                scheme: TopBottom           # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 5:
                lf: 267.0 mm, 860.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.000 mm               # Диаметр блока, мм
                scheme: BottomTop           # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 6:
                lf: 136.0 mm, -35.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.000 mm               # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                deflector-angle: 90 deg     # Угол перекидывания, град. Блок включается в работу только когда стрела проходит положение перекидывания.
            - 7:
                lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 0.0 mm                   # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
    ").unwrap());
    let conf = CraneConf::new(&dbg, conf);
    log::trace!("{dbg} | conf: {:#?}", conf);
    let result = Rc::new(RefCell::new(vec![0.00; (conf.rope.length.as_m() / conf.rope.segment.as_m()) as usize]));
    let result_count = Rc::new(RefCell::new(0));
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
    let parking = true;
    let mut deprecation = Deprecation::new(
        &dbg,
        &conf,
        inputs.clone(),
        Bendings::new(
            &dbg,
            &conf.rope,
            BlockArcs::new(
                &dbg,
                &conf.rope.segment,
                RopeSections::new(
                    &dbg,
                    Blocks::new(
                        &dbg,
                        conf.rope.aux_length,
                        &conf.blocks,
                        parking,
                        Booms::new(&dbg, &conf.booms, inputs.clone(), parking),
                    ),
                ),
            ),
        ),
        |slice_ix, deprecation| {
            // let dbg = &dbg.clone();
            // log::debug!("{dbg} | Deprecation slice[{slice_ix}]: {:?}", deprecation);
            result.replace_with(|r| {
                r[slice_ix] += deprecation;
                r.to_owned()
            });
            result_count.replace_with(|r| {
                *r + 1
            });
        },
    );
    inputs.insert("Winch.Pos", 0.0);
    inputs.insert("Winch.Load", 0.0);
    for row in csv.take(1) {
        let row: CsvRecord = row.unwrap();
        let step = row.step;
        inputs.insert("MainBoom.Angle", row.a21);
        inputs.insert("RotaryBoom.Angle", row.a22);
        let time = Instant::now();
        deprecation.eval();
        for (_, event_name, event_value, _, _) in test_data {
            inputs.insert(event_name.to_string(), *event_value);
            let t = Instant::now();
            deprecation.eval();
            log::debug!("{dbg} | step {step} | Rope pos {event_value} | Elapsed: {:?}", t.elapsed());
        }
        let elapsed = time.elapsed();
        let r = result.borrow();
        let r: Vec<(usize, &f64)> = r
            .iter().enumerate()
            .filter_map(|(i, v)| (v.abs() > 0.0).then(|| (i, v)))
            .collect();
        log::debug!("{dbg} | step {step} elapsed: {:?}, result: {:?}", elapsed, r);
        let path = format!("src/tests/unit/services/frdm_service/deprecation_{step}.png");
        let series = result.borrow().iter().enumerate().map(|(x, y)| (x as f64 * conf.rope.segment.as_m(), *y)).collect();
        if let Err(err) = plot(&path, None, vec![series], SeriesKind::Points) {
            log::debug!("{dbg} | step {step} Can't write chart to '{path}', error: {:?}", err);
        }
    }

    // for (step, event_name, event_value, target_i, target_count_i) in test_data {
    //     // target = target_i;
    //     // target_count = target_count_i;
    //     log::debug!("{dbg} | step {step}  Event '{}': {:.4}", event_name, event_value);
    //     let time = Instant::now();
    //     inputs.insert(event_name, event_value);
    //     deprecation.eval();
    //     log::debug!("{dbg} | step {step} elapsed: {:?}", time.elapsed());
    //     // assert!(
    //     //     result.borrow().iter().enumerate().all(|(ix, r)| {
    //     //         log::trace!("{dbg} | step {step} result: {}, target: {},  test: {}", r.round(), target[ix].round(), r.round() == target[ix].round());
    //     //         r.round() == target[ix].round()
    //     //     }),
    //     //     "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result.borrow().to_vec(), target,
    //     // );
    // }
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
