#[cfg(test)]

use std::{sync::Once, time::Duration};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::domain::filter::{filter::{Filter, FilterEmpty}};
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
fn init_each() -> &'static [(i16, Option<i16>)] {
    &[
        (0, Some(0)),
        (1, Some(1)),
        (2, Some(2)),
        (3, Some(3)),
        (4, Some(4)),
        (4, None),
        (5, Some(5)),
        (6, Some(6)),
        (7, Some(7)),
        (8, Some(8)),
        (9, Some(9)),
        (10, Some(10)),
        (10, None),
        (9, Some(9)),
        (8, Some(8)),
        (7, Some(7)),
        (6, Some(6)),
        (5, Some(5)),
        (4, Some(4)),
        (3, Some(3)),
        (2, Some(2)),
        (1, Some(1)),
        (0, Some(0)),
        (-1, Some(-1)),
        (i16::MIN, Some(i16::MIN)),
        (i16::MAX, Some(i16::MAX)),
    ]
}
///
///
#[test]
fn test_filter_empty_abs_pos_i16() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let self_id = "test_filter_empty_abs_pos 0 - 10 - 0";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = init_each();
    let mut filter = FilterEmpty::<i16>::new(None);
    for (step, (value, target)) in test_data.into_iter().map(|(value, target)| (*value, *target)).enumerate() {
        let result = filter.add(value);
        // println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
        assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
///
///
#[test]
fn test_filter_empty_abs_pos_i32() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let self_id = "test_filter_empty_abs_pos_i32 0 - 10 - 0";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = init_each();
    let mut filter = FilterEmpty::<i32>::new(None);
    for (step, (value, target)) in test_data.into_iter().map(|(value, target)| (*value as i32, target.map(|t| t as i32))).enumerate() {
        let result = filter.add(value);
        // println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
        assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
///
///
#[test]
fn test_filter_empty_abs_pos_i64() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let self_id = "test_filter_empty_abs_pos_i64 0 - 10 - 0";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = init_each();
    let mut filter = FilterEmpty::<i64>::new(None);
    for (step, (value, target)) in test_data.into_iter().map(|(value, target)| (*value as i64, target.map(|t| t as i64))).enumerate() {
        let result = filter.add(value);
        // println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
        assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
///
/// Testing FilterEmpty with absolute empty
#[test]
fn test_filter_empty_abs_neg_i16() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let self_id = "test_filter_empty_abs_neg_i16 (-10) - 10 - (-10)";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (-10, Some(-10)),
        (-8, Some(-8)),
        (-8, None),
        (-6, Some(-6)),
        (-6, None),
        (-4, Some(-4)),
        (-2, Some(-2)),
        (-1, Some(-1)),
        (0, Some(0)),
        (0, None),
        (1, Some(1)),
        (2, Some(2)),
        (4, Some(4)),
        (6, Some(6)),
        (8, Some(8)),
        (10, Some(10)),
        (10, None),
        (8, Some(8)),
        (8, None),
        (6, Some(6)),
        (4, Some(4)),
        (2, Some(2)),
        (0, Some(0)),
        (-1, Some(-1)),
        (-2, Some(-2)),
        (-4, Some(-4)),
        (-6, Some(-6)),
        (-8, Some(-8)),
        (-8, None),
        (-10, Some(-10)),
    ];
    let mut filter = FilterEmpty::<i16>::new(None);
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        let result = filter.add(value);
        // println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
        assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
///
/// Testing FilterEmpty with absolute empty
#[test]
fn test_filter_empty_abs_neg_i32() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let self_id = "test_filter_empty_abs_neg_i32 (-10) - 10 - (-10)";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (-10, Some(-10)),
        (-10, None),
        (-2, Some(-2)),
        (-1, Some(-1)),
        (0, Some(0)),
        (0, None),
        (1, Some(1)),
        (2, Some(2)),
        (10, Some(10)),
        (10, None),
        (8, Some(8)),
        (1, Some(1)),
        (0, Some(0)),
        (-1, Some(-1)),
        (-2, Some(-2)),
        (-2, None),
        (-10, Some(-10)),
    ];
    let mut filter = FilterEmpty::<i32>::new(None);
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        let result = filter.add(value);
        // println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
        assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
///
/// Testing FilterEmpty with absolute empty
#[test]
fn test_filter_empty_abs_neg_i64() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let self_id = "test_filter_empty_abs_neg_i64 (-10) - 10 - (-10)";
    println!("\n{}", self_id);
    let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
    test_duration.run().unwrap();
    let test_data = [
        (-10, Some(-10)),
        (-10, None),
        (-8, Some(-8)),
        (-2, Some(-2)),
        (-1, Some(-1)),
        (0, Some(0)),
        (0, None),
        (1, Some(1)),
        (2, Some(2)),
        (2, None),
        (10, Some(10)),
        (10, None),
        (8, Some(8)),
        (8, None),
        (2, Some(2)),
        (1, Some(1)),
        (1, None),
        (0, Some(0)),
        (-1, Some(-1)),
        (-2, Some(-2)),
        (-10, Some(-10)),
        (-10, None),
    ];
    let mut filter = FilterEmpty::<i64>::new(None);
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        let result = filter.add(value);
        // println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
        assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    test_duration.exit();
}
