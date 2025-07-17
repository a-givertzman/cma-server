#[cfg(test)]
use std::cell::RefCell;
use std::{sync::Once, time::{Duration, Instant}};
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::ConfTree, entity::ToPoint};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::{RopeConf, RopeSlices};

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
    let dbg = Dbg::own("add");
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(&dbg, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        //    pos         load    
        (01,  Some(0.50),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (02,  None      ,  Some(1.0),      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (03,  Some(0.51),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (04,  Some(0.52),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (05,  Some(0.53),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (06,  Some(0.54),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (07,  Some(0.55),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (08,  Some(0.56),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (09,  Some(0.57),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (10,  Some(0.58),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (11,  Some(0.59),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (12,  Some(0.60),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.61),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.62),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.63),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.64),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.65),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.66),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.67),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.68),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.69),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.70),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.71),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
        (13,  Some(0.72),  None     ,      vec![(0, 0.0), (0, 0.0), (0, 0.0)]),
    ];
    let target = test_data.len();
    let conf = ConfTree::new_root(serde_yaml::from_str(r"
        width: 35 mm
        length: 10 m
        segment: 100 mm
        bendings:
            - D300mm 0.500 .. 0.600 m
            - D300mm 0.700 .. 0.800 m
        pos: point real '/App/Winch.EncoderBR2'      # in meters
        load: point real '/App/Winch.Load'             # in tonn
    ").unwrap());
    let conf = RopeConf::new(&dbg, conf);
    let result = RefCell::new(0);
    let mut rope_slices = RopeSlices::new(conf, |ix, deprecation| {
        let result = result.clone();
        let dbg = &dbg.clone();
        log::debug!("{dbg} | Deprication slice[{ix}]: {:?}", deprecation);
        *result.borrow_mut() += 1;
    });
    for (step, pos, load, target) in test_data {
        log::debug!("{dbg} | step {step}  pos: {:?},  load: {:?}", pos, load);
        let pos = pos.map(|val| val.to_point(0, "pos"));
        let load = load.map(|val| val.to_point(0, "load"));
        let time = Instant::now();
        rope_slices.add(pos, load);
        log::debug!("{dbg} | step {step} elapsed: {:?}", time.elapsed());
        // assert!(result == target, "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    assert!(*result.borrow() == target, "{dbg} | \nresult: {:?}\ntarget: {:?}", result.borrow(), target);
    test_duration.exit();
}
