#[cfg(test)]

mod tests {
    use core::f64;
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::{DebugSession, LogLevel};
    use crate::domain::filter::filter::{Filter, FilterEmpty};
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
    fn test_filter_empty_f64() {
        DebugSession::new().filter(LogLevel::Debug).init().unwrap();
        init_once();
        init_each();
        let dbg = "test_filter_empty_f64 0.0 - 1.0 - 0.0";
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (-0.2, Some(-0.2)),
            (-0.2, None),
            (-0.1, Some(-0.1)),
            (0.0, Some(0.0)),
            (0.1, Some(0.1)),
            (0.2, Some(0.2)),
            (0.3, Some(0.3)),
            (0.4, Some(0.4)),
            (0.5, Some(0.5)),
            (0.6, Some(0.6)),
            (0.7, Some(0.7)),
            (0.8, Some(0.8)),
            (0.9, Some(0.9)),
            (1.0, Some(1.0)),
            (1.0, None),
            (0.9, Some(0.9)),
            (0.8, Some(0.8)),
            (0.7, Some(0.7)),
            (0.6, Some(0.6)),
            (0.5, Some(0.5)),
            (0.4, Some(0.4)),
            (0.3, Some(0.3)),
            (0.2, Some(0.2)),
            (0.1, Some(0.1)),
            (0.0, Some(0.0)),
            (0.0, None),
            (-0.1, Some(-0.1)),
            (-0.1, None),
            (-0.2, Some(-0.2)),
            (f64::NAN, None),
        ];
        let threasold = f64::EPSILON;
        let mut filter = FilterEmpty::<f64>::new(None);
        let mut prev = 0.0;
        for (value, target) in test_data {
            let result = filter.add(value);
            let diff = (prev - value).abs();
            if diff > threasold {
                prev = value;
            }
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", dbg, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        let mut filter = FilterEmpty::<f64>::new(None);
        let mut prev = 0.0;
        for (value, target) in test_data.into_iter().map(|(value, target)| (value, target.map(|t| t))) {
            let result = filter.add(value);
            let diff = (prev as f64 - (value as f64)).abs();
            if diff > threasold {
                prev = value;
            }
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", dbg, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing FilterEmpty with absolute empty and negative input
    #[test]
    fn test_filter_empty_f32() {
        DebugSession::new().filter(LogLevel::Info).init().unwrap();
        init_once();
        init_each();
        let self_id = "test_filter_empty_f32 (-1.0) - 1.0 - (-1.0)";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (-1.0, Some(-1.0)),
            (-1.0, None),
            (-0.8, Some(-0.8)),
            (-0.6, Some(-0.6)),
            (-0.4, Some(-0.4)),
            (-0.2, Some(-0.2)),
            (-0.2, None),
            (0.0, Some(0.0)),
            (0.0, None),
            (0.1, Some(0.1)),
            (0.2, Some(0.2)),
            (0.8, Some(0.8)),
            (0.8, None),
            (1.0, Some(1.0)),
            (1.0, None),
            (0.9, Some(0.9)),
            (0.8, Some(0.8)),
            (0.6, Some(0.6)),
            (0.6, None),
            (0.4, Some(0.4)),
            (0.2, Some(0.2)),
            (0.2, None),
            (0.0, Some(0.0)),
            (0.0, None),
            (-0.1, Some(-0.1)),
            (-0.2, Some(-0.2)),
            (-0.4, Some(-0.4)),
            (-0.4, None),
            (-0.6, Some(-0.6)),
            (-0.8, Some(-0.8)),
            (-0.8, None),
            (-1.0, Some(-1.0)),
            (f32::NAN, None),
        ];
        let threasold = f32::EPSILON;
        let mut filter = FilterEmpty::<f32>::new(None);
        let mut prev = 0.0;
        for (value, target) in test_data {
            let result = filter.add(value);
            let diff = (prev - value).abs();
            if diff > threasold {
                prev = value;
            }
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        let mut filter = FilterEmpty::<f32>::new(None);
        let mut prev = 0.0;
        for (value, target) in test_data.into_iter().map(|(value, target)| (value, target.map(|t| t))) {
            let result = filter.add(value);
            let diff = (prev - value).abs();
            if diff > threasold {
                prev = value;
            }
            println!("{}    in: {}   |   out: {:?}   |   diff: {}", self_id, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
}
