use std::{fs::OpenOptions, sync::{Arc, atomic::AtomicBool}};
#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfTree;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{services::{Booms, CraneConf, FrdmServiceConf, Inputs, Offset}, tests::unit::services::frdm_service::CsvRecord};

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
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("Booms-test");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    let path = "src/tests/unit/services/frdm_service/deprecation_test.csv";
    log::debug!("{dbg} | reading csv: '{}'", path);
    let csv = match OpenOptions::new().read(true).open(path) {
        Ok(rdr) => {
            let mut rdr = csv::Reader::from_reader(rdr);
            log::debug!("{dbg} | Parse csv data...");
            let csv: csv::DeserializeRecordsIter<'_, _, CsvRecord> = rdr.deserialize();
            let mut test_data = vec![];
            for row in csv {
                let row: CsvRecord = row.unwrap();
                test_data.push((
                    row.step,
                    [
                        ("MainBoom.Angle",   row.a21),
                        ("RotaryBoom.Angle", row.a22)
                    ],
                    [
                        // boom[i].G
                        (0.00,   0.00),
                        (row.xg, row.yg),
                    ],
                ));
            }
            Some(test_data)
        }
        Err(err) => {
            log::debug!("{dbg} | Can't read csv test data from '{}', error: {:?}", path, err);
            None
        },
    };
    let test_data = match csv {
        Some(csv) => csv,
        None => vec![
            (01,  [
                // Input Events
                ("MainBoom.Angle",    69.71),
                ("RotaryBoom.Angle", 155.30)
            ],
            // Targets
            [
                // boom[i].G
                (3883.8458824556724, 20835.034086633517),
                (9528.40100476262, 26481.55987434036),
            ]),
            (02,  [
                // Input Events
                ("MainBoom.Angle",    74.00),
                ("RotaryBoom.Angle", 128.00)
            ],
            // Targets
            [
                // boom.G
                (3087.138385150391, 21096.13099450917),
                (10489.774280011621, 24086.990036341813),
            ]),
        ],
    };
    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        rope:
            width: 35 mm            # Diameter of the rome
            length: 3000 m          # Total working length of the rope
            aux-length: 1.200 m     # Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
            segment: 100 mm         # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
            pos: point real 'Winch.EncoderBR2'      # meters, current rope position
            load: point real 'Winch.Load'          # tonn, current rope load
        booms:
            - Main-Boom:
                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                l4: 10330.0 mm              # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                len: 11200.0 mm                                         # length of the boom
                angle: point real 'MainBoom.Angle'   # degrees, current angle of the boom (relative axis)
                parking: 0.0 deg            # Угол в парковочном положении, град (обязателен для главной стрелы, для остальных может быть опущен)
            - Rotary-Boom:
                l1: 0.0 mm                  # Растояние от продольной оси стрелы до точки A (оси ее поворота), константа
                l2: 0.0 mm                  # Растояние по продольной оси стрелы от точки D (корня стрелы) до точки A (оси ее поворота), константа
                l3: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до продольной оси предыдущей стрелы (до ГСК для первой срелы), константа
                l4: 0.0 mm                  # Расстояние от точки A (ось поворота) стрелы до перпендикуляра к продольной оси через точку G предыдущей стрелы (до ГСК для первой срелы), константа
                len: 7984.0 mm                                          # length of the rotary boom
                angle: point real 'RotaryBoom.Angle' # degrees, current angle of the boom (relative axis)
                parking: 155.299999999996 deg  # Угол в парковочном положении, град (обязателен для главной стрелы, для остальных может быть опущен)
        blocks:
            - '1':
                lf: 1830.0 mm,  710.0 mm    # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 844.0 mm                 # Диаметры блоков, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Drum                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
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
                bind: Boom 1            # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - '6':
                lf: 136.0 mm, -35.0 mm      # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 816.0 mm                 # Диаметры блоков, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Boom 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
                deflector-angle: 90 deg     # Угол перекидывания, град. Блок включается в работу только когда стрела проходит положение перекидывания.
            - '7':
                lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 0.0 mm                   # Диаметры блоков, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
    ").unwrap());
    let conf = CraneConf::new(&dbg, conf);
    let inputs = Arc::new(Inputs::fake(
        &dbg,
        &FrdmServiceConf::default(),
        [("", 0.0)],
        Arc::new(AtomicBool::new(false)),
    ));
    let mut booms = Booms::new(&dbg, &conf.booms, inputs.clone(), true);
    let _parking = booms.eval().unwrap();
    for (step, events, target_pos) in test_data {
        let t = Instant::now();
        for (key, val) in events {
            log::trace!("{dbg} | step {step}  Event '{}': {:?}", key, val);
            inputs.insert(key.to_owned(), val);
        }
        let result = booms.eval().unwrap();
        log::trace!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
        log::trace!("{dbg} | step {step}  result: {:#?}", result);
        // for (i, target) in target_alpha.into_iter().enumerate() {
        //     assert!(result[i].alpha.aprox_eq(target, 3), "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", result[i].alpha, target);
        // }
        for (i, (target_gx, target_gy)) in target_pos.into_iter().enumerate() {
            if i == target_pos.len() - 1 {
                let (Offset{x: dx, y: dy}, Offset{x: gx, y: gy}) = (result[i].dpt, result[i].gpt);
                assert!((gx - target_gx).abs() < 0.1, "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", gx, target_gx);
                assert!((gy - target_gy).abs() < 0.1, "{dbg} | step {step}  \nresult: {:?}\ntarget: {:?}", gy, target_gy);
            }
        }
    }
    test_duration.exit();
}
