#![allow(non_snake_case)]
#[cfg(test)]

mod tests {
    use log::{warn, info, debug};
    use std::{sync::Once, time::{Duration, Instant}};
    use crate::{core_::{
        debug::debug_session::{DebugSession, LogLevel, Backtrace}, 
        testing::test_stuff::max_test_duration::TestDuration,
    }, services::profinet_client::s7::{filter::Filter, s7_parse_int::FilterThreshol}}; 
    
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    // use super::*;
    
    static INIT: Once = Once::new();
    
    ///
    /// once called initialisation
    fn initOnce() {
        INIT.call_once(|| {
                // implement your initialisation code to be called only once for current test file
            }
        )
    }
    
    
    ///
    /// returns:
    ///  - ...
    fn initEach() -> () {
    
    }
    
    #[test]
    fn test_FilterThresholdAbs_pos() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        initOnce();
        initEach();
        println!("");
        let selfId = "test FilterThresholdAbs 0.0 - 1.0 - 0.0";
        println!("{}", selfId);
        let testDuration = TestDuration::new(selfId, Duration::from_secs(10));
        testDuration.run().unwrap();
        let testData = [
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
        let mut filter = FilterThreshol::new(0.0, threasold, 0.0);
        let mut prev = 0.0;
        for (value, target) in testData {
            filter.add(value);
            let diff = (prev as f32 - (value as f32)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {}   |   diff: {}", selfId, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        testDuration.exit();
    }

    #[test]
    fn test_FilterThresholdAbs_neg() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        initOnce();
        initEach();
        println!("");
        let selfId = "test FilterThresholdAbs (-1.0) - 1.0 - (-1.0)";
        println!("{}", selfId);
        let testDuration = TestDuration::new(selfId, Duration::from_secs(10));
        testDuration.run().unwrap();
        let testData = [
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
        let mut filter = FilterThreshol::new(0.0, threasold, 0.0);
        let mut prev = 0.0;
        for (value, target) in testData {
            filter.add(value);
            let diff = (prev as f32 - (value as f32)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {}   |   diff: {}", selfId, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        testDuration.exit();
    }
    
    
    #[test]
    fn test_FilterThreshold_factor_pos() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        initOnce();
        initEach();
        println!("");
        let selfId = "test FilterThresholdAbs 0.0 - 1.0 - 0.0 | factor";
        println!("{}", selfId);
        let testDuration = TestDuration::new(selfId, Duration::from_secs(10));
        testDuration.run().unwrap();
        let testData = [
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
        let mut filter = FilterThreshol::new(0.0, threasold, 1.5);
        let mut prev = 0.0;
        for (value, target) in testData {
            filter.add(value);
            let diff = (prev as f32 - (value as f32)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {}   |   diff: {}", selfId, value, result, diff);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        testDuration.exit();
    }

    #[test]
    fn test_FilterThreshold_factor_neg() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        initOnce();
        initEach();
        println!("");
        let selfId = "test FilterThresholdAbs (-1.0) - 1.0 - (-1.0) | factor";
        println!("{}", selfId);
        let testDuration = TestDuration::new(selfId, Duration::from_secs(10));
        testDuration.run().unwrap();
        let testData = [
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
            (0.8, 0.2),
            (0.9, 0.2),
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
            (-0.8, -0.2),
            (-0.9, -0.2),
            (-1.0, -1.0),
        ];
        let threasold = 1.0;
        let mut filter = FilterThreshol::new(0.0, threasold, 1.5);
        let mut prev = 0.0;
        for (value, target) in testData {
            filter.add(value);
            let diff = (prev as f32 - (value as f32)).abs();
            if diff > threasold {
                prev = value;
            }
            let result = filter.value();
            println!("{}    in: {}   |   out: {}   |   diff: {}", selfId, value, result, diff);
            // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        testDuration.exit();
    }        
}
