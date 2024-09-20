#[cfg(test)]

mod fn_va_fft {
    use core::f64;
    use std::{cell::RefCell, f64::consts::PI, rc::Rc, sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use rustfft::{num_complex::ComplexFloat, Fft, FftPlanner};
    use sal_sync::{
        collections::map::IndexMapFxHasher,
        services::{
            entity::{name::Name, object::Object, point::{point::ToPoint, point_tx_id::PointTxId}}, retain::{retain_conf::RetainConf, retain_point_conf::RetainPointConf},
            service::service::Service, task::functions::conf::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions},
        },
    };
    use testing::stuff::{max_test_duration::TestDuration, wait::WaitTread};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::fn_config::FnConfig, core_::{failure::errors_limit::ErrorLimit, types::fn_in_out_ref::FnInOutRef},
        services::{
            safe_lock::rwlock::SafeLock, services::Services,
            task::{
                nested_function::{fn_::FnOut, fn_input::FnInput, va::{fft_buff::FftBuf, fn_va_fft::FnVaFft}},
                task_test_receiver::TaskTestReceiver,
            }
        },
    };
    ///
    /// Colors
    const YELLOW: &str = "\x1b[0;33m";
    const NC: &str = "\x1b[0m";
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
    fn init_each(default: Option<&str>, type_: FnConfPointType) -> FnInOutRef {
        let mut conf = FnConfig { name: "test".to_owned(), type_, options: FnConfOptions {default: default.map(|d| d.into()), ..Default::default()}, ..Default::default()};
        Rc::new(RefCell::new(Box::new(
            FnInput::new("test", 0, &mut conf)
        )))
    }
    ///
    /// Testing FftBuf basic functionality
    #[test]
    fn basic() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        // init_each();
        log::debug!("");
        let self_id = "test";
        let self_name = Name::new("", self_id);
        let tx_id = PointTxId::from_str(&self_id);
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(30));
        test_duration.run().unwrap();
        let test_data = [
            // sampl_freq   fft_size    ffts    target
            (     12,            12,    1,      vec![(  2.0, 50.0), (  3.0, 150.0), (   4.0, 200.0)]),
            // (     16,            16,    2,      vec![(  3.0, 50.0), (  5.0, 150.0), (   6.0, 200.0)]),
            // (    128,           128,    2,      vec![( 16.0, 50.0), ( 36.0, 150.0), (  62.0, 200.0)]),
            // (    256,           256,    2,      vec![(  2.0, 50.0), (  4.0, 150.0), (  12.0, 200.0), (  37.0, 20.0), (  112.0, 12.0), (  126.0, 15.0)]),
            // ( 10_000,        10_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (4998.0, 300.0)]),
            // ( 30_000,        30_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (9000.0, 200.0), (12000.0, 200.0), (14998.0, 300.0)]),
            // (300_000,       300_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (9000.0, 200.0), (12000.0, 200.0), (24000.0, 250.0), (64000.0, 264.0), (120000.0, 280.0), (149998.0, 300.0)]),
        ];
        for (sampl_freq, fft_size, target_ffts, target_freqs) in test_data {
            let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(
                Some("assets/testing/retain/"),
                Some(RetainPointConf::new("point/id.json", None))
            ))));
            let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(
                self_id,
                "",
                "in-queue",
                usize::MAX,
            )));
            let receiver_name = receiver.read().unwrap().name().join();
            log::debug!("{} | receiver: '{}'", self_id, receiver_name);
            services.wlock(self_id).insert(receiver.clone());
            //
            // Configuring FnVaFft
            let enable = init_each(Some("true"), FnConfPointType::Bool);
            let fn_va_fft_input = init_each(None, FnConfPointType::Double);
            let conf = serde_yaml::from_str(&format!(r#"
                fn VaFft:
                    enable: const bool true         # optional, default true
                    send-to: {}.in-queue
                    conf point Fft:                 # full name will be: /App/Task/Ffr.freq
                        type: 'Double'
                    input: point string /AppTest/Exit
                    freq: {}                        # Sampling freq
                    len: {}                         # Length of the                         
            "#, receiver_name, sampl_freq, fft_size)).unwrap();
            let conf = match FnConfig::from_yaml(self_id, &self_name, &conf, &mut vec![]) {
                crate::conf::fn_::fn_conf_kind::FnConfKind::Fn(conf) => conf,
                _ => panic!("{} | Wrong VaFft config: {:#?}", self_id, conf),
            };
            let mut fn_va_fft = FnVaFft::new(self_id, Some(enable), fn_va_fft_input.clone(), conf, services.clone());
            let mut va_fft_buf = vec![];

            //
            // Runing all services
            let services_handle = services.wlock(self_id).run().unwrap();
            let receiver_handle = receiver.write().unwrap().run().unwrap();
            thread::sleep(Duration::from_millis(50));
            log::debug!("{} | All services started", self_id);
    
            let fft: Arc<dyn Fft<f64>> = FftPlanner::new().plan_fft_forward(fft_size);
            let mut fft_buf = FftBuf::new(fft_size, sampl_freq);
            log::debug!("main | fft_buf.sampling_freq: {}", fft_buf.sampl_freq());
            assert!(fft_buf.sampl_freq() == sampl_freq, "\nresult: {:?}\ntarget: {:?}", fft_buf.sampl_freq(), sampl_freq);
            let fft_amp_factor = fft_buf.amp_factor();
            log::debug!("main | fft_buf.amp_factor: {}", fft_amp_factor);
            assert!(fft_amp_factor == 1.0 / ((fft_size as f64) / 2.0), "\nresult: {:?}\ntarget: {:?}", fft_amp_factor, 1.0 / ((fft_size as f64) / 2.0));
            let mut ffts: Vec< Vec<f64> > = vec![];
            for step in 0..fft_size * target_ffts {
                let t = fft_buf.time();
                let value = target_freqs.iter().fold(0.0, |val, (freq, amp)| {
                    val + amp * (2. * PI *  freq * t).sin()
                });

                // FnVaFft process
                let time = Instant::now();
                fn_va_fft_input.borrow_mut().add(&value.to_point(tx_id, &format!("t: {}", t)));
                fn_va_fft.out();
                log::debug!("main | {}  freq: {}  FnVaFft Elapsed: {:?}", step, sampl_freq, time.elapsed());

                match fft_buf.add(value) {
                    Some(buf) => {
                        // Pure FFT process
                        // log::debug!("main | t: {:.4},  buf: {:?}", t, buf);
                        let time = Instant::now();
                        fft.process(buf);
                        log::debug!("main | freq: {}  Pure FFT Elapsed: {:?}", sampl_freq, time.elapsed());
                        // log::debug!("main | t: {:.4},  fft: {:?}", t, buf);
                        let fft_scalar: Vec<f64> = buf.iter().take(fft_size / 2).map(|val| val.abs() * fft_amp_factor).collect();
                        log::trace!("main | t: {:.4},  fft_scalar: {:?}", t, fft_scalar.iter().map(|v| format!("{:.3}", v)).collect::<Vec<String>>());
                        ffts.push(fft_scalar.clone());

        
                        // Receiving FnVaFft results
                        let time = Instant::now();
                        while receiver.read().unwrap().received().read().unwrap().len() < fft_scalar.len() {
                            thread::sleep(Duration::from_millis(10));
                        }
                        let received = receiver.read().unwrap().received().read().unwrap().to_vec();
                        receiver.write().unwrap().clear_received();
                        println!("main | FnVaFft received in {:?}, \t received: {}", time.elapsed(), received.len());
                        for point in &received {
                            va_fft_buf.push(point.as_double().value)
                        }
        
                        if let Err((result, target)) = compare_vecs(&va_fft_buf, &fft_scalar)  {
                            log::error!("main | FnVaFft({} sec) error \n result: {:?} \n target {:?}", t, result, target);
                            // log::error!("FnVaFft({} sec) error \n result: {:?} \n target {:?}", t, va_fft_buf, fft_scalar);
                        }
                        va_fft_buf = vec![];
                    }
                    None => {
                        log::trace!("main | t: {:.4}", t);
                    },
                };
            }

            receiver.read().unwrap().exit();
            services.rlock(self_id).exit();
            services_handle.wait().unwrap();
            receiver_handle.wait().unwrap();

            log::trace!("main | ffts: {}", ffts.len());
            let result = ffts.len();
            let target = target_ffts;
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
            for (step, fft) in ffts.into_iter().enumerate() {
                let mut error_limit = ErrorLimit::new(3);
                let mut detected_freqs = 0;
                // First elebent of fft_buf have to be skeeped because it refers to DC
                for (i, amp) in fft.into_iter().skip(1).enumerate() {
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
    ///
    /// Testing FnVaFft behavior
    // #[test]
    fn _tmp() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        log::debug!("");
        let self_id = "test";
        let self_name = Name::new("", self_id);
        let tx_id = PointTxId::from_str(&self_id);
        log::debug!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(30));
        // test_duration.run().unwrap();
        // Sampling freq
        let sampl_freq = 1000;
        // FFT Window size
        let fft_len = 100; //131_072;
        let frequencies = frequencies(sampl_freq, fft_len);
        let mut fft_buf = FftBuf::new(sampl_freq, 0);

        let mut buf = vec![];
        let fft = FftPlanner::new().plan_fft_forward(fft_len);
        // let mut input = vec![];
        // let mut series = vec![];
        let y_scale = 1.0 / (fft_len as f64);

        // Time of sampling, sec
        let until = 1.0;
        let mut t = 0.0;
        // Allowed accuracy for detected frequency
        let freq_accuracy = 0.15;
        // Allowed accuracy for amplitude in the detected frequency
        let amp_accuracy = 0.25;


        let mut rnd = rand::thread_rng();
        let mut target_freqs: Vec<(f64, f64)> = vec![];
        target_freqs.sort_by(|(freq1, _amp1), (freq2, _amp2)| {
            freq1.partial_cmp(freq2).unwrap()
        });
        let mut results = vec![];
        let mut steps = 0;
        let mut fft_operations = 0;
        while t < until {
            t = fft_buf.time();
            let value = target_freqs.iter().fold(0.0, |val, (freq, amp)| {
                val + amp * (2. * PI *  freq * t).sin()
            });
        buf.push(value);
            // Processing FnVaFft
            let val = value.abs();
            println!("t: {},  complex:  {},   module: {}", t, value, val);


            // println!("x: {}  |  y: {}", t, round(value.abs(), 3));
            if let Some(buf) = fft_buf.add(value) {
                
                // Processing pure FFT algorithm
                let timer = Instant::now();
                fft.process(buf);
                let elapsed = timer.elapsed();
                println!("Pure FFT elapsed  |  {:?}", elapsed);
                let fft_scalar: Vec<f64> = buf.iter().map(|complex| {
                    round(complex.abs() * y_scale, 3)
                }).collect();

                // println!("{}  |  {:?}", t, fft_scalar);
                // freq index  amplitude
                let mut sub_results = vec![];
                for (i, amplitude) in fft_scalar.iter().enumerate() {
                    // freq corresponding to index `i`
                    match frequencies.get(i) {
                        Some(freq_i) => {
                            match nierest_freq(*freq_i, &target_freqs) {
                                Some((test_freq, test_amp)) => {
                                    if *amplitude > test_amp * 0.5 {
                                        sub_results.push((*freq_i, *amplitude));
                                        if (amplitude - test_amp).abs() < test_amp * amp_accuracy {
                                            println!("{} sec  |  freq: {}, \tamp: {}   |   target freq: {}, target amp: {}", t, freq_i, amplitude, test_freq, test_amp);
                                        } else {
                                            println!("{} sec  |  freq: {}, \tamp: {}{}{}   |   target freq: {}, target amp: {}", t, freq_i, YELLOW, amplitude, NC, test_freq, test_amp);
                                        }

                                    }
                                }
                                None => log::error!("Not found nierest test freq for current {} Hz", freq_i),
                            }
                        }
                        None => log::error!("Frequencys[ {} ] cand be retrived, Frequencys.len: {}", i, frequencies.len()),
                    };
                }
                results.push(sub_results);
                // series.push(
                //     fft_scalar.into_iter().map(|y|, )
                // );
                fft_operations += 1;
            }
            steps += 1;
        }
        let mut error_limit = ErrorLimit::new((fft_operations as f64 * 0.2).round() as usize);
        // Report
        println!("Total fft frequencies: {}", frequencies.len());
        println!("Total test freqs ({}):", target_freqs.len());
        target_freqs.iter().for_each(|(freq, amp)| {
            println!("\t freq: {}  |  amp: {}", freq, amp);
        });
        println!("Total steps: {}", steps);
        println!("Total FFT's: {}", fft_operations);
        println!("Frequency accuracy: {}", freq_accuracy);
        println!("Amplitude accuracy: {}", amp_accuracy);
        // Main assert
        assert!(results.len() > 0, "\nresult: {:?}\ntarget: {:?}", results.len() > 0, true);
        let mut result_amps = IndexMapFxHasher::default();
        for (i, results) in results.into_iter().enumerate() {
            let targets = target_freqs.iter();
            for (step, (target_freq, target_amp)) in targets.enumerate() {
                match nierest_freq(*target_freq as f64, &results) {
                    Some((freq, amp)) => {
                        let result = freq as f64;
                        let target = *target_freq as f64;
                        if !((result - target).abs() < target * freq_accuracy) {
                            log::warn!("step: {}.{}, freq: {} \nresult: {:?}\ntarget: {:?}", i, step, freq, result, target);
                        }
                        let target_freq_key = target_freq.to_string();
                        let result_amp_err = result_amps.entry(target_freq_key.clone()).or_insert(ErrorLimit::new(fft_operations - 1));
                        // assert!((result - target).abs() < 6.01, "step: {}, freq: {} \nresult: {:?}\ntarget: {:?}", step, freq, result, target);
                        let result = amp;
                        let target = *target_amp;
                        if !((result - target).abs() < target * amp_accuracy) {
                            if let Err(_) = result_amp_err.add() {
                                if let Err(_) = error_limit.add() {
                                    panic!("step: {}.{}, freq: {} \nresult: {:?}\ntarget: {:?}", i, step, freq, result, target);
                                }
                                log::error!("step: {}.{}, freq: {}, errors: {} \nresult: {:?}\ntarget: {:?}", i, step, freq, result_amp_err.errors(), result, target);
                            } else {
                                log::debug!("step: {}.{}, freq: {}, errors: {} \nresult: {:?}\ntarget: {:?}", i, step, freq, result_amp_err.errors(),result, target);
                            }
                        }
                        // assert!((result - target).abs() < target * 0.5, "step: {}, freq: {} \nresult: {:?}\ntarget: {:?}", step, freq, result, target);
                    },
                    None => log::warn!("step: {}.{}, target freq: {} - not found in the results", i, step, target_freq),
                }
            }
        }
        // Plotting, disabled by default
        // let input_len = input.len();
        // series.push(
        //     input,
        // );
        // plot::plot("src/tests/unit/services/task/functions/va/plot_input.png", input_len / 2, series).unwrap();
        // println!("{:?}", f);
        test_duration.exit();
    }
    ///
    /// List of FFT frequencies
    fn frequencies(smpl_freq: usize, fft_len: usize) -> Vec<f64> {
        let delta = (smpl_freq as f64) / (fft_len as f64);
        let mut f = vec![0.0];
        while f.last().unwrap().to_owned() < (smpl_freq as f64) {
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
    ///
    /// Comparasion of vectors
    fn compare_vecs(v1: &[f64], v2: &[f64]) -> Result<(), (String, String)> {
        let mut result1 = String::new();
        let mut result2 = String::new();
        let (long, short, r1, r2) = if v1.len() >= v2.len() {
            (v1, v2, &mut result1, &mut result2)
        } else {
            (v2, v1, &mut result2, &mut result1)
        };
        let mut short_iter = short.into_iter();
        let mut matched = true;
        for value1 in long {
            match short_iter.next() {
                Some(value2) => {
                    if value1 == value2 {
                        r1.push_str(&format!("| {:.3} ",round(*value1, 3)));
                        r2.push_str(&format!("| {:.3} ",round(*value2, 3)));
                    } else {
                        matched = false;
                        r1.push_str(&format!("| - {:.3} - ",round(*value1, 3)));
                        r2.push_str(&format!("| - {:.3} - ",round(*value2, 3)));
                    }
                }
                None => {
                    matched = false;
                    let value1 = format!("| {:.3} ",round(*value1, 3));
                    let alignment = value1.len();
                    r1.push_str(&value1);
                    r2.push_str(&format!("| {:fill$} ", "-", fill = alignment - 3));
                }
            }
        }
        if matched {
            Ok(())
        } else {
            Err((result1, result2))
        }
    }

}
