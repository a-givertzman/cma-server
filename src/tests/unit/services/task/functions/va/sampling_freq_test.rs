#[cfg(test)]

mod sampling_freq {
    use std::{f64::consts::PI, sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{core_::aprox_eq::aprox_eq::AproxEq, services::task::nested_function::va::sampling_freq::SamplingFreq};
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
    /// Testing SamplingFreq.next()
    #[test]
    fn next() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        // Sampling freq
        let test_data = [
            (
                8, vec![
                    (0.0/8.0,      0,          0.0, ( 1.0f64,               0.0f64           )),
                    (1.0/8.0,     45,       PI/4.0, ( 2.0f64.sqrt()/2.0,    2.0f64.sqrt()/2.0)),
                    (2.0/8.0,     90,       PI/2.0, ( 0.0f64,               1.0f64           )),
                    (3.0/8.0,    135,   3.0*PI/4.0, (-2.0f64.sqrt()/2.0,    2.0f64.sqrt()/2.0)),
                    (4.0/8.0,    180,          PI,  (-1.0f64,               0.0f64           )),
                    (5.0/8.0,    225,   5.0*PI/4.0, (-2.0f64.sqrt()/2.0,   -2.0f64.sqrt()/2.0)),
                    (6.0/8.0,    270,   3.0*PI/2.0, ( 0.0f64,              -1.0f64           )),
                    (7.0/8.0,    315,   7.0*PI/4.0, ( 2.0f64.sqrt()/2.0,   -2.0f64.sqrt()/2.0)),
                    (8.0/8.0,    360,       2.0*PI, ( 1.0f64,               0.0f64           )),
                ],
            ),
            (
                12, vec![
                    ( 0.0/12.0,      0,          0.0, ( 1.0f64,               0.0f64           )),
                    ( 1.0/12.0,     30,       PI/6.0, ( 3.0f64.sqrt()/2.0,    1.0f64/2.0       )),
                    ( 2.0/12.0,     60,       PI/3.0, ( 1.0f64/2.0,           3.0f64.sqrt()/2.0)),
                    ( 3.0/12.0,     90,       PI/2.0, ( 0.0f64,               1.0f64           )),
                    ( 4.0/12.0,    120,   2.0*PI/3.0, (-1.0f64/2.0,           3.0f64.sqrt()/2.0)),
                    ( 5.0/12.0,    150,   5.0*PI/6.0, (-3.0f64.sqrt()/2.0,    1.0f64/2.0       )),
                    ( 6.0/12.0,    180,          PI,  (-1.0f64,               0.0f64           )),
                    ( 7.0/12.0,    210,   7.0*PI/6.0, (-3.0f64.sqrt()/2.0,   -1.0f64/2.0       )),
                    ( 8.0/12.0,    240,   4.0*PI/3.0, (-1.0f64/2.0,          -3.0f64.sqrt()/2.0)),
                    ( 9.0/12.0,    270,   3.0*PI/2.0, ( 0.0f64,              -1.0f64           )),
                    (10.0/12.0,    300,   5.0*PI/3.0, ( 1.0f64/2.0,          -3.0f64.sqrt()/2.0)),
                    (11.0/12.0,    330,  11.0*PI/6.0, ( 3.0f64.sqrt()/2.0,   -1.0f64/2.0       )),
                    (12.0/12.0,    360,       2.0*PI, ( 1.0f64,               0.0f64           )),
        
                ]
            )
        ];
        for (freq, test_data) in test_data {
            let mut sampling_freq = SamplingFreq::new(freq as usize);
            for (target_t, target_angle_grad, target_angle, target_complex) in test_data.clone().into_iter().chain(test_data.into_iter().skip(1).map(|(t, ag, ar, (cr, ci))| (t + 1.0, ag, ar, (cr, ci)))) {
                log::debug!("target | time: {} sec,  angle: {:.4} ({}°), complex: ({:.3}, {:.3}i)", target_t, target_angle, target_angle_grad, target_complex.0, target_complex.1);
                let t = sampling_freq.next();
                log::debug!("result | time: {} sec,", t);
                // log::debug!("result | time: {} sec,  angle: {} ({}°), complex: ({}, {}i)", t, angle, angle_grad, complex.re, complex.im);
                let result = t;
                let target = target_t;
                assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", target_angle, target_angle_grad, result, target);
                // let result = angle;
                // let target = target_angle;
                // assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
                // let result = complex.re;
                // let target = target_re;
                // assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
                // let result = complex.im;
                // let target = target_im;
                // assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
            }
        }
        test_duration.exit();
    }
    ///
    /// Testing SamplingFreq.angle()
    #[test]
    fn angle() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        // Sampling freq
        let test_data = [
            (
                8, vec![
                    (0.0/8.0,      0,          0.0, ( 1.0f64,               0.0f64           )),
                    (1.0/8.0,     45,       PI/4.0, ( 2.0f64.sqrt()/2.0,    2.0f64.sqrt()/2.0)),
                    (2.0/8.0,     90,       PI/2.0, ( 0.0f64,               1.0f64           )),
                    (3.0/8.0,    135,   3.0*PI/4.0, (-2.0f64.sqrt()/2.0,    2.0f64.sqrt()/2.0)),
                    (4.0/8.0,    180,          PI,  (-1.0f64,               0.0f64           )),
                    (5.0/8.0,    225,   5.0*PI/4.0, (-2.0f64.sqrt()/2.0,   -2.0f64.sqrt()/2.0)),
                    (6.0/8.0,    270,   3.0*PI/2.0, ( 0.0f64,              -1.0f64           )),
                    (7.0/8.0,    315,   7.0*PI/4.0, ( 2.0f64.sqrt()/2.0,   -2.0f64.sqrt()/2.0)),
                    (8.0/8.0,    360,       2.0*PI, ( 1.0f64,               0.0f64           )),
                ],
            ),
            (
                12, vec![
                    ( 0.0/12.0,      0,          0.0, ( 1.0f64,               0.0f64           )),
                    ( 1.0/12.0,     30,       PI/6.0, ( 3.0f64.sqrt()/2.0,    1.0f64/2.0       )),
                    ( 2.0/12.0,     60,       PI/3.0, ( 1.0f64/2.0,           3.0f64.sqrt()/2.0)),
                    ( 3.0/12.0,     90,       PI/2.0, ( 0.0f64,               1.0f64           )),
                    ( 4.0/12.0,    120,   2.0*PI/3.0, (-1.0f64/2.0,           3.0f64.sqrt()/2.0)),
                    ( 5.0/12.0,    150,   5.0*PI/6.0, (-3.0f64.sqrt()/2.0,    1.0f64/2.0       )),
                    ( 6.0/12.0,    180,          PI,  (-1.0f64,               0.0f64           )),
                    ( 7.0/12.0,    210,   7.0*PI/6.0, (-3.0f64.sqrt()/2.0,   -1.0f64/2.0       )),
                    ( 8.0/12.0,    240,   4.0*PI/3.0, (-1.0f64/2.0,          -3.0f64.sqrt()/2.0)),
                    ( 9.0/12.0,    270,   3.0*PI/2.0, ( 0.0f64,              -1.0f64           )),
                    (10.0/12.0,    300,   5.0*PI/3.0, ( 1.0f64/2.0,          -3.0f64.sqrt()/2.0)),
                    (11.0/12.0,    330,  11.0*PI/6.0, ( 3.0f64.sqrt()/2.0,   -1.0f64/2.0       )),
                    (12.0/12.0,    360,       2.0*PI, ( 1.0f64,               0.0f64           )),
        
                ]
            )
        ];
        for (freq, test_data) in test_data {
            let mut sampling_freq = SamplingFreq::new(freq as usize);
            for (target_t, target_angle_grad, target_angle, target_complex) in test_data.clone().into_iter().chain(test_data.into_iter().skip(1).map(|(t, ag, ar, (cr, ci))| (t + 1.0, ag, ar, (cr, ci)))) {
                log::debug!("target | time: {} sec,  angle: {:.4} ({}°), complex: ({:.3}, {:.3}i)", target_t, target_angle, target_angle_grad, target_complex.0, target_complex.1);
                let t = sampling_freq.next();
                let angle = sampling_freq.angle(t);
                let angle_grad = 180.0 * angle / (2.0 * PI);
                log::debug!("result | time: {} sec,  angle: {:.4} ({}°)", t, angle, angle_grad);
                // log::debug!("result | time: {} sec,  angle: {} ({}°), complex: ({}, {}i)", t, angle, angle_grad, complex.re, complex.im);
                let result = t;
                let target = target_t;
                assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", target_angle, target_angle_grad, result, target);
                let result = angle;
                let target = target_angle;
                assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
                // let result = complex.re;
                // let target = target_re;
                // assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
                // let result = complex.im;
                // let target = target_im;
                // assert!(result.aprox_eq(target, 8), "angle {} ({}) \nresult: {:?}\ntarget: {:?}", angle, target_angle_grad, result, target);
            }
        }
        test_duration.exit();
    }
}
