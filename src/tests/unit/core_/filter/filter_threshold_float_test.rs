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
    fn test_filter_threshold_abs_pos() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
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
        let mut filter = FilterThreshold::<2, f32>::new(None, threasold, 0.0);
        let mut prev = 0.0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        let mut filter = FilterThreshold::<2, f64>::new(None, threasold, 0.0);
        let mut prev = 0.0;
        for (value, target) in test_data.into_iter().map(|(value, target)| (value as f64, target.map(|t| t as f64))) {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing FilterThreshold with absolute threshold and negative input
    #[test]
    fn test_filter_threshold_abs_neg_f64() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
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
        let mut filter = FilterThreshold::<2, f32>::new(None, threasold, 0.0);
        let mut prev = 0.0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        let mut filter = FilterThreshold::<2, f64>::new(None, threasold, 0.0);
        let mut prev = 0.0;
        for (value, target) in test_data.into_iter().map(|(value, target)| (value as f64, target.map(|t| t as f64))) {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing FilterThreshold with factor and pisitive input
    #[test]
    fn test_filter_threshold_factor_pos() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
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
        let mut filter = FilterThreshold::<2, f32>::new(None, threasold, 1.5);
        let mut prev = 0.0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        let mut filter = FilterThreshold::<2, f64>::new(None, threasold, 1.5);
        let mut prev = 0.0;
        for (value, target) in test_data.into_iter().map(|(value, target)| (value as f64, target.map(|t| t as f64))) {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing FilterThreshold with factor and negative input
    #[test]
    fn test_filter_threshold_factor_neg() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test_filter_threshold_factor_neg (-1.0) - 1.0 - (-1.0) | factor";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
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
        let mut filter = FilterThreshold::<2, f32>::new(None, threasold, 1.5);
        let mut prev = 0.0;
        for (value, target) in test_data {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        let mut filter = FilterThreshold::<2, f64>::new(None, threasold, 1.5);
        let mut prev = 0.0;
        for (value, target) in test_data.into_iter().map(|(value, target)| (value as f64, target.map(|t| t as f64))) {
            filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.pop();
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
}
