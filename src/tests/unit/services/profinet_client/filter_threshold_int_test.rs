#![allow(non_snake_case)]
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
    fn init_each() -> () {}
    ///
    ///
    #[test]
    fn test_FilterThresholdAbs_pos() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test FilterThresholdAbs 0 - 10 - 0";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (0, 0),
            (1, 0),
            (2, 2),
            (3, 2),
            (4, 4),
            (5, 4),
            (6, 6),
            (7, 6),
            (8, 8),
            (9, 8),
            (10, 10),
            (10, 10),
            (9, 10),
            (8, 8),
            (7, 8),
            (6, 6),
            (5, 6),
            (4, 4),
            (3, 4),
            (2, 2),
            (1, 2),
            (0, 0),
        ];
        let threasold = 1.5;
        let mut filter = FilterThreshold::<i32>::new(None, threasold, 0.0);
        let mut prev = 0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == Some(target), "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing FilterThreshold with absolute threshold
    #[test]
    fn test_FilterThresholdAbs_neg() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test FilterThresholdAbs (-10) - 10 - (-10)";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (-10, -10),
            (-9, -10),
            (-8, -8),
            (-7, -8),
            (-6, -6),
            (-5, -6),
            (-4, -4),
            (-3, -4),
            (-2, -2),
            (-1, -2),
            (0, 0),
            (1, 0),
            (2, 2),
            (3, 2),
            (4, 4),
            (5, 4),
            (6, 6),
            (7, 6),
            (8, 8),
            (9, 8),
            (10, 10),
            (10, 10),
            (9, 10),
            (8, 8),
            (7, 8),
            (6, 6),
            (5, 6),
            (4, 4),
            (3, 4),
            (2, 2),
            (1, 2),
            (0, 0),
            (-1, 0),
            (-2, -2),
            (-3, -2),
            (-4, -4),
            (-5, -4),
            (-6, -6),
            (-7, -6),
            (-8, -8),
            (-9, -8),
            (-10, -10),
        ];
        let threasold = 1.5;
        let mut filter = FilterThreshold::<i32>::new(None, threasold, 0.0);
        let mut prev = 0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == Some(target), "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
}
