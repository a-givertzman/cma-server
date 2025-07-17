#[cfg(test)]
use std::cell::RefCell;
use std::{rc::Rc, sync::Once, time::{Duration, Instant}};
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
        //                                       rope slices deprecetion
        //    pos         load                   slice[0]  slice[1]  slice[2]  count of dep's
        (01,  Some(0.50),  None     ,      vec![ 0.00,     0.00,     0.00],     0),
        (02,  None      ,  Some(1.0),      vec![ 3.33,     0.00,     3.33],     2),
        (03,  Some(0.51),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (04,  Some(0.52),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (05,  Some(0.53),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (06,  Some(0.54),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (07,  Some(0.55),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (08,  Some(0.56),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (09,  Some(0.57),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (10,  Some(0.58),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (11,  Some(0.59),  None     ,      vec![ 3.33,     0.00,     3.33],     2),
        (12,  Some(0.60),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (13,  Some(0.61),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (14,  Some(0.62),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (15,  Some(0.63),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (16,  Some(0.64),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (17,  Some(0.65),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (18,  Some(0.66),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (19,  Some(0.67),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (20,  Some(0.68),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (21,  Some(0.69),  None     ,      vec![ 6.66,     3.33,     6.66],     5),
        (22,  Some(0.70),  None     ,      vec![ 9.99,     3.33,     6.66],     6),
        (23,  Some(0.71),  None     ,      vec![ 9.99,     6.66,     6.66],     7),
        (24,  Some(0.72),  None     ,      vec![ 9.99,     6.66,     6.66],     7),
    ];
    let mut target: Vec<f64> = vec![];
    let mut target_count = 0;
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
    let result = Rc::new(RefCell::new(vec![0.00, 0.00, 0.00]));
    let result_count = Rc::new(RefCell::new(0));
    let mut rope_slices = RopeSlices::new(conf, |ix, deprecation| {
        let dbg = &dbg.clone();
        log::debug!("{dbg} | Deprication slice[{ix}]: {:?}", deprecation);
        result.replace_with(|r| {
            r[ix] += deprecation;
            r.to_owned()
        });
        result_count.replace_with(|r| {
            *r + 1
        });
    });
    for (step, pos, load, target_i, target_count_i) in test_data {
        target = target_i;
        target_count = target_count_i;
        log::debug!("{dbg} | step {step}  pos: {:?},  load: {:?}", pos, load);
        let pos = pos.map(|val| val.to_point(0, "pos"));
        let load = load.map(|val| val.to_point(0, "load"));
        let time = Instant::now();
        rope_slices.eval(pos, load);
        log::debug!("{dbg} | step {step} elapsed: {:?}", time.elapsed());
        assert!(
            result.borrow().iter().enumerate().all(|(ix, r)| {
                log::trace!("{dbg} | step {step} result: {}, target: {},  test: {}", r.round(), target[ix].round(), r.round() == target[ix].round());
                r.round() == target[ix].round()
            }),
            "{dbg} | step {} \nresult: {:?}\ntarget: {:?}", step, result.borrow().to_vec(), target,
        );
    }
    assert!(*result_count.borrow() == target_count, "{dbg} | \nresult: {:?}\ntarget: {:?}", result_count.borrow(), target_count);
    let result = result.borrow().to_vec();
    assert!(
        result.iter().enumerate().all(|(ix, r)| {
            log::debug!("{dbg} | result: {}, target: {},  test: {}", r.round(), target[ix].round(), r.round() == target[ix].round());
            r.round() == target[ix].round()
        }),
        "{dbg} | \nresult: {:?}\ntarget: {:?}", result, target,
    );
    test_duration.exit();
}
