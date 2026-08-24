use std::{fs::OpenOptions, sync::{Arc, atomic::AtomicBool}};
#[cfg(test)]
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfTree;
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{services::{Block, Blocks, Booms, CraneConf, FrdmServiceConf, Inputs, RopeSections}, tests::unit::services::frdm_service::CsvRecord};

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
/// Testing [RopeSection]
#[test]
fn new() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = Dbg::own("RopeSection-test");
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
                        ("MainBoom.Angle", row.a21),
                        ("RotaryBoom.Angle", row.a22)
                    ],
                    [
                        // rope_len_bck,     rope_len_fwd,              rope_alpha_bck,     rope_alpha_fwd
                        (0.0000,                row.lrope_straight1,               0.00,    row.rope_alpha1),
                        (row.lrope_straight1,   row.lrope_straight2,    row.rope_alpha1,    row.rope_alpha2),
                        (row.lrope_straight2,   row.lrope_straight3,    row.rope_alpha2,    row.rope_alpha3),
                        (row.lrope_straight3,   row.lrope_straight4,    row.rope_alpha3,    row.rope_alpha4),
                        (row.lrope_straight4,   row.lrope_straight5,    row.rope_alpha4,    row.rope_alpha5),
                        (row.lrope_straight5,   row.lrope_straight6,    row.rope_alpha5,    row.rope_alpha6),
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
                    width: 35 mm            # Diameter of the rome
                    length: 85.045 m        # Total working length of the rope
                    aux-length: 1.200 m     # Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
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
                        bind: Drum                 # Привязка блока к стреле (нумерация с 0), Fixed - Барабан, Boom 0 - Блок на первой стреле, Hook - Блок на подвесе
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
                    ("MainBoom.Angle",    69.71),
                    ("RotaryBoom.Angle", 155.30)
                ],
                // Targets
                [
                    // rope_len_bck,     rope_len_fwd,  rope_alpha_bck,     rope_alpha_fwd
                    (    0.00,           11509.01,      0.00,               0.00),
                    (11509.01,            1722.44,      0.00,               0.00),
                    ( 1722.44,            5481.52,      0.00,               0.00),
                    ( 5481.52,            1128.33,      0.00,               0.00),
                    ( 1128.33,             389.89,      0.00,               0.00),
                    (  389.89,            1200.00,      0.00,               0.00),
                ]),
                (02,  [
                    // Input Events
                    ("MainBoom.Angle",    74.00),
                    ("RotaryBoom.Angle", 128.00)
                ],
                // Targets
                [
                    // rope_len_bck,     rope_len_fwd,  rope_alpha_bck,     rope_alpha_fwd
                    (    0.00,           11362.11,      0.00,               0.00),
                    (11362.11,            2261.27,      0.00,               0.00),
                    ( 2261.27,            5481.52,      0.00,               0.00),
                    ( 5481.52,            1128.33,      0.00,               0.00),
                    ( 1128.33,             389.89,      0.00,               0.00),
                    (  389.89,            1200.00,      0.00,               0.00),
                ]),
            ],
        )]
    };
    for (path, conf, test_data) in test_data {
        log::info!("{dbg} | Test Blicks for '{path}'");
        let inputs = Arc::new(Inputs::fake(
            &dbg,
            &FrdmServiceConf::default(),
            [("", 0.0)],
            Arc::new(AtomicBool::new(false)),
        ));
        let parking = true;
        let mut rope_sections = RopeSections::new(
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
        );
        for (step, events, target) in test_data {
            let t = Instant::now();
            for (key, val) in events {
                log::debug!("{dbg} | step {step}  Event '{}': {:?}", key, val);
                inputs.insert(key.to_owned(), val);
            }
            let result: Vec<Block> = rope_sections.eval().unwrap()
                    .into_iter()
                    .filter(|block| !block.skipped)
                    .collect();
            log::debug!("{dbg} | step {step}  Elapsed: {:?}", t.elapsed());
            log::trace!("{dbg} | step {step}  result: {:#?}", result);
            log::debug!("{dbg} | step {step}  result rope alpha bck: \n\t{:?}", result.iter().map(|b| format!("{:.3}", b.rope_alpha_bck)).collect::<Vec<_>>());
            log::debug!("{dbg} | step {step}  target rope alpha: \n\t{:?}", target.iter().map(|(_, _, a, _)| format!("{:.3}", a)).collect::<Vec<_>>());
            log::debug!("{dbg} | step {step}  result rope alpha fwd: \n\t{:?}", result.iter().map(|b| format!("{:.3}", b.rope_alpha_fwd)).collect::<Vec<_>>());
            log::debug!("{dbg} | step {step}  target rope alpha: \n\t{:?}", target.iter().map(|(_, _, _, a)| format!("{:.3}", a)).collect::<Vec<_>>());
            log::debug!("{dbg} | step {step}  result rope len: \n\t{:?}", result.iter().map(|b| format!("{:.3}", b.rope_len_fwd)).collect::<Vec<_>>());
            log::debug!("{dbg} | step {step}  target rope len: \n\t{:?}", target.iter().map(|(_, l, _, _)| format!("{:.3}", l)).collect::<Vec<_>>());
            for (i, (rope_len_bck, rope_len_fwd, rope_alpha_bck, rope_alpha_fwd)) in target.into_iter().enumerate() {
                if i < result.len() - 2 {
                    if i > 0 { // Лебедку пропускаем, в ней лежит wrap_delta - разница обусловленная изменением угла схода с лебедки относительно парковочного
                        assert!((result[i].rope_len_bck - rope_len_bck).abs() < 1.0, "{dbg} | step {step}  block[{i}] \n\tresult: {:?}\n\ttarget: {:?}", result[i].rope_len_bck, rope_len_bck);
                    }
                    assert!((result[i].rope_len_fwd - rope_len_fwd).abs() < 1.0, "{dbg} | step {step}  block[{i}] \n\tresult: {:?}\n\ttarget: {:?}", result[i].rope_len_fwd, rope_len_fwd);
                }
                assert!((result[i].rope_alpha_bck - rope_alpha_bck).abs() < 1.0, "{dbg} | step {step}  block[{i}] \n\tresult: {:?}\n\ttarget: {:?}", result[i].rope_alpha_bck, rope_alpha_bck);
                assert!((result[i].rope_alpha_fwd - rope_alpha_fwd).abs() < 1.0, "{dbg} | step {step}  block[{i}] \n\tresult: {:?}\n\ttarget: {:?}", result[i].rope_alpha_fwd, rope_alpha_fwd);
            }
        }
    }
    test_duration.exit();
}
