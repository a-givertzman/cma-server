#[cfg(test)]

mod unit_circle {
    use std::{f64::consts::PI, sync::Once, time::Duration};
    use rustfft::num_complex::Complex;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

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
    /// Testing UnutCycle   0,  45,  90, 135, 180, 225, 270, 315, 360 grad
    #[test]
    fn next_8() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        // Sampling freq
        let freq = 8;
        // Step of angle on the unit circle
        let delta = 2.0 * PI / (freq as f64);
        // Vec<(angle, complex)>
        let test_data = (0..=freq).map(|step| {
            let angle = delta * (step as f64);
            let complex = Complex::new(angle.cos(), angle.sin());
            (angle, complex)
        });
        let targets = [
            (  0,          0.0,  1.0f64,               0.0f64           ),
            ( 45,       PI/4.0,  2.0f64.sqrt()/2.0,    2.0f64.sqrt()/2.0),
            ( 90,       PI/2.0,  0.0f64,               1.0f64),
            (135,   3.0*PI/4.0, -2.0f64.sqrt()/2.0,    2.0f64.sqrt()/2.0),
            (180,          PI,  -1.0f64,               0.0f64           ),
            (225,   5.0*PI/4.0, -2.0f64.sqrt()/2.0,   -2.0f64.sqrt()/2.0),
            (270,   3.0*PI/2.0,  0.0f64,              -1.0f64),
            (315,   7.0*PI/4.0,  2.0f64.sqrt()/2.0,   -2.0f64.sqrt()/2.0),
            (360,       2.0*PI,  1.0f64,               0.0f64           ),
        ];
        let mut targets_iter = targets.into_iter();
        for (angle, complex) in test_data {
            let (target_angle_grad, target_angle, target_re, target_im) = targets_iter.next().unwrap();
            log::debug!("target angle: {} ({}), complex: ({}, {}i)   |   result angle: {} ({}), complex: {}", target_angle, target_angle_grad, target_re, target_im, angle, 180.0 * angle / PI, complex);
            let result = complex.re * complex.re + complex.im * complex.im;
            let target = 1.0;
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
            let result = angle;
            let target = target_angle;
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
            let result = complex.re;
            let target = target_re;
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
            let result = complex.im;
            let target = target_im;
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing UnutCycle   0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330, 360 grad
    #[test]
    fn next_12() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        // Sampling freq
        let freq = 12;
        // Step of angle on the unit circle
        let delta = 2.0 * PI / (freq as f64);
        // Vec<(angle, complex)>
        let test_data = (0..=freq).map(|step| {
            let angle = delta * (step as f64);
            let complex = Complex::new(angle.cos(), angle.sin());
            (angle, complex)
        });
        let targets = [
            (  0,          0.0,  1.0f64,               0.0f64           ),
            ( 30,       PI/6.0,  3.0f64.sqrt()/2.0,    1.0f64/2.0),
            ( 60,       PI/3.0,  1.0f64/2.0,           3.0f64.sqrt()/2.0),
            ( 90,       PI/2.0,  0.0f64,               1.0f64),
            (120,   2.0*PI/3.0, -1.0f64/2.0,           3.0f64.sqrt()/2.0),
            (150,   5.0*PI/6.0, -3.0f64.sqrt()/2.0,    1.0f64/2.0),
            (180,          PI,  -1.0f64,               0.0f64           ),
            (210,   7.0*PI/6.0, -3.0f64.sqrt()/2.0,   -1.0f64/2.0),
            (240,   4.0*PI/3.0, -1.0f64/2.0,          -3.0f64.sqrt()/2.0),
            (270,   3.0*PI/2.0,  0.0f64,              -1.0f64),
            (300,   5.0*PI/3.0,  1.0f64/2.0,          -3.0f64.sqrt()/2.0),
            (330,  11.0*PI/6.0,  3.0f64.sqrt()/2.0,   -1.0f64/2.0),
            (360,       2.0*PI,  1.0f64,               0.0f64           ),
        ];
        let mut targets_iter = targets.into_iter();
        for (angle, complex) in test_data {
            let (target_angle_grad, target_angle, target_re, target_im) = targets_iter.next().unwrap();
            log::debug!("target angle: {} ({}), complex: ({}, {}i)   |   result angle: {} ({}), complex: {}", target_angle, target_angle_grad, target_re, target_im, angle, 180.0 * angle / PI, complex);
            let result = complex.re * complex.re + complex.im * complex.im;
            let target = 1.0;
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
            let result = angle;
            let target = target_angle;
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
            let result = complex.re;
            let target = target_re;
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
            let result = complex.im;
            let target = target_im;
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
        }        test_duration.exit();
    }
    ///
    /// Testing UnutCycle   0,  45,  90, 135, 180, 225, 270, 315, 360 grad
    #[test]
    fn next() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        // Sampling freq
        let freq = 500_000;
        // Step of angle on the unit circle
        let delta = 2.0 * PI / (freq as f64);
        // Vec<(angle, complex)>
        let test_data = (0..freq).map(|step| {
            let angle = delta * (step as f64);
            let complex = Complex::new(angle.cos(), angle.sin());
            (angle, complex)
        });
        for (angle, complex) in test_data {
            log::trace!("angle: {}  |  complex: {}", angle, complex);
            let result = complex.re * complex.re + complex.im * complex.im;
            let target = 1.0;
            assert!(result.aprox_eq(target, 8), "angle {} \nresult: {:?}\ntarget: {:?}", angle, result, target);
            let result = complex.re;
            let target = angle.cos();
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, 180.0 * angle / PI, result, target);
            let result = complex.im;
            let target = angle.sin();
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, 180.0 * angle / PI, result, target);
        }
        test_duration.exit();
    }
}
