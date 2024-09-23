#[cfg(test)]

mod unit_circle {
    use std::{f64::consts::PI, sync::Once, time::Duration};
    use rustfft::num_complex::Complex;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

    use crate::{core_::aprox_eq::aprox_eq::AproxEq, services::task::nested_function::va::unit_circle::UnitCircle};
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
    /// Testing (angle, complex) for:  0,  45,  90, 135, 180, 225, 270, 315, 360 grad
    #[test]
    fn anderstanding() {
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
        let test_data = [
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
        let mut values = (0..=freq).map(|step| {
            let angle = delta * (step as f64);
            let complex = Complex::new(angle.cos(), angle.sin());
            (angle, complex)
        });
        for (target_angle_grad, target_angle, target_re, target_im) in test_data {
            let (angle, complex) = values.next().unwrap();
            log::debug!("target angle: {:.4} ({}), complex: ({:.4}, {:.4}i)   |   result angle: {:.4} ({}), complex: {:.4}", target_angle, target_angle_grad, target_re, target_im, angle, 180.0 * angle / PI, complex);
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
    /// Testing UnutCycle::at_angle for   0,  45,  90, 135, 180, 225, 270, 315, 360 grad
    #[test]
    fn at_angle_8() {
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
        let test_data = [
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
        let unit_circle = UnitCircle::new(freq);
        for (target_angle_grad, target_angle, target_re, target_im) in test_data {
            let (time, complex) = unit_circle.at_angle(target_angle);
            // let k = (target_angle / (2.0 * PI)).trunc();
            // let result = angle + 2.0 * PI * k;
            // let target = target_angle;
            // assert!(result.aprox_eq(target, 8), "time, sec: {}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, angle, target_angle_grad, result, target);
            let result = complex.re;
            let target = target_re;
            assert!(result.aprox_eq(target, 8), "time, sec: {:.8}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, target_angle, target_angle_grad, result, target);
            let result = complex.im;
            let target = target_im;
            assert!(result.aprox_eq(target, 8), "time, sec: {:.8}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, target_angle, target_angle_grad, result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing UnutCycle::at_angle for   0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330, 360 grad
    #[test]
    fn at_angle_12() {
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
        let test_data = [
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
        let unit_circle = UnitCircle::new(freq);
        for (target_angle_grad, target_angle, target_re, target_im) in test_data {
            let (time, complex) = unit_circle.at_angle(target_angle);
            // let k = (target_angle / (2.0 * PI)).trunc();
            // let result = angle + 2.0 * PI * k;
            // let target = target_angle;
            // assert!(result.aprox_eq(target, 8), "time, sec: {}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, angle, target_angle_grad, result, target);
            let result = complex.re;
            let target = target_re;
            assert!(result.aprox_eq(target, 8), "time, sec: {:.8}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, target_angle, target_angle_grad, result, target);
            let result = complex.im;
            let target = target_im;
            assert!(result.aprox_eq(target, 8), "time, sec: {:.8}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, target_angle, target_angle_grad, result, target);
        }
        test_duration.exit();
    }
    ///
    /// Testing UnutCycle::at_angle deppending on specified sampling frequency
    #[test]
    fn at_angle() {
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
        let test_data = (0..freq * 3).map(|step| {
            let angle = delta * (step as f64);
            let complex = Complex::new(angle.cos(), angle.sin());
            (angle, complex)
        });
        let unit_circle = UnitCircle::new(freq);
        for (target_angle, target_complex) in test_data {
            log::trace!("angle: {:.4}  |  complex: {:.4}", target_angle, target_complex);
            let result = target_complex.re * target_complex.re + target_complex.im * target_complex.im;
            let target = 1.0;
            assert!(result.aprox_eq(target, 8), "angle {} \nresult: {:?}\ntarget: {:?}", target_angle, result, target);
            let result = target_complex.re;
            let target = target_angle.cos();
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", target_angle, 180.0 * target_angle / PI, result, target);
            let result = target_complex.im;
            let target = target_angle.sin();
            assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", target_angle, 180.0 * target_angle / PI, result, target);

            let (time, complex) = unit_circle.at_angle(target_angle);
            // let k = (target_angle / (2.0 * PI)).trunc();
            // let result = angle + 2.0 * PI * k;
            // let target = target_angle;
            // assert!(result.aprox_eq(target, 8), "time, sec: {}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, target_angle, 180.0 * target_angle / PI, result, target);
            let result = complex.re;
            let target = target_complex.re;
            assert!(result.aprox_eq(target, 8), "time, sec: {:.8}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, target_angle, 180.0 * target_angle / PI, result, target);
            let result = complex.im;
            let target = target_complex.im;
            assert!(result.aprox_eq(target, 8), "time, sec: {:.8}  angle {} ({}) \nresult: {:?}\ntarget: {:?}", time, target_angle, 180.0 * target_angle / PI, result, target);
        }
    }
    ///
    /// Testing UnutCycle::angle for   0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330, 360 grad
    #[test]
    fn angle_12() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        // Sampling freq
        let freq = 12.0;
        let test_data = [
            ( 0.0/(freq * freq),     0,          0.0,  1.0f64,               0.0f64           ),
            ( 1.0/(freq * freq),    30,       PI/6.0,  3.0f64.sqrt()/2.0,    1.0f64/2.0),
            ( 2.0/(freq * freq),    60,       PI/3.0,  1.0f64/2.0,           3.0f64.sqrt()/2.0),
            ( 3.0/(freq * freq),    90,       PI/2.0,  0.0f64,               1.0f64),
            ( 4.0/(freq * freq),   120,   2.0*PI/3.0, -1.0f64/2.0,           3.0f64.sqrt()/2.0),
            ( 5.0/(freq * freq),   150,   5.0*PI/6.0, -3.0f64.sqrt()/2.0,    1.0f64/2.0),
            ( 6.0/(freq * freq),   180,          PI,  -1.0f64,               0.0f64           ),
            ( 7.0/(freq * freq),   210,   7.0*PI/6.0, -3.0f64.sqrt()/2.0,   -1.0f64/2.0),
            ( 8.0/(freq * freq),   240,   4.0*PI/3.0, -1.0f64/2.0,          -3.0f64.sqrt()/2.0),
            ( 9.0/(freq * freq),   270,   3.0*PI/2.0,  0.0f64,              -1.0f64),
            (10.0/(freq * freq),   300,   5.0*PI/3.0,  1.0f64/2.0,          -3.0f64.sqrt()/2.0),
            (11.0/(freq * freq),   330,  11.0*PI/6.0,  3.0f64.sqrt()/2.0,   -1.0f64/2.0),
            (12.0/(freq * freq),   360,       2.0*PI,  1.0f64,               0.0f64           ),
        ];
        let unit_circle = UnitCircle::new(freq as usize);
        for (target_t, target_angle_grad, target_angle, _target_re, _target_im) in test_data {
            let t = target_angle / (2.0 * PI * freq);
            log::debug!("angle: {:.4} ({:.4}),  target time, sec: {:.4}  t {:.4} ", target_angle, target_angle_grad, target_t, t);
            // assert!(result.aprox_eq(target, 8), "time, sec: {:.4}  angle {:.4} ({:.4}) \nresult: {:?}\ntarget: {:?}", target_t, angle, target_angle_grad, result, target);
            // let angle = unit_circle.angle(target_t);
            // // let k = (target_angle / (2.0 * PI)).trunc();
            // let result = angle;// + 2.0 * PI * k;
            // let target = target_angle;
            // assert!(result.aprox_eq(target, 8), "time, sec: {:.4}  angle {:.4} ({:.4}) \nresult: {:?}\ntarget: {:?}", target_t, angle, target_angle_grad, result, target);
        }
        test_duration.exit();
    }

}
