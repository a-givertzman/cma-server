#[cfg(test)]
use std::{fs::OpenOptions, sync::{Arc, atomic::AtomicBool}};
use std::{sync::Once, time::Duration};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfTree;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{services::{Bendings, BlockArcs, Blocks, Booms, CraneConf, FrdmServiceConf, Inputs, RopeDeprecationConf, RopeSections}, tests::unit::services::frdm_service::CsvRecord};

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
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Bendings-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(20));
    test_duration.run().unwrap();
    let path = [
        // ("src/tests/unit/services/frdm_service/ysz-deprecation_test.yaml",
        // "src/tests/unit/services/frdm_service/ysz-deprecation_test.csv"),
        ("src/tests/unit/services/frdm_service/spu-tnpa-deprecation_test.yaml",
        "src/tests/unit/services/frdm_service/spu-tnpa-deprecation_test.csv"),
    ];
    let test_data = if !path.is_empty() {
        let mut res = Vec::with_capacity(path.len());
        for (conf, csv_path) in path {
            log::debug!("{dbg} | reading csv: '{csv_path}'");
            let rdr = OpenOptions::new().read(true).open(csv_path).unwrap();
            let mut rdr = csv::Reader::from_reader(rdr);
            log::debug!("{dbg} | Parse csv data...");
            let csv: csv::DeserializeRecordsIter<'_, _, CsvRecord> = rdr.deserialize();
            let mut test_data = vec![];
            for row in csv {
                let row: CsvRecord = row.unwrap();
                test_data.push((
                    row.step,
                    [
                        ("Winch.Pos",           0.00),  // rope position, m
                        // ("Winch.Pos",        row.pos / 1000.0),  // rope position, m
                        ("MainBoom.Angle",   row.a21),
                        ("RotaryBoom.Angle", row.a22)
                    ],
                    [
                        // enter        ..      exit, mm
                            0.00         ..      row.t01,    // exit from winch
                        row.t02         ..      row.t03,
                        row.t04         ..      row.t05,
                        row.t06         ..      row.t07,
                        row.t08         ..      row.t09,
                        row.t10         ..      row.t11,
                        // row.t12         ..      f64::NAN,
                    ],
                ));
            }
            log::debug!("{dbg} | reading conf: '{conf}'");
            let rdr = OpenOptions::new().read(true).open(conf).unwrap();
            let conf = ConfTree::new_root(serde_yaml::from_reader(rdr).unwrap());
            let conf = CraneConf::new(&dbg, conf);
            res.push((csv_path, conf, test_data));
        }
        res
    } else {
        log::debug!("{dbg} | CSV datasets are not set, continue test with embeded dataset");
        vec![(
            "embedded conf and dataset",
            CraneConf::new(&dbg, ConfTree::new_root(serde_yaml::from_str(r"
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
            ").unwrap())),
            vec![
                (01,  [
                    // Input Events
                    ("Winch.Pos",          0.00),
                    ("MainBoom.Angle",    69.71),
                    ("RotaryBoom.Angle", 155.30),
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
            ]
        )]
    };
    for (path, conf, test_data) in test_data {
        log::info!("{dbg} | Test Booms for '{path}'");
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
        let mut bendings = Bendings::new(
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
                        Booms::new(
                            &dbg,
                            &conf.booms,
                            inputs.clone(),
                            parking,
                        ),
                    ),
                ),
            ),
        );
        let tolerance = 0.9;
        let mut errors = vec![];
        for (step, events, target) in test_data.iter() {
            let t = std::time::Instant::now();
            let mut events_log = String::new();
            for (key, val) in events {
                events_log.push_str(&format!("{}: {:.3}; ", key, val));
                inputs.insert(key.to_owned(), *val);
            }
            log::debug!("{dbg} | step {step}  Events: {:?}", events_log);
            let result = bendings.eval(inputs.rope_pos()).unwrap();
            log::debug!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
            // log::trace!("{dbg} | step {step}  result: {:#?}", result);
            log::debug!("{dbg} | step {step}  result bending: \n\t{:?}", result.iter().map(|b| format!("{:.3}..{:.3}", b.bending.start, b.bending.end)).collect::<Vec<_>>());
            log::debug!("{dbg} | step {step}  target bending: \n\t{:?}", target.iter().map(|b| format!("{:.3}..{:.3}", b.start, b.end)).collect::<Vec<_>>());
            let mut ok = true;
            for (i, bending) in target.iter().enumerate() {
                if (bending.end - bending.start).abs() > 0.00001 {
                    if i > 0 {
                        assert!((result[i].bending.start - bending.start).abs() < tolerance, "{dbg} | step {step} Bending {i}  \nresult: {:?}\ntarget: {:?}", result[i].bending.start, bending.start);
                    }
                    let start = bending.start.min(bending.end);
                    let end = bending.end.max(bending.start);
                    assert!((result[i].bending.end - end).abs() < tolerance, "{dbg} | step {step} Bending {i}  \nresult: {:?}\ntarget: {:?}", result[i].bending.end, end);
                    ok = ok && (((result[i].bending.start - start).abs() < tolerance) && ((result[i].bending.end - end).abs() < tolerance));
                }
            }
            errors.push(format!("step {step}  result: {:?}", result.iter().map(|b| format!("{:.3}..{:.3}", b.bending.start, b.bending.end)).collect::<Vec<_>>()));
            errors.push(format!("step {step}  target: {:?}", target.iter().map(|b| format!("{:.3}..{:.3}", b.start, b.end)).collect::<Vec<_>>()));
            if result.iter().zip(target).enumerate().any(|(i, (r, t))| (i > 0 && (t.start - r.bending.start).abs() >= tolerance) || (t.end - r.bending.end).abs() >= tolerance) {
                errors.push(format!("step {step}  delta : {:?}", result.iter().zip(target).map(|(r, t)| format!("{:.3}..{:.3}", t.start - r.bending.start, t.end - r.bending.end)).collect::<Vec<_>>()));
            }
            if !ok {
            }
        }
        for line in errors {
            println!("{line}");
        }
    }
    test_duration.exit();
}
