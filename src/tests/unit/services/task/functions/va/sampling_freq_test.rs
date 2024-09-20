#[cfg(test)]

mod sampling_freq {
    use std::{f64::consts::PI, sync::Once, time::Duration};
    use rustfft::num_complex::Complex;
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
            8,
            12,
            16,
            24,
            36,
            64,
            128,
            256,
            516,
            1024,
            2048,
            4096,
            8192,
            16_484,
            32_768,
            65_536,
            131_072,
            262_144,
            524_288,
            1_048_576,
        ];
        for freq in test_data {
            let test_data = (0..freq * 2)
                .map(|i| {
                    let t = (i as f64) / (freq as f64);
                    let circle_index = i % freq;
                    let angle = (circle_index as f64) * PI * 2.0 / (freq as f64);
                    let complex = Complex::new(angle.cos(), angle.sin());
                    // log::debug!("target | freq: {}, i: {},  time: {} sec", freq, i, t);
                    (t, angle, angle * PI / 180.0, complex)
                });
            let mut sampling_freq = SamplingFreq::new(freq as usize, 0);
            for (target_t, target_angle, target_angle_grad, target_complex) in test_data.into_iter() {
                log::trace!("target | time: {} sec", target_t);
                let (t, complex) = sampling_freq.next();
                log::trace!("result | time: {} sec", t);
                let result = t;
                let target = target_t;
                assert!(result.aprox_eq(target, 8), "t: {:.4}, angle {:.4} ({:.4}) \nresult: {:?}\ntarget: {:?}", target_t, target_angle, target_angle_grad, result, target);
                let result = complex.re;
                let target = target_complex.re;
                assert!(result.aprox_eq(target, 8), "t: {:.4}, angle {:.4} ({:.4}) \nresult: {:?}\ntarget: {:?}", target_t, target_angle, target_angle_grad, result, target);
                let result = complex.im;
                let target = target_complex.im;
                assert!(result.aprox_eq(target, 8), "t: {:.4}, angle {:.4} ({:.4}) \nresult: {:?}\ntarget: {:?}", target_t, target_angle, target_angle_grad, result, target);
            }
        }
        test_duration.exit();
    }
}
