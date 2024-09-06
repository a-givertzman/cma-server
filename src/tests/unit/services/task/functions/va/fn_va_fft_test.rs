#[cfg(test)]

mod fn_va_fft {
    use core::f64;
    use std::{sync::Once, time::{Duration, Instant}};
    use rand::Rng;
    use rustfft::{num_complex::{Complex, ComplexFloat}, FftPlanner};
    use sal_sync::collections::map::IndexMapFxHasher;
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{services::task::nested_function::va::unit_circle::UnitCircle, tests::unit::services::task::functions::va::plot};
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
    /// Testing FFT it self behavior
    #[test]
    fn fft_anderstending() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(30));
        // test_duration.run().unwrap();
        // Sampling freq
        let sampling_freq = 300_000;
        // FFT Window size
        let fft_len = 262_144; //131_072;
        let frequencies = frequencies(sampling_freq, fft_len);
        let mut sampling_unit_circle = UnitCircle::new(sampling_freq);

        let mut buf = vec![];
        let fft = FftPlanner::new().plan_fft_forward(fft_len);
        let mut input = vec![];
        let mut series = vec![];
        let y_scale = 1.0 / (fft_len as f64);

        // Time of sampling, sec
        let until = 2.0;
        let mut t = 0.0;

        let mut rnd = rand::thread_rng();
        let mut test_freqs = IndexMapFxHasher::from_iter([]);
        let circles: Vec<(f64, UnitCircle)> = (0..rnd.gen_range(5..20)).map(|_| {
            let test_amp = rnd.gen_range(50.0..200.0);
            let test_freq = rnd.gen_range(sampling_freq / 1000..sampling_freq / 10);
            test_freqs.insert(test_freq, test_amp);
            (test_amp, UnitCircle::new(test_freq))
        })
        .collect();
        test_freqs.sort_by(|freq1, _amp1, freq2, _amp2| {
            freq1.cmp(freq2)
        });
        let mut results = vec![];
        let mut steps = 0;
        let mut fft_procs = 0;
        while t < until {
            (t, _, _) = sampling_unit_circle.next();
            let value: Complex<f64> = circles.iter()
                .map(|(amp, circle)| circle.at_with(t, *amp))
                .map(|(_angle, complex)| complex).sum();
            buf.push(value);
            // println!("x: {}  |  y: {}", t, round(value.abs(), 3));
            input.push(
                (t, value.abs())
            );
            if buf.len() >= fft_len {
                let timer = Instant::now();
                fft.process(&mut buf);
                let elapsed = timer.elapsed();
                let fft_scalar: Vec<f64> = buf.iter().map(|complex| {
                    round(complex.abs() * y_scale, 3)
                }).collect();
                // println!("{}  |  {:?}", t, fft_scalar);
                for (i, value) in fft_scalar.iter().enumerate() {
                    match frequencies.get(i) {
                        Some(freq_i) => {
                            match nierest_freq(*freq_i, &test_freqs) {
                                Some((test_freq, test_amp)) => {
                                    if *value > test_amp / 2.0 {
                                        results.push((*freq_i, *value));
                                        println!("{} sec  |  freq: {}, \tamp: {}   |   target freq: {}, anp: {}", t, freq_i, value, test_freq, test_amp);
                                    }
                                }
                                None => log::error!("Not found nierest test freq for current {} Hz", freq_i),
                            }
                        }
                        None => log::error!("Frequencys[ {} ] cand be retrived, Frequencys.len: {}", i, frequencies.len()),
                    };
                }
                println!("elapsed  |  {:?}", elapsed);
                // series.push(
                //     fft_scalar.into_iter().map(|y|, )
                // );
                buf = vec![];
                fft_procs += 1;
            }
            steps += 1;
        }
        // Report
        println!("Total fft frequencies: {}", frequencies.len());
        println!("Total test freqs ({}):", test_freqs.len());
        test_freqs.iter().for_each(|(freq, amp)| {
            println!("\t freq: {}  |  amp: {}", freq, amp);
        });
        println!("Total steps: {}", steps);
        println!("Total FFT's: {}", fft_procs);
        // Main assert
        assert!(results.len() > 0, "\nresult: {:?}\ntarget: {:?}", results.len() > 0, true);
        let mut targets = test_freqs.iter();
        for (step, (freq, amp)) in results.into_iter().enumerate() {
            match targets.next() {
                Some((target_freq, target_amp)) => {
                    let result = freq as f64;
                    let target = *target_freq as f64;
                    if !((result - target).abs() < 6.01) {
                        log::warn!("step: {}, freq: {} \nresult: {:?}\ntarget: {:?}", step, freq, result, target);
                    }
                    // assert!((result - target).abs() < 6.01, "step: {}, freq: {} \nresult: {:?}\ntarget: {:?}", step, freq, result, target);
                    let result = amp;
                    let target = *target_amp;
                    if !((result - target).abs() < target * 0.2) {
                        log::warn!("step: {}, freq: {} \nresult: {:?}\ntarget: {:?}", step, freq, result, target);
                    }
                    // assert!((result - target).abs() < target * 0.5, "step: {}, freq: {} \nresult: {:?}\ntarget: {:?}", step, freq, result, target);
                }
                None => log::warn!("step: {}, freq: {} - target not found", step, freq),
            };
        }
        // Plotting, disabled by default
        // let input_len = input.len();
        series.push(
            input,
        );
        // plot::plot("src/tests/unit/services/task/functions/va/plot_input.png", input_len / 2, series).unwrap();
        // println!("{:?}", f);
        test_duration.exit();
    }
    ///
    /// Returns nierest `freq` and coresponding Amp in `freqs`
    fn nierest_freq(freq: f64, freqs: &IndexMapFxHasher<usize, f64>) -> Option<(usize, f64)> {
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
    ///
    /// List of frequencies
    fn frequencies(smpling_freq: usize, fft_len: usize) -> Vec<f64> {
        let delta = (smpling_freq as f64) / (fft_len as f64);
        let mut f = vec![0.0];
        while f.last().unwrap().to_owned() < (smpling_freq as f64) {
            f.push(
                round(f.last().unwrap() + delta, 3)
            );
        }
        f
    }
    ///
    /// Returns float rounded to the specified digits
    fn round(value: f64, digits: usize) -> f64 {
        let factor = 10.0f64.powi(digits as i32);
        (value * factor).round() / factor
    }
    struct Average<T> {
        count: u32,
        sum: T,
    }
    // impl Average<f64> {
    //     pub fn new() -> Self {
    //         Self { count: 0, sum: 0.0 }
    //     }
    //     pub fn add(&mut self, value: f64) {
    //         self.sum = self.sum + value;
    //         self.count += 1;
    //     }
    //     pub fn eval(&self) -> f64 {
    //         self.sum / (self.count as f64)
    //     }
    // }
    impl Average<std::time::Duration> {
        pub fn new() -> Self {
            Self { count: 0, sum: std::time::Duration::ZERO }
        }
        pub fn add(&mut self, value: std::time::Duration) {
            self.sum = self.sum + value;
            self.count += 1;
        }
        pub fn eval(&self) -> std::time::Duration {
            self.sum / self.count
        }
    }
}
