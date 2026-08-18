use std::{fs::OpenOptions, sync::{Arc, atomic::AtomicBool}};
#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfTree;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{services::{BlockArcs, Blocks, Booms, CraneConf, FrdmServiceConf, Inputs, RopeSections}, tests::unit::services::frdm_service::CsvRecord};

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
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("BlockArcs-test");
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
                        // wrap_alpha, deg     wrap_length, mm
                        (row.wrap_alpha1,      row.wrap_l1),
                        (row.wrap_alpha2,      row.wrap_l2),
                        (row.wrap_alpha3,      row.wrap_l3),
                        (row.wrap_alpha4,      row.wrap_l4),
                        (row.wrap_alpha5,      row.wrap_l5),
                        (row.wrap_alpha6,      row.wrap_l6),
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
                // wrap_alpha, deg     wrap_length, mm
                (  0.000,                0.000),
                (  0.127,                0.903),
                ( 28.393,              202.232),
                ( 32.597,              232.176),
                ( 6.674,                47.540),
                (101.150,              720.452),
            ]),
            (02,  [
                // Input Events
                ("MainBoom.Angle",    74.00),
                ("RotaryBoom.Angle", 128.00)
            ],
            // Targets
            [
                // wrap_alpha, deg     wrap_length, mm
                ( 0.000,                 0.000),
                (26.619,               189.599),
                (28.930,               206.057),
                (32.597,               232.176),
                ( 6.674,                47.540),
                (78.140,               556.560),
            ]),
        ],
    };
    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        rope:
            width: 35 mm                    # Diameter of the rome
            length: 85.045 m                # Total working length of the rope
            aux-length: 1.200 m             # Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
            segment: 100 mm                 # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
            pos: point real 'Winch.Pos'     # meters, current rope position (длина каната размотанного с барабана считая от парковочного)
            load: point real 'Winch.Load'   # tonn, current rope load
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
    let inputs = Arc::new(Inputs::fake(
        &dbg,
        &FrdmServiceConf::default(),
        [("", 0.0)],
        Arc::new(AtomicBool::new(false)),
    ));
    let parking = true;
    let mut block_arcs = BlockArcs::new(
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
    );
    for (step, events, target) in test_data {
        let t = Instant::now();
        for (key, val) in events {
            log::debug!("{dbg} | step {step}  Event '{}': {:?}", key, val);
            inputs.insert(key.to_owned(), val);
        }
        let result = block_arcs.eval().unwrap();
        log::debug!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
        log::trace!("{dbg} | step {step}  result: {:#?}", result);
        log::trace!("{dbg} | step {step}  result rope alpha: \n\t{:?}", result.iter().map(|b| format!("{:.3}", b.rope_alpha_fwd)).collect::<Vec<_>>());
        log::trace!("{dbg} | step {step}  result rope alpha: \n\t{:?}", result.iter().map(|b| format!("{:.3}", b.rope_alpha_bck)).collect::<Vec<_>>());
        log::trace!("{dbg} | step {step}  result wrap alpha: \n\t{:?}", result.iter().map(|b| format!("{:.3}", b.rope_alpha_fwd - b.rope_alpha_bck)).collect::<Vec<_>>());
        log::debug!("{dbg} | step {step}  result wrap alpha: \n\t{:?}", result.iter().map(|b| format!("{:.3}", b.wrap_alpha)).collect::<Vec<_>>());
        log::debug!("{dbg} | step {step}  target wrap alpha: \n\t{:?}", target.iter().map(|(a, _)| format!("{:.3}", a)).collect::<Vec<_>>());
        for (i, (wrap_alpha, wrap_length)) in target.into_iter().enumerate() {
            if !result[i].skipped {
                assert!((result[i].wrap_alpha - wrap_alpha).abs() < 1.0, "{dbg} | step {step}  block {i} \nresult: {:?}\ntarget: {:?}", result[i].wrap_alpha, wrap_alpha);
            }
            if !result[i].skipped {
                assert!((result[i].wrap_length - wrap_length).abs() < 1.0, "{dbg} | step {step}  block {i} \nresult: {:?}\ntarget: {:?}", result[i].wrap_length, wrap_length);
            }
        }
    }
    test_duration.exit();
}
