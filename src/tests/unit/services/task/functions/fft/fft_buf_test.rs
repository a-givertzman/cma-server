#[cfg(test)]

mod fft_buf {
    use std::{f64::consts::PI, sync::{Arc, Once}, time::{Duration, Instant}};
    use rustfft::{num_complex::ComplexFloat, Fft, FftPlanner};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{core_::failure::errors_limit::ErrorLimit, services::task::nested_function::va::fft_buff::FftBuf};
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
    /// Testing FftBuf basic functionality
    #[test]
    fn basic() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        let test_data = [
            // sampl_freq   fft_size    ffts    target
            (     12,            12,    1,      vec![(  2.0, 50.0), (  3.0, 150.0), (   4.0, 200.0)]),
            (     16,            16,    2,      vec![(  3.0, 50.0), (  5.0, 150.0), (   6.0, 200.0)]),
            (    128,           128,    2,      vec![( 16.0, 50.0), ( 36.0, 150.0), (  62.0, 200.0)]),
            (    256,           256,    2,      vec![(  2.0, 50.0), (  4.0, 150.0), (  12.0, 200.0), (  37.0, 20.0), (  112.0, 12.0), (  126.0, 15.0)]),
            ( 10_000,        10_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (4998.0, 300.0)]),
            ( 30_000,        30_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (9000.0, 200.0), (12000.0, 200.0), (14998.0, 300.0)]),
            (300_000,       300_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (9000.0, 200.0), (12000.0, 200.0), (24000.0, 250.0), (64000.0, 264.0), (120000.0, 280.0), (149998.0, 300.0)]),
        ];
        for (sampl_freq, fft_size, target_ffts, target_freqs) in test_data {
            let fft: Arc<dyn Fft<f64>> = FftPlanner::new().plan_fft_forward(fft_size);
            let mut fft_buf = FftBuf::new(fft_size, sampl_freq);
            log::debug!("main | fft_buf.sampling_freq: {}", fft_buf.sampl_freq());
            assert!(fft_buf.sampl_freq() == sampl_freq, "\nresult: {:?}\ntarget: {:?}", fft_buf.sampl_freq(), sampl_freq);
            let fft_amp_factor = fft_buf.amp_factor();
            log::debug!("main | fft_buf.amp_factor: {}", fft_amp_factor);
            assert!(fft_amp_factor == 1.0 / ((fft_size as f64) / 2.0), "\nresult: {:?}\ntarget: {:?}", fft_amp_factor, 1.0 / ((fft_size as f64) / 2.0));
            let mut ffts: Vec< Vec<f64> > = vec![];
            for _ in 0..fft_size * target_ffts {
                let t = fft_buf.time();
                let value = target_freqs.iter().fold(0.0, |val, (freq, amp)| {
                    val + amp * (2. * PI *  freq * t).sin()
                });
                match fft_buf.add(value) {
                    Some(buf) => {
                        // log::debug!("main | t: {:.4},  buf: {:?}", t, buf);
                        let time = Instant::now();
                        fft.process(buf);
                        log::debug!("main | freq: {}  Elapsed: {:?}", sampl_freq, time.elapsed());
                        // log::debug!("main | t: {:.4},  fft: {:?}", t, buf);
                        // 
                        // Take half of the FFT results, because it's mirrired
                        // First elebent of fft_buf have to be skeeped because it refers to DC
                        let fft_scalar: Vec<f64> = buf.iter().take(fft_size / 2).skip(1).map(|val| val.abs() * fft_amp_factor).collect();
                        log::trace!("main | t: {:.4},  fft_scalar: {:?}", t, fft_scalar.iter().map(|v| format!("{:.3}", v)).collect::<Vec<String>>());
                        ffts.push(fft_scalar);
                    }
                    None => {
                        log::trace!("main | t: {:.4}", t);
                    },
                };
            }
            log::trace!("main | ffts: {}", ffts.len());
            let result = ffts.len();
            let target = target_ffts;
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
            for (step, fft) in ffts.into_iter().enumerate() {
                let mut error_limit = ErrorLimit::new(3);
                let mut detected_freqs = 0;
                for (i, amp) in fft.into_iter().enumerate() {
                    let freq = fft_buf.freq_of(i);
                    log::trace!("main | fft.freq[{}]: {}", i, freq);
                    if amp > 1.0 && freq > 0.0 {
                        match nierest_freq(freq, &target_freqs) {
                            Some((target_freq, target_amp)) => {
                                let freq_err = (100.0 * (target_freq - freq) / target_freq).abs();
                                let amp_err = (100.0 * (target_amp - amp) / target_amp).abs();
                                if freq_err < 5.0 && amp_err < 5.0 {
                                    log::debug!("main | fft.freq[{}]: {:.3} ({:.3} %), amp: {:.3} ({:.3} %)", i, freq, freq_err, amp, amp_err);
                                    detected_freqs += 1;
                                } else {
                                    log::warn!("main | fft.freq[{}]: {:.3} ({:.3} %), amp: {:.3} ({:.3} %)", i, freq, freq_err, amp, amp_err);
                                    if let Err(_) = error_limit.add() {
                                        panic!("main | errors limit ({}) exceeded", error_limit.limit());
                                    }
                                }
                            },
                            None => {
                                log::warn!("main | fft.freq[{}]: {:.3} - not found", i, freq);
                                if let Err(_) = error_limit.add() {
                                    panic!("main | errors limit ({}) exceeded", error_limit.limit());
                                }
                            },
                        }
                    }
                }
                let result = (((target_freqs.len() as f64) - (detected_freqs as f64)) / (target_freqs.len() as f64)).abs();
                let target = 0.15;
                assert!(result < target, "step {} \nresult:  {:.4}\ntarget: <{:.4}", step, result, target);
            }
        }
        // assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        test_duration.exit();
    }
    ///
    /// Returns nierest `freq` and coresponding Amp in `freqs`
    fn nierest_freq(freq: f64, freqs: &Vec<(f64, f64)>) -> Option<(f64, f64)> {
        let mut min_delta = f64::MAX;
        let mut delta;
        let mut result = None;
        for (f, amp) in freqs {
            delta = ((*f as f64) - freq).abs();
            if delta < min_delta {
                min_delta = delta;
                result = Some((*f, *amp));
            }
        }
        result
    }
}
