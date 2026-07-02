#[cfg(test)]

use std::{sync::Once, time::Duration};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::domain::filter::{filter::Filter, filter_threshold::FilterThreshold};
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
///
#[test]
fn test_filter_threshold_abs_pos() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let self_id = "test_filter_threshold_abs_pos 0.0 - 1.0 - 0.0";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (0.0, Some(0.0)),
        (0.1, None),
        (0.2, Some(0.2)),
        (0.3, None),
        (0.4, Some(0.4)),
        (0.5, None),
        (0.6, Some(0.6)),
        (0.7, None),
        (0.8, Some(0.8)),
        (0.9, None),
        (1.0, Some(1.0)),
        (1.0, None),
        (0.9, None),
        (0.8, Some(0.8)),
        (0.7, None),
        (0.6, Some(0.6)),
        (0.5, None),
        (0.4, Some(0.4)),
        (0.3, None),
        (0.2, Some(0.2)),
        (0.1, None),
        (0.0, Some(0.0)),
    ];
    let threasold = 0.15;
    let mut filter = FilterThreshold::<f32>::new(None, threasold, 0.0);
    let mut prev = 0.0;
    for (value, target) in test_data {
        let result = filter.add(value);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    let mut filter = FilterThreshold::<f64>::new(None, threasold, 0.0);
    let mut prev = 0.0;
    for (value, target) in test_data.into_iter().map(|(value, target)| (value as f64, target.map(|t| t as f64))) {
        let result = filter.add(value);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    test_duration.exit();
}
///
/// Testing FilterThreshold with absolute threshold and negative input
#[test]
fn test_filter_threshold_abs_neg_f64() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let self_id = "test_filter_threshold_abs_neg (-1.0) - 1.0 - (-1.0)";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (-1.0, Some(-1.0)),
        (-0.9, None),
        (-0.8, Some(-0.8)),
        (-0.7, None),
        (-0.6, Some(-0.6)),
        (-0.5, None),
        (-0.4, Some(-0.4)),
        (-0.3, None),
        (-0.2, Some(-0.2)),
        (-0.1, None),
        (0.0, Some(0.0)),
        (0.1, None),
        (0.2, Some(0.2)),
        (0.3, None),
        (0.4, Some(0.4)),
        (0.5, None),
        (0.6, Some(0.6)),
        (0.7, None),
        (0.8, Some(0.8)),
        (0.9, None),
        (1.0, Some(1.0)),
        (1.0, None),
        (0.9, None),
        (0.8, Some(0.8)),
        (0.7, None),
        (0.6, Some(0.6)),
        (0.5, None),
        (0.4, Some(0.4)),
        (0.3, None),
        (0.2, Some(0.2)),
        (0.1, None),
        (0.0, Some(0.0)),
        (-0.1, None),
        (-0.2, Some(-0.2)),
        (-0.3, None),
        (-0.4, Some(-0.4)),
        (-0.5, None),
        (-0.6, Some(-0.6)),
        (-0.7, None),
        (-0.8, Some(-0.8)),
        (-0.9, None),
        (-1.0, Some(-1.0)),
    ];
    let threasold = 0.15;
    let mut filter = FilterThreshold::<f32>::new(None, threasold, 0.0);
    let mut prev = 0.0;
    for (value, target) in test_data {
        let result = filter.add(value);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    let mut filter = FilterThreshold::<f64>::new(None, threasold, 0.0);
    let mut prev = 0.0;
    for (value, target) in test_data.into_iter().map(|(value, target)| (value as f64, target.map(|t| t as f64))) {
        let result = filter.add(value);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    test_duration.exit();
}
///
/// Testing FilterThreshold with factor and pisitive input
#[test]
fn test_filter_threshold_factor_pos() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let self_id = "test_filter_threshold_factor_pos 0.0 - 1.0 - 0.0 | factor";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (0.0, Some(0.0)),
        (0.1, None),
        (0.2, None),
        (0.3, None),
        (0.4, Some(0.4)),
        (0.5, None),
        (0.6, None),
        (0.7, None),
        (0.8, Some(0.8)),
        (0.9, None),
        (1.0, None),
        (1.0, None),
        (0.9, None),
        (0.8, None),
        (0.7, None),
        (0.6, None),
        (0.5, None),
        (0.4, None),
        (0.3, Some(0.3)),
        (0.2, None),
        (0.1, None),
        (0.0, None),
    ];
    let threasold = 1.0;
    let mut filter = FilterThreshold::<f32>::new(None, threasold, 1.5);
    let mut prev = 0.0;
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        let result = filter.add(value);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    let mut filter = FilterThreshold::<f64>::new(None, threasold, 1.5);
    let mut prev = 0.0;
    for (step, (value, target)) in test_data.into_iter().map(|(value, target)| (value as f64, target.map(|t| t as f64))).enumerate() {
        let result = filter.add(value);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    test_duration.exit();
}
///
/// Testing FilterThreshold with factor and negative input
#[test]
fn test_filter_threshold_factor_neg() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let dbg = "test_filter_threshold_factor_neg (-1.0) - 1.0 - (-1.0) | factor";
    println!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (-1.0, Some(-1.0)),
        (-0.9, None),
        (-0.8, None),
        (-0.7, None),
        (-0.6, Some(-0.6)),
        (-0.5, None),
        (-0.4, None),
        (-0.3, None),
        (-0.2, Some(-0.2)),
        (-0.1, None),
        (0.0, None),
        (0.1, None),
        (0.2, Some(0.2)),
        (0.3, None),
        (0.4, None),
        (0.5, None),
        (0.6, Some(0.6)),
        (0.7, None),
        (0.8, None),
        (0.9, None),
        (1.0, Some(1.0)),
        (1.0, None),
        (0.9, None),
        (0.8, None),
        (0.7, None),
        (0.6, Some(0.6)),
        (0.5, None),
        (0.4, None),
        (0.3, None),
        (0.2, Some(0.2)),
        (0.1, None),
        (0.0, None),
        (-0.1, None),
        (-0.2, Some(-0.2)),
        (-0.3, None),
        (-0.4, None),
        (-0.5, None),
        (-0.6, Some(-0.6)),
        (-0.7, None),
        (-0.8, None),
        (-0.9, None),
        (-1.0, Some(-1.0)),
    ];
    let threasold = 1.0;
    let mut filter = FilterThreshold::<f32>::new(None, threasold, 1.5);
    let mut prev = 0.0;
    let mut last = None;
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        let result = filter.add(value);
        last = result.or_else(|| last);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        println!("{}    in: {}   |   out: {:?}   |   diff: {}", dbg, value, result, diff);
        assert!(result == target, "{dbg} | step {step}  |  \nresult: {:?}\ntarget: {:?}", result, target);
        let result = filter.last();
        assert!(result == last, "{dbg} | step {step}  |  \nresult: {:?}\ntarget: {:?}", result, last);
    }
    let mut filter = FilterThreshold::<f64>::new(None, threasold, 1.5);
    let mut prev = 0.0;
    let mut last = None;
    for (step, (value, target)) in test_data.into_iter().map(|(value, target)| (value as f64, target.map(|t| t as f64))).enumerate() {
        let result = filter.add(value);
        last = result.or_else(|| last);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        println!("{} |  in: {}   |   out: {:?}   |   diff: {}", dbg, value, result, diff);
        assert!(result == target, "{dbg} | step {step}  |  \nresult: {:?}\ntarget: {:?}", result, last);
        let result = filter.last();
        assert!(result == last, "{dbg} | step {step}  |  \nresult: {:?}\ntarget: {:?}", result, last);
    }
    test_duration.exit();
}
///
/// Manual test for exact cases
#[test]
fn manual_test() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "manual_test";
    log::debug!("{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (-1.0, Some(-1.0)),
        (-0.9, None),
        (-0.8, None),
        (-0.7, None),
        (-0.6, Some(-0.6)),
        (-0.5, None),
        (-0.4, None),
        (-0.3, None),
        (-0.2, Some(-0.2)),
        (-0.1, None),
        (0.0, None),
        (0.1, None),
        (0.2, Some(0.2)),
        (0.3, None),
        (0.4, None),
        (0.5, None),
        (0.6, Some(0.6)),
        (0.7, None),
        (0.8, None),
        (0.9, None),
        (1.0, Some(1.0)),
        (1.0, None),
        (10.0, None),
        (12.0, None),
        (14.0, None),
        (16.0, None),
        (18.0, None),
        (20.0, None),
        (22.0, None),
        (24.0, None),
        (25.0, None),
        (26.0, None),
        (0.9, None),
        (0.8, None),
        (0.7, None),
        (0.6, Some(0.6)),
        (0.5, None),
        (0.4, None),
        (0.3, None),
        (0.2, Some(0.2)),
        (0.1, None),
        (0.0, None),
        (-0.1, None),
        (-0.2, Some(-0.2)),
        (-0.3, None),
        (-0.4, None),
        (-0.5, None),
        (-0.6, Some(-0.6)),
        (-0.7, None),
        (-0.8, None),
        (-0.9, None),
        (-1.0, Some(-1.0)),
    ];
    let threasold = 25.0;
    let factor = 1.0;
    log::debug!("{dbg} | threasold: {threasold}");
    log::debug!("{dbg} | factor: {factor}");
    let mut filter = FilterThreshold::<i64>::new(None, threasold, factor);
    let mut prev = 0.0;
    for (value, target) in test_data {
        let result = filter.add(value as i64);
        let diff = (prev as f64 - (value as f64)).abs();
        if diff > threasold {
            prev = value;
        }
        log::debug!("{dbg} | in: {:0.000}   |\tout: {:?}   |   diff: {}", value, result, diff);
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    test_duration.exit();
}
