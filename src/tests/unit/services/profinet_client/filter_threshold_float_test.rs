#![allow(non_snake_case)]
#[cfg(test)]

mod tests {
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::core_::filter::{filter::Filter, filter_threshold::FilterThreshold};
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
        let self_id = "test FilterThresholdAbs 0.0 - 1.0 - 0.0";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (0.0, 0.0),
            (0.1, 0.0),
            (0.2, 0.2),
            (0.3, 0.2),
            (0.4, 0.4),
            (0.5, 0.4),
            (0.6, 0.6),
            (0.7, 0.6),
            (0.8, 0.8),
            (0.9, 0.8),
            (1.0, 1.0),
            (1.0, 1.0),
            (0.9, 1.0),
            (0.8, 0.8),
            (0.7, 0.8),
            (0.6, 0.6),
            (0.5, 0.6),
            (0.4, 0.4),
            (0.3, 0.4),
            (0.2, 0.2),
            (0.1, 0.2),
            (0.0, 0.0),
        ];
        let threasold = 0.15;
        let mut filter = FilterThreshold::new(0.0, threasold, 0.0);
        let mut prev = 0.0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }

    #[test]
    fn test_FilterThresholdAbs_neg() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test FilterThresholdAbs (-1.0) - 1.0 - (-1.0)";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (-1.0, -1.0),
            (-0.9, -1.0),
            (-0.8, -0.8),
            (-0.7, -0.8),
            (-0.6, -0.6),
            (-0.5, -0.6),
            (-0.4, -0.4),
            (-0.3, -0.4),
            (-0.2, -0.2),
            (-0.1, -0.2),
            (0.0, 0.0),
            (0.1, 0.0),
            (0.2, 0.2),
            (0.3, 0.2),
            (0.4, 0.4),
            (0.5, 0.4),
            (0.6, 0.6),
            (0.7, 0.6),
            (0.8, 0.8),
            (0.9, 0.8),
            (1.0, 1.0),
            (1.0, 1.0),
            (0.9, 1.0),
            (0.8, 0.8),
            (0.7, 0.8),
            (0.6, 0.6),
            (0.5, 0.6),
            (0.4, 0.4),
            (0.3, 0.4),
            (0.2, 0.2),
            (0.1, 0.2),
            (0.0, 0.0),
            (-0.1, 0.0),
            (-0.2, -0.2),
            (-0.3, -0.2),
            (-0.4, -0.4),
            (-0.5, -0.4),
            (-0.6, -0.6),
            (-0.7, -0.6),
            (-0.8, -0.8),
            (-0.9, -0.8),
            (-1.0, -1.0),
        ];
        let threasold = 0.15;
        let mut filter = FilterThreshold::new(0.0, threasold, 0.0);
        let mut prev = 0.0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }


    #[test]
    fn test_FilterThreshold_factor_pos() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test FilterThresholdAbs 0.0 - 1.0 - 0.0 | factor";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (0.0, 0.0),
            (0.1, 0.0),
            (0.2, 0.0),
            (0.3, 0.0),
            (0.4, 0.4),
            (0.5, 0.4),
            (0.6, 0.4),
            (0.7, 0.4),
            (0.8, 0.8),
            (0.9, 0.8),
            (1.0, 0.8),
            (1.0, 0.8),
            (0.9, 0.8),
            (0.8, 0.8),
            (0.7, 0.8),
            (0.6, 0.8),
            (0.5, 0.8),
            (0.4, 0.8),
            (0.3, 0.3),
            (0.2, 0.3),
            (0.1, 0.3),
            (0.0, 0.3),
        ];
        let threasold = 1.0;
        let mut filter = FilterThreshold::new(0.0, threasold, 1.5);
        let mut prev = 0.0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }

    #[test]
    fn test_FilterThreshold_factor_neg() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test FilterThresholdAbs (-1.0) - 1.0 - (-1.0) | factor";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (-1.0, -1.0),
            (-0.9, -1.0),
            (-0.8, -1.0),
            (-0.7, -1.0),
            (-0.6, -0.6),
            (-0.5, -0.6),
            (-0.4, -0.6),
            (-0.3, -0.6),
            (-0.2, -0.2),
            (-0.1, -0.2),
            (0.0, -0.2),
            (0.1, -0.2),
            (0.2, 0.2),
            (0.3, 0.2),
            (0.4, 0.2),
            (0.5, 0.2),
            (0.6, 0.6),
            (0.7, 0.6),
            (0.8, 0.6),
            (0.9, 0.6),
            (1.0, 1.0),
            (1.0, 1.0),
            (0.9, 1.0),
            (0.8, 1.0),
            (0.7, 1.0),
            (0.6, 0.6),
            (0.5, 0.6),
            (0.4, 0.6),
            (0.3, 0.6),
            (0.2, 0.2),
            (0.1, 0.2),
            (0.0, 0.2),
            (-0.1, 0.2),
            (-0.2, -0.2),
            (-0.3, -0.2),
            (-0.4, -0.2),
            (-0.5, -0.2),
            (-0.6, -0.6),
            (-0.7, -0.6),
            (-0.8, -0.6),
            (-0.9, -0.6),
            (-1.0, -1.0),
        ];
        let threasold = 1.0;
        let mut filter = FilterThreshold::new(0.0, threasold, 1.5);
        let mut prev = 0.0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
}
