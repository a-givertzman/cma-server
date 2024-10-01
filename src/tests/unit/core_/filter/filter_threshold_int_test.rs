#[cfg(test)]

mod tests {
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::core_::filter::{filter_threshold::FilterThreshold, filter::Filter};
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
            (1, None),
            (2, Some(2)),
            (3, None),
            (4, Some(4)),
            (5, None),
            (6, Some(6)),
            (7, None),
            (8, Some(8)),
            (9, None),
            (10, Some(10)),
            (10, None),
            (9, None),
            (8, Some(8)),
            (7, None),
            (6, Some(6)),
            (5, None),
            (4, Some(4)),
            (3, None),
            (2, Some(2)),
            (1, None),
            (0, Some(0)),
        ]
    }
    ///
    ///
    #[test]
    fn test_filter_threshold_abs_pos_i16() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test_filter_threshold_abs_pos 0 - 10 - 0";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = init_each();
        let threasold = 1.5;
        let mut filter = FilterThreshold::<2, i16>::new(None, threasold, 0.0);
        let mut prev = 0;
        for (step, (value, target)) in test_data.into_iter().map(|(value, target)| (*value, *target)).enumerate() {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
            assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
    ///
    ///
    #[test]
    fn test_filter_threshold_abs_pos_i32() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test_filter_threshold_abs_pos_i32 0 - 10 - 0";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = init_each();
        let threasold = 1.5;
        let mut filter = FilterThreshold::<2, i32>::new(None, threasold, 0.0);
        let mut prev = 0;
        for (step, (value, target)) in test_data.into_iter().map(|(value, target)| (*value as i32, target.map(|t| t as i32))).enumerate() {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
            assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
    ///
    ///
    #[test]
    fn test_filter_threshold_abs_pos_i64() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test_filter_threshold_abs_pos_i64 0 - 10 - 0";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = init_each();
        let threasold = 1.5;
        let mut filter = FilterThreshold::<2, i64>::new(None, threasold, 0.0);
        let mut prev = 0;
        for (step, (value, target)) in test_data.into_iter().map(|(value, target)| (*value as i64, target.map(|t| t as i64))).enumerate() {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
            assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing FilterThreshold with absolute threshold
    #[test]
    fn test_filter_threshold_abs_neg_i16() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test_filter_threshold_abs_neg_i16 (-10) - 10 - (-10)";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (-10, Some(-10)),
            (-9, None),
            (-8, Some(-8)),
            (-7, None),
            (-6, Some(-6)),
            (-5, None),
            (-4, Some(-4)),
            (-3, None),
            (-2, Some(-2)),
            (-1, None),
            (0, Some(0)),
            (1, None),
            (2, Some(2)),
            (3, None),
            (4, Some(4)),
            (5, None),
            (6, Some(6)),
            (7, None),
            (8, Some(8)),
            (9, None),
            (10, Some(10)),
            (10, None),
            (9, None),
            (8, Some(8)),
            (7, None),
            (6, Some(6)),
            (5, None),
            (4, Some(4)),
            (3, None),
            (2, Some(2)),
            (1, None),
            (0, Some(0)),
            (-1, None),
            (-2, Some(-2)),
            (-3, None),
            (-4, Some(-4)),
            (-5, None),
            (-6, Some(-6)),
            (-7, None),
            (-8, Some(-8)),
            (-9, None),
            (-10, Some(-10)),
        ];
        let threasold = 1.5;
        let mut filter = FilterThreshold::<2, i16>::new(None, threasold, 0.0);
        let mut prev = 0;
        for (step, (value, target)) in test_data.into_iter().enumerate() {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
            assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing FilterThreshold with absolute threshold
    #[test]
    fn test_filter_threshold_abs_neg_i32() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test_filter_threshold_abs_neg_i32 (-10) - 10 - (-10)";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (-10, Some(-10)),
            (-9, None),
            (-8, Some(-8)),
            (-7, None),
            (-6, Some(-6)),
            (-5, None),
            (-4, Some(-4)),
            (-3, None),
            (-2, Some(-2)),
            (-1, None),
            (0, Some(0)),
            (1, None),
            (2, Some(2)),
            (3, None),
            (4, Some(4)),
            (5, None),
            (6, Some(6)),
            (7, None),
            (8, Some(8)),
            (9, None),
            (10, Some(10)),
            (10, None),
            (9, None),
            (8, Some(8)),
            (7, None),
            (6, Some(6)),
            (5, None),
            (4, Some(4)),
            (3, None),
            (2, Some(2)),
            (1, None),
            (0, Some(0)),
            (-1, None),
            (-2, Some(-2)),
            (-3, None),
            (-4, Some(-4)),
            (-5, None),
            (-6, Some(-6)),
            (-7, None),
            (-8, Some(-8)),
            (-9, None),
            (-10, Some(-10)),
        ];
        let threasold = 1.5;
        let mut filter = FilterThreshold::<2, i32>::new(None, threasold, 0.0);
        let mut prev = 0;
        for (step, (value, target)) in test_data.into_iter().enumerate() {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
            assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing FilterThreshold with absolute threshold
    #[test]
    fn test_filter_threshold_abs_neg_i64() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test_filter_threshold_abs_neg_i64 (-10) - 10 - (-10)";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (-10, Some(-10)),
            (-9, None),
            (-8, Some(-8)),
            (-7, None),
            (-6, Some(-6)),
            (-5, None),
            (-4, Some(-4)),
            (-3, None),
            (-2, Some(-2)),
            (-1, None),
            (0, Some(0)),
            (1, None),
            (2, Some(2)),
            (3, None),
            (4, Some(4)),
            (5, None),
            (6, Some(6)),
            (7, None),
            (8, Some(8)),
            (9, None),
            (10, Some(10)),
            (10, None),
            (9, None),
            (8, Some(8)),
            (7, None),
            (6, Some(6)),
            (5, None),
            (4, Some(4)),
            (3, None),
            (2, Some(2)),
            (1, None),
            (0, Some(0)),
            (-1, None),
            (-2, Some(-2)),
            (-3, None),
            (-4, Some(-4)),
            (-5, None),
            (-6, Some(-6)),
            (-7, None),
            (-8, Some(-8)),
            (-9, None),
            (-10, Some(-10)),
        ];
        let threasold = 1.5;
        let mut filter = FilterThreshold::<2, i64>::new(None, threasold, 0.0);
        let mut prev = 0;
        for (step, (value, target)) in test_data.into_iter().enumerate() {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    step: {}  in: {}   |   out: {:?}   |   diff: {}", self_id, step, value, result, diff);
            assert!(result == target, "step: {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        test_duration.exit();
    }
}
