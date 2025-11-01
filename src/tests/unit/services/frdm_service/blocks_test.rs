use std::{fs::OpenOptions, sync::{Arc, atomic::AtomicBool}};
#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::{math::AproxEq, services::conf::ConfTree};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::{services::frdm_service::{Blocks, Booms, CraneConf, FrdmServiceConf, Inputs}, tests::unit::services::frdm_service::CsvRecord};

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
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(20));
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
                        ("MainBoom.Angle", row.a21),
                        ("RotaryBoom.Angle", row.a22)
                    ],
                    [
                        // block.x, block.y
                        (-1829.9999999999993, 11040.0),
                        // (row.x1,    row.y1)
                        (row.x2,    row.y2),
                        (row.x3,    row.y3),
                        (row.x4,    row.y4),
                        (row.x5,    row.y5),
                        (row.x6,    row.y6),
                        (row.x_hook,    row.y_hook),
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
                // block.x,           block.y
                (-1829.9999999999993, 11040.0),
                (2958.9072250002687, 21505.37167318569),
                (3674.151798369415, 23072.283377861302),
                (8047.737692695379, 26376.6496446291),
                (9108.947602987579, 27278.396020445285),
                (9649.303797749028, 26552.998761846295),
                (10057.303797749028, 25552.998761846295),
            ]),
            (02,  [
                // Input Events
                ("MainBoom.Angle",    74.00),
                ("RotaryBoom.Angle", 128.00)
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
        ]
    };
    let conf = ConfTree::new_root(serde_yaml::from_str(r"
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
                lf: -6549.0 mm, 1730.0 mm   # Растояние (x, y) от **конца** стрелы до оси блока, мм
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
                bind: BoomPair 1                # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
            - '7':
                lf: 0.0 mm, 0.0 mm          # Растояние (x, y) от **конца** стрелы до оси блока, мм
                d: 0.0 mm                   # Диаметры блоков, мм
                scheme: TopTop              # Схема схода каната с блоком к следующему: 1 - TopTop, 2 - TopBottom, 3 - BottomTop, 4 - BottomBottom,
                bind: Hook                  # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
        rope:
            width: 35 mm            # Diameter of the rome
            length: 82.243 m          # Total working length of the rope
            winch-length: 58.330 m    # Rope length on the winch drum
            segment: 100 mm         # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
            pos: point real 'Winch.EncoderBR2'      # meters, current rope position
            load: point real 'Winch.Load'          # tonn, current rope load
    ").unwrap());
    let conf = CraneConf::new(&dbg, conf);
    let inputs = Arc::new(Inputs::fake(
        &dbg,
        &FrdmServiceConf::default(),
        [("", 0.0)],
        Arc::new(AtomicBool::new(false)),
    ));
    let mut blocks = Blocks::new(
        &dbg,
        1200.0,        // TODO: replace with config or calculated value
        &conf.blocks,
        Booms::new(&dbg, &conf.booms, inputs.clone()),
    );
    let t = Instant::now();
    for (step, events, target) in test_data {
        for (key, val) in events {
            log::debug!("{dbg} | step {step}  Event '{}': {:?}", key, val);
            inputs.insert(key.to_owned(), val);
        }
        let result = blocks.eval().unwrap();
        log::trace!("{dbg} | step {step}  result: {:#?}", result);
        for (i, (target_x, target_y)) in target.into_iter().enumerate() {
            assert!(result[i].pos.x.aprox_eq(target_x, 1), "{dbg} | step {step}  block[{i}] \n\tresult: {:?}\n\ttarget: {:?}", result[i].pos.x, target_x);
            if i < 6 {
                assert!(result[i].pos.y.aprox_eq(target_y, 1), "{dbg} | step {step}  block[{i}] \n\tresult: {:?}\n\ttarget: {:?}", result[i].pos.y, target_y);
            }
        }
        log::debug!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
    }
    test_duration.exit();
}
