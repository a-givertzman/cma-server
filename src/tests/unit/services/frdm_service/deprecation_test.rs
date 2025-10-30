#[cfg(test)]
use std::cell::RefCell;
use std::{fs::OpenOptions, rc::Rc, sync::{Arc, Once, atomic::AtomicBool}, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfTree;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, CraneConf, Deprecation, FrdmServiceConf, Inputs, LooseRopeSections, RopeDeprecationConf};

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
/// Representation of the csv data single row
#[derive(Debug, Clone, serde::Deserialize)]
struct CsvHeader {
    step: String,
    a21: String,       // a21, град, угол стрелы относительно предыдущей
    a22: String,       // a22, град, угол стрелы относительно предыдущей
    x_nok: String,     // Xнок, мм, - координата крайнего блока (5 или 6 блок в зависимости от переброса каната)
    y_nok: String,     // Yнок, мм, - координата крайнего блока (5 или 6 блок в зависимости от переброса каната)
    xg: String,        // XG, мм, - координаты конца стрелы
    yg: String,        // YG, мм, - координаты конца стрелы
    x_kp: String,       // Xкп, мм, - координаты Крюковой Подвески
    y_kp: String,       // Yкп, мм, - координаты Крюковой Подвески
    lpodv_min: String, // lподв_min, мм, - длина подвеса
    lkan_прям: String, // Lкан_прям, мм, - сумма длин прямолинейных участков каната 
    lkan_дуг: String,  // Lкан_дуг, мм, - сумма длин дуг каната
    lкан_леб: String,  // Lкан_леб,мм, - длина каната на лебедке
    x2: String,        // X2, мм, - координаты блока 2
    y2: String,        // Y2, мм, - координаты блока 2 
    x3: String,        // X3, мм, - координаты блока 3
    y3: String,        // Y3, мм, - координаты блока 3 
    x4: String,        // X4, мм, - координаты блока 4 
    y4: String,        // Y4, мм, - координаты блока 4 
    x5: String,        // X5, мм, - координаты блока 5 
    y5: String,        // Y5, мм, - координаты блока 5 
    x6: String,        // X6, мм, - координаты блока 6 
    y6: String,        // Y6, мм, - координаты блока 6 
}
///
/// Representation of the csv data single row
#[derive(Debug, Clone, serde::Deserialize)]
struct CsvRecord {
    step: usize,
    a21: f64,       // a21, град, угол стрелы относительно предыдущей
    a22: f64,       // a22, град, угол стрелы относительно предыдущей
    x_nok: f64,     // Xнок, мм,
    y_nok: f64,     // Yнок, мм,
    xg: f64,        // XG, мм,
    yg: f64,        // YG, мм,
    x_hook: f64,       // Xкп, мм,
    y_hook: f64,       // Yкп, мм,
    lrope_hook_min: f64, // lподв_min, мм,
    lrope_straight: f64, // Lкан_прям, мм,
    lrope_ark: f64,  // Lкан_дуг, мм,
    lrope_winch: f64,  // Lкан_леб,мм,
    x2: f64,        // X2, мм,
    y2: f64,        // Y2, мм,
    x3: f64,        // X3, мм,
    y3: f64,        // Y3, мм,
    x4: f64,        // X4, мм,
    y4: f64,        // Y4, мм,
    x5: f64,        // X5, мм,
    y5: f64,        // Y5, мм,
    x6: f64,        // X6, мм,
    y6: f64,        // Y6, мм

}
///
/// Testing [Deprecation]
#[test]
fn eval() {
    DebugSession::init(LogLevel::Debug, Backtrace::Short);
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Deprecation-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(30));
    test_duration.run().unwrap();
    let path = "src/tests/unit/services/frdm_service/deprecation_test.csv";
    log::debug!("{dbg} | reading csv: '{}'", path);
    let rdr = OpenOptions::new().read(true).open(path).unwrap();
    let mut rdr = csv::Reader::from_reader(rdr);
    log::debug!("{dbg} | Parse csv data...");
    let mut csv: csv::DeserializeRecordsIter<'_, _, CsvRecord> = rdr.deserialize();
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
    // let test_data = [
    //     //    input values                            rope slices deprecetion
    //     //    pos, m                                  slice[0]  slice[1]  slice[2]  count of dep's
    //     //    load, tonn
    //     //    angle, degree
    //     (01,  "MainBoom.Angle",           69.710,     vec![ 0.00,     0.00,     0.00],     0),
    //     (02,  "RotaryBoom.Angle",        155.300,     vec![ 0.00,     0.00,     0.00],     0),
    //     (03,  "Winch.Pos",                 0.000,     vec![ 0.00,     0.00,     0.00],     0),
    //     (04,  "Winch.Load",                1.000,     vec![ 3.33,     0.00,     3.33],     2),
    //     (05,  "Winch.Pos",                 0.002,     vec![ 0.00,     0.00,     0.00],     0),
    //     (06,  "Winch.Pos",                 0.003,     vec![ 0.00,     0.00,     0.00],     0),
    //     (06,  "Winch.Pos",                 0.006,     vec![ 0.00,     0.00,     0.00],     0),
    //     (06,  "Winch.Pos",                 0.008,     vec![ 0.00,     0.00,     0.00],     0),
    //     (07,  "Winch.Pos",                 0.010,     vec![ 0.00,     0.00,     0.00],     0),
    //     (08,  "Winch.Pos",                 0.012,     vec![ 0.00,     0.00,     0.00],     0),
    //     (08,  "Winch.Pos",                 0.014,     vec![ 0.00,     0.00,     0.00],     0),
    //     (08,  "Winch.Pos",                 0.016,     vec![ 0.00,     0.00,     0.00],     0),
    //     (08,  "Winch.Pos",                 0.018,     vec![ 0.00,     0.00,     0.00],     0),
    //     (09,  "Winch.Pos",                 0.020,     vec![ 0.00,     0.00,     0.00],     0),
    //     (10,  "Winch.Pos",                 0.022,     vec![ 0.00,     0.00,     0.00],     0),
    //     (10,  "Winch.Pos",                 0.024,     vec![ 0.00,     0.00,     0.00],     0),
    //     (10,  "Winch.Pos",                 0.026,     vec![ 0.00,     0.00,     0.00],     0),
    //     (10,  "Winch.Pos",                 0.028,     vec![ 0.00,     0.00,     0.00],     0),
    //     (11,  "Winch.Pos",                 0.030,     vec![ 0.00,     0.00,     0.00],     0),
    //     (12,  "Winch.Pos",                 0.032,     vec![ 0.00,     0.00,     0.00],     0),
    //     (12,  "Winch.Pos",                 0.034,     vec![ 0.00,     0.00,     0.00],     0),
    //     (12,  "Winch.Pos",                 0.036,     vec![ 0.00,     0.00,     0.00],     0),
    //     (12,  "Winch.Pos",                 0.038,     vec![ 0.00,     0.00,     0.00],     0),
    //     (13,  "Winch.Pos",                 0.040,     vec![ 0.00,     0.00,     0.00],     0),
    //     (14,  "Winch.Pos",                 0.042,     vec![ 0.00,     0.00,     0.00],     0),
    //     (14,  "Winch.Pos",                 0.044,     vec![ 0.00,     0.00,     0.00],     0),
    //     (14,  "Winch.Pos",                 0.046,     vec![ 0.00,     0.00,     0.00],     0),
    //     (14,  "Winch.Pos",                 0.048,     vec![ 0.00,     0.00,     0.00],     0),
    //     (15,  "Winch.Pos",                 0.050,     vec![ 0.00,     0.00,     0.00],     0),
    //     (16,  "Winch.Pos",                 0.052,     vec![ 0.00,     0.00,     0.00],     0),
    //     (16,  "Winch.Pos",                 0.054,     vec![ 0.00,     0.00,     0.00],     0),
    //     (16,  "Winch.Pos",                 0.056,     vec![ 0.00,     0.00,     0.00],     0),
    //     (16,  "Winch.Pos",                 0.058,     vec![ 0.00,     0.00,     0.00],     0),
    //     (17,  "Winch.Pos",                 0.060,     vec![ 0.00,     0.00,     0.00],     0),
    //     (18,  "Winch.Pos",                 0.062,     vec![ 0.00,     0.00,     0.00],     0),
    //     (18,  "Winch.Pos",                 0.064,     vec![ 0.00,     0.00,     0.00],     0),
    //     (18,  "Winch.Pos",                 0.066,     vec![ 0.00,     0.00,     0.00],     0),
    //     (18,  "Winch.Pos",                 0.068,     vec![ 0.00,     0.00,     0.00],     0),
    //     (19,  "Winch.Pos",                 0.070,     vec![ 0.00,     0.00,     0.00],     0),
    // ];
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
            segment: 10 mm         # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
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
                d: 844 mm                   # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Fixed                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 2:
                lf: 308.0 mm, 1100.0 mm     # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816 mm                   # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 0                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 3:
                lf: 1435.0 mm, 1730.0 mm   # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816 mm                   # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 4:
                lf: -1121.0 mm, 973.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816 mm                   # Диаметр блока, мм
                scheme: TopBottom           # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 5:
                lf: 267.0 mm, 860.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816 mm                   # Диаметр блока, мм
                scheme: BottomTop           # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 6:
                lf: 136.0 mm, -35.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816 mm                   # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - 7:
                lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 0.0 mm                   # Диаметр блока, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе    ").unwrap());
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
    let mut deprecation = Deprecation::new(
        &dbg,
        &conf,
        inputs.clone(),
        Bendings::new(
            &dbg,
            &conf.rope,
            BlockArcs::new(
                &dbg,
                LooseRopeSections::new(
                    &dbg,
                    Blocks::new(
                        &dbg,
                        &conf.blocks,
                        Booms::new(&dbg, &conf.booms, inputs.clone()),
                    ),
                ),
            ),
        ),
        |slice_ix, deprecation| {
            // let dbg = &dbg.clone();
            // log::debug!("{dbg} | Deprecation slice[{slice_ix}]: {:?}", deprecation);
            result.replace_with(|r| {
                r[*slice_ix] += deprecation;
                r.to_owned()
            });
            result_count.replace_with(|r| {
                *r + 1
            });
        },
    );
    inputs.insert("Winch.Pos", 0.0);
    inputs.insert("Winch.Load", 0.0);
    for row in csv {
        let row: CsvRecord = row.unwrap();
        let step = row.step;
        inputs.insert("MainBoom.Angle", row.a21);
        inputs.insert("RotaryBoom.Angle", row.a22);
        let time = Instant::now();
        deprecation.eval();
        let r = result.borrow();
        let r: Vec<&f64> = r
            .iter()
            .filter(|v| **v > 0.0)
            .collect();
        log::debug!("{dbg} | step {step} result: {:?}", r);
        log::debug!("{dbg} | step {step} elapsed: {:?}", time.elapsed());
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
