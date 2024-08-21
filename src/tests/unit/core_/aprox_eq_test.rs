#![allow(non_snake_case)]
#[cfg(test)]

mod tests {
    use log::{debug, info};
    use std::sync::Once;
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::core_::aprox_eq::aprox_eq::AproxEq;
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
    fn test_f32() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test_f32");

        // let (initial, switches) = init_each();
        let test_data = vec![
            (6, (1.234567000f32, 1.234567890f32)),
            (5, (12.34567000f32, 12.34567890f32)),
            (4, (123.4567000f32, 123.4567890f32)),
            (3, (1234.567000f32, 1234.567890f32)),
            (2, (12345.67000f32, 12345.67890f32)),
            (1, (123456.7000f32, 123456.7890f32)),
            (0, (1234567.000f32, 1234567.890f32)),
            (0, (12345678.90f32, 12345678.91f32)),
            (0, (123456789.0f32, 123456789.1f32)),
        ];
        for (decimals, (value, target)) in test_data {
            let aproxEq = value.trunc_eq(target, decimals);
            debug!("value: {:?}   |   target: {:?}  |    decimals: {:?}     |   aproxEq: {:?}", value, target, decimals, aproxEq);
            assert_eq!(aproxEq, true, "value: {:?}   |   target: {:?}  |    decimals: {:?}    |   aproxEq: {:?}", value, target, decimals, aproxEq);
        }
    }

    #[test]
    fn test_f64() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test_f64");

        // let (initial, switches) = init_each();
        let test_data = vec![
            (16, (1.0123456789123456f64, 1.0123456789123456f64)),
            (15, (12.0123456789123451f64, 12.0123456789123456f64)),
            (14, (123.0123456789123411f64, 123.0123456789123456f64)),
            (13, (1234.0123456789123111f64, 1234.0123456789123456f64)),
            (12, (12345.0123456789121111f64, 12345.0123456789123456f64)),
            (11, (123456.0123456789111111f64, 123456.0123456789123456f64)),
            (10, (1234567.0123456789011111f64, 1234567.0123456789123456f64)),
            (9, (12345678.0123456789011111f64, 12345678.0123456789123456f64)),
            (8, (123456789.0123456789111111f64, 123456789.0123456789123456f64)),
            (7, (1234567890.0123456781111111f64, 1234567890.0123456789123456f64)),
            (6, (12345678901.0123456111111111f64, 12345678901.0123456789123456f64)),
            (5, (123456789012.0123451111111111f64, 123456789012.0123456789123456f64)),
            (4, (1234567890123.0123411111111111f64, 1234567890123.0123456789123456f64)),
            (3, (12345678901234.0123111111111111f64, 12345678901234.0123456789123456f64)),
            (2, (123456789012345.0121111111111111f64, 123456789012345.0123456789123456f64)),
            (1, (1234567890123456.0111111111111111f64, 1234567890123456.0123456789123456f64)),
            (0, (12345678901234567.0111111111111111f64, 12345678901234567.0123456789123456f64)),
        ];
        for (decimals, (value, target)) in test_data {
            let aproxEq = value.trunc_eq(target, decimals);
            debug!("value: {:?}   |   target: {:?}  |    decimals: {:?}     |   aproxEq: {:?}", value, target, decimals, aproxEq);
            assert_eq!(aproxEq, true, "value: {:?}   |   target: {:?}  |    decimals: {:?}    |   aproxEq: {:?}", value, target, decimals, aproxEq);
        }
    }
}
