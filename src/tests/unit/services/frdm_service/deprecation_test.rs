#[cfg(test)]
use std::cell::RefCell;
use std::{fs::OpenOptions, rc::Rc, sync::{Arc, Once, atomic::AtomicBool}, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfTree;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::{DebugSession, LogLevel};
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
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    _ = std::process::Command::new("clear").status();
    log::debug!("");
    let dbg = Dbg::own("Deprecation-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(30));
    test_duration.run().unwrap();
    let test_data: &[(i32, &str, f64, Vec<(usize, f64)>)] = &[
        //    input values                            rope slices deprecetion
        //    pos, m                                  [(slice-index, deprecation)]
        //    load, tonn
        //    angle, degree
        (01,  "MainBoom.Angle",            0.000,     vec![]),
        (02,  "RotaryBoom.Angle",         23.780,     vec![]),
        (03,  "Winch.Pos",                 0.000,     vec![]),
        (04,  "Winch.Load",                1.000,     vec![]),
        (05,  "Winch.Pos",                 0.020,     vec![]),
        (06,  "Winch.Pos",                 0.030,     vec![(583, 1.183), (581, 1.183), (757, 1.225), (816, 1.225)]),
        (06,  "Winch.Pos",                 0.060,     vec![(724, 1.225), (762, 1.225), (819, 1.225), (838, 1.225)]),
        (06,  "Winch.Pos",                 0.080,     vec![(715, 1.225)]),
        (07,  "Winch.Pos",                 0.100,     vec![(829, 1.225)]),
        (08,  "Winch.Pos",                 0.120,     vec![]),
        (08,  "Winch.Pos",                 0.140,     vec![(582, 1.183), (580, 1.183), (723, 1.225), (756, 1.225), (815, 1.225)]),
        (08,  "Winch.Pos",                 0.160,     vec![(761, 1.225), (818, 1.225), (837, 1.225)]),
        (08,  "Winch.Pos",                 0.180,     vec![(714, 1.225)]),
        (09,  "Winch.Pos",                 0.200,     vec![(828, 1.225)]),
        (10,  "Winch.Pos",                 0.220,     vec![]),
        (10,  "Winch.Pos",                 0.240,     vec![(581, 1.183), (579, 1.183), (722, 1.225), (755, 1.225), (814, 1.225)]),
        (10,  "Winch.Pos",                 0.260,     vec![(760, 1.225), (817, 1.225), (836, 1.225)]),
        (10,  "Winch.Pos",                 0.280,     vec![(713, 1.225)]),
        (11,  "Winch.Pos",                 0.300,     vec![(827, 1.225)]),
        (12,  "Winch.Pos",                 0.320,     vec![]),
        (12,  "Winch.Pos",                 0.340,     vec![(580, 1.183), (578, 1.183), (721, 1.225), (754, 1.225), (813, 1.225)]),
        (12,  "Winch.Pos",                 0.360,     vec![(759, 1.225), (816, 1.225), (835, 1.225)]),
        (12,  "Winch.Pos",                 0.380,     vec![(712, 1.225)]),
        (13,  "Winch.Pos",                 0.400,     vec![(826, 1.225)]),
        (14,  "Winch.Pos",                 0.420,     vec![]),
        (14,  "Winch.Pos",                 0.440,     vec![(579, 1.183), (577, 1.183), (720, 1.225), (753, 1.225), (812, 1.225)]),
        (14,  "Winch.Pos",                 0.460,     vec![(758, 1.225), (815, 1.225), (834, 1.225)]),
        (14,  "Winch.Pos",                 0.480,     vec![(711, 1.225)]),
        (15,  "Winch.Pos",                 0.500,     vec![(825, 1.225)]),
        (16,  "Winch.Pos",                 0.520,     vec![]),
        (16,  "Winch.Pos",                 0.540,     vec![(578, 1.183), (576, 1.183), (719, 1.225), (752, 1.225), (811, 1.225)]),
        (16,  "Winch.Pos",                 0.560,     vec![(757, 1.225), (814, 1.225), (833, 1.225)]),
        (16,  "Winch.Pos",                 0.580,     vec![(710, 1.225)]),
        (17,  "Winch.Pos",                 0.600,     vec![(824, 1.225)]),
        (18,  "Winch.Pos",                 0.620,     vec![]),
        (18,  "Winch.Pos",                 0.640,     vec![(577, 1.183), (575, 1.183), (718, 1.225), (751, 1.225), (810, 1.225)]),
        (18,  "Winch.Pos",                 0.660,     vec![(756, 1.225), (813, 1.225), (832, 1.225)]),
        (18,  "Winch.Pos",                 0.680,     vec![(709, 1.225)]),
        (19,  "Winch.Pos",                 0.700,     vec![(823, 1.225)]),
    ];
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
    let result = Rc::new(RefCell::new(vec![0.00; conf.rope.slices()]));
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
    let mut deprecation = Deprecation::new(&dbg, &conf,
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
            log::debug!("{dbg} | Deprecation slice[{slice_ix}]: {:?}", deprecation);
            result.replace_with(|r| {
                r[slice_ix] += deprecation;
                r.to_owned()
            });
        },
    );
    inputs.insert("Winch.Pos", 0.0);
    inputs.insert("Winch.Load", 0.0);
    for (step, event_name, event_value, target) in test_data {
        // target = target_i;
        // target_count = target_count_i;
        log::debug!("{dbg} | step {step}  Event '{}': {:.4}", event_name, event_value);
        let time = Instant::now();
        inputs.insert(*event_name, *event_value);
        deprecation.eval();
        log::debug!("{dbg} | step {step} elapsed: {:?}", time.elapsed());
        if target.is_empty() {
            let res: f64 = result.borrow_mut().iter().sum();
            assert!(res < f64::EPSILON , "{dbg} | step {step} | Износа на этом шаге быть не должно: \n result: {:?}\n target: {:?}", res, 0.0);
        } else {
            let res: Vec<_> = result.borrow_mut().clone();
            for (slice, dep) in target {
                assert!((res[*slice] - *dep).abs() < 0.001 , "{dbg} | step {step} | slice {slice} | \n result: {:?}\n target: {:?}", res[*slice], target);
            }
        }
        result.borrow_mut().fill(0.00);
    }
    test_duration.exit();
}
///
/// Test Deprecation on csv data
#[ignore = "Isn't implemented yet"]
#[test]
fn test_eval_on_csv() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
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
    let test_data: &[(i32, &str, f64, Vec<(usize, f64)>)] = &[
        //    input values                            rope slices deprecetion
        //    pos, m                                  [(slice-index, deprecation)]
        //    load, tonn
        //    angle, degree
        // (01,  "MainBoom.Angle",           69.710,     0,    vec![]),
        // (02,  "RotaryBoom.Angle",        155.300,     0,    vec![]),
        // (03,  "Winch.Pos",                 0.000,     0,    vec![]),
        (04,  "Winch.Load",                1.000,     vec![]),
        (05,  "Winch.Pos",                 0.020,     vec![]),
        (06,  "Winch.Pos",                 0.030,     vec![(583, 1.183), (581, 1.183), (757, 1.225), (816, 1.225)]),
        (06,  "Winch.Pos",                 0.060,     vec![(724, 1.225), (762, 1.225), (819, 1.225), (838, 1.225)]),
        (06,  "Winch.Pos",                 0.080,     vec![(715, 1.225)]),
        (07,  "Winch.Pos",                 0.100,     vec![(829, 1.225)]),
        (08,  "Winch.Pos",                 0.120,     vec![]),
        (08,  "Winch.Pos",                 0.140,     vec![(582, 1.183), (580, 1.183), (723, 1.225), (756, 1.225), (815, 1.225)]),
        (08,  "Winch.Pos",                 0.160,     vec![(761, 1.225), (818, 1.225), (837, 1.225)]),
        (08,  "Winch.Pos",                 0.180,     vec![(714, 1.225)]),
        (09,  "Winch.Pos",                 0.200,     vec![(828, 1.225)]),
        (10,  "Winch.Pos",                 0.220,     vec![]),
        (10,  "Winch.Pos",                 0.240,     vec![(581, 1.183), (579, 1.183), (722, 1.225), (755, 1.225), (814, 1.225)]),
        (10,  "Winch.Pos",                 0.260,     vec![(760, 1.225), (817, 1.225), (836, 1.225)]),
        (10,  "Winch.Pos",                 0.280,     vec![(713, 1.225)]),
        (11,  "Winch.Pos",                 0.300,     vec![(827, 1.225)]),
        (12,  "Winch.Pos",                 0.320,     vec![]),
        (12,  "Winch.Pos",                 0.340,     vec![(580, 1.183), (578, 1.183), (721, 1.225), (754, 1.225), (813, 1.225)]),
        (12,  "Winch.Pos",                 0.360,     vec![(759, 1.225), (816, 1.225), (835, 1.225)]),
        (12,  "Winch.Pos",                 0.380,     vec![(712, 1.225)]),
        (13,  "Winch.Pos",                 0.400,     vec![(826, 1.225)]),
        (14,  "Winch.Pos",                 0.420,     vec![]),
        (14,  "Winch.Pos",                 0.440,     vec![(579, 1.183), (577, 1.183), (720, 1.225), (753, 1.225), (812, 1.225)]),
        (14,  "Winch.Pos",                 0.460,     vec![(758, 1.225), (815, 1.225), (834, 1.225)]),
        (14,  "Winch.Pos",                 0.480,     vec![(711, 1.225)]),
        (15,  "Winch.Pos",                 0.500,     vec![(825, 1.225)]),
        (16,  "Winch.Pos",                 0.520,     vec![]),
        (16,  "Winch.Pos",                 0.540,     vec![(578, 1.183), (576, 1.183), (719, 1.225), (752, 1.225), (811, 1.225)]),
        (16,  "Winch.Pos",                 0.560,     vec![(757, 1.225), (814, 1.225), (833, 1.225)]),
        (16,  "Winch.Pos",                 0.580,     vec![(710, 1.225)]),
        (17,  "Winch.Pos",                 0.600,     vec![(824, 1.225)]),
        (18,  "Winch.Pos",                 0.620,     vec![]),
        (18,  "Winch.Pos",                 0.640,     vec![(577, 1.183), (575, 1.183), (718, 1.225), (751, 1.225), (810, 1.225)]),
        (18,  "Winch.Pos",                 0.660,     vec![(756, 1.225), (813, 1.225), (832, 1.225)]),
        (18,  "Winch.Pos",                 0.680,     vec![(709, 1.225)]),
        (19,  "Winch.Pos",                 0.700,     vec![(823, 1.225)]),
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
    let mut deprecation = Deprecation::new(&dbg, &conf,
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
        for (_, event_name, event_value, _target_deprecation) in test_data {
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
    test_duration.exit();
}
