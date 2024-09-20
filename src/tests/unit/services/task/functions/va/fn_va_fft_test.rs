#[cfg(test)]

mod fn_va_fft {
    use core::f64;
    use std::{cell::RefCell, f64::consts::PI, rc::Rc, sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use rand::Rng;
    use rustfft::{num_complex::{Complex, ComplexFloat}, FftPlanner};
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
                nested_function::{fn_::FnOut, fn_input::FnInput, va::{fft_buff::FftBuf, fn_va_fft::FnVaFft, unit_circle::UnitCircle}},
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
    /// Testing FFT it self behavior
    #[test]
    fn fft_anderstending() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        log::debug!("");
        let self_id = "test";
        log::debug!("{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(30));
        // test_duration.run().unwrap();
        // Sampling freq
        let sampl_freq = 10_000;
        // FFT Window size
        let fft_size = 10_000; // 16_384 // 131_072;
        let frequencies = frequencies(sampl_freq, fft_size / 2);
        let mut fft_buf = FftBuf::new(sampl_freq, fft_size);

        let fft = FftPlanner::new().plan_fft_forward(fft_size);
        // let mut input = vec![];
        // let mut series = vec![];
        let y_scale = 1.0 / (fft_size as f64);

        // Time of sampling, sec
        let until = 0.1;
        let mut t = 0.0;
        // Allowed accuracy for detected frequency
        let freq_accuracy = 0.15;
        // Allowed accuracy for amplitude in the detected frequency
        let amp_accuracy = 0.5;

        let mut rnd = rand::thread_rng();
        let mut test_freqs = vec![];
        let circles: Vec<(f64, UnitCircle)> = (0..rnd.gen_range(3..4)).map(|_| {
            let test_amp = rnd.gen_range(50.0..200.0);
            let test_freq = rnd.gen_range(sampl_freq / 1000..sampl_freq / 10);
            test_freqs.push((test_freq as f64, test_amp));
            (test_amp, UnitCircle::new(test_freq))
        }).collect();
        test_freqs.sort_by(|(freq1, _amp1), (freq2, _amp2)| {
            freq1.partial_cmp(freq2).unwrap()
        });
        let mut results = vec![];
        let mut steps = 0;
        let mut fft_operations = 0;
        let u_circle_1 = UnitCircle::new(3000);
        let u_circle_1 = UnitCircle::new(1000);
        // let mut complex;
        while t < until {
            let t = fft_buf.time();
            // let value: Complex<f64> = circles.iter()
            //     .map(|(amp, circle)| {
            //         // let alpha = 2.0 * PI * (circle.freq as f64) * t;
            //         // let alpha = circle.angle(t);
            //         // Complex::new(amp * alpha.cos(), amp * alpha.sin())
            //         let (_alpha, complex) = circle.at_with(t, *amp);
            //         complex
            //     })
            //     .sum();
            let value = u_circle_1.at_with(t, 10.0).1.abs();
            match fft_buf.add(value) {
                Some(buf) => {
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
                    for (i, amplitude) in fft_scalar.iter().take(fft_size / 2).enumerate() {
                        let freq_i = fft_buf.freq_of(i);
                        if *amplitude > 1.0 {
                            println!("{} sec  |  freq: {:.4}, \tamp: {:.4}", t, freq_i, amplitude);
                        }
                        // freq corresponding to index `i`
                        // match frequencies.get(i) {
                        //     Some(freq_i) => {
                        //         match nierest_freq(*freq_i, &test_freqs) {
                        //             Some((test_freq, test_amp)) => {
                        //                 if *amplitude > test_amp * 0.5 {
                        //                     sub_results.push((*freq_i, *amplitude));
                        //                     if (amplitude - test_amp).abs() < test_amp * amp_accuracy {
                        //                         println!("{} sec  |  freq: {}, \tamp: {}   |   target freq: {}, target amp: {}", t, freq_i, amplitude, test_freq, test_amp);
                        //                     } else {
                        //                         println!("{} sec  |  freq: {}, \tamp: {}{}{}   |   target freq: {}, target amp: {}", t, freq_i, YELLOW, amplitude, NC, test_freq, test_amp);
                        //                     }

                        //                 }
                        //             }
                        //             None => log::error!("Not found nierest test freq for current {} Hz", freq_i),
                        //         }
                        //     }
                        //     None => log::error!("Frequencys[ {} ] cand be retrived, Frequencys.len: {}", i, frequencies.len()),
                        // };
                    }
                    results.push(sub_results);
                    // series.push(
                    //     fft_scalar.into_iter().map(|y|, )
                    // );
                    fft_operations += 1;
                }
                None => {},
            }
            steps += 1;
        }
        let mut error_limit = ErrorLimit::new((fft_operations as f64 * 0.2).round() as usize);
        // Report
        println!("Total fft frequencies: {}", frequencies.len());
        println!("Total test freqs ({}):", test_freqs.len());
        test_freqs.iter().for_each(|(freq, amp)| {
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
            let targets = test_freqs.iter();
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
    /// Testing FnVaFft behavior
    // #[test]
    fn basic() {
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

        let mut va_fft_buf = vec![];
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
                freq: 1000                    # Sampling freq
                len: 100                      # Length of the                         
        "#, receiver.read().unwrap().name())).unwrap();
        let conf = match FnConfig::from_yaml(self_id, &self_name, &conf, &mut vec![]) {
            crate::conf::fn_::fn_conf_kind::FnConfKind::Fn(conf) => conf,
            _ => panic!("{} | Wrong VaFft config: {:#?}", self_id, conf),
        };
        let mut fn_va_fft = FnVaFft::new(self_id, Some(enable), fn_va_fft_input.clone(), conf, services.clone());
        //
        // Runing all services
        let services_handle = services.wlock(self_id).run().unwrap();
        let receiver_handle = receiver.write().unwrap().run().unwrap();
        thread::sleep(Duration::from_millis(50));
        log::debug!("{} | All services started", self_id);

        let mut rnd = rand::thread_rng();
        let mut test_freqs = vec![];
        let circles: Vec<(f64, UnitCircle)> = (0..rnd.gen_range(1..20)).map(|_| {
            let test_amp = rnd.gen_range(50.0..200.0);
            let test_freq = rnd.gen_range(sampl_freq / 1000..sampl_freq / 10);
            test_freqs.push((test_freq as f64, test_amp));
            (test_amp, UnitCircle::new(test_freq))
        }).collect();
        test_freqs.sort_by(|(freq1, _amp1), (freq2, _amp2)| {
            freq1.partial_cmp(freq2).unwrap()
        });
        let mut results = vec![];
        let mut steps = 0;
        let mut fft_operations = 0;
        while t < until {
            // (t, _) = fft_buf.next();
            let value: Complex<f64> = circles.iter()
                .map(|(amp, circle)| {
                    // let alpha = 2.0 * PI * (circle.freq as f64) * t;
                    // let alpha = circle.angle(t);
                    // Complex::new(amp * alpha.cos(), amp * alpha.sin())
                    let (_alpha, complex) = circle.at_with(t, *amp);
                    complex
                })
                .sum();
            buf.push(value);
            // Processing FnVaFft
            let val = value.abs();
            println!("t: {},  complex:  {},   module: {}", t, value, val);
            let timer = Instant::now();
            fn_va_fft_input.borrow_mut().add(&val.to_point(tx_id, &format!("t: {}", t)));
            fn_va_fft.out();
            let va_fft_elapsed = timer.elapsed();
            // println!("FnVaFft elapsed  |  {:?}", elapsed);
            // println!("x: {}  |  y: {}", t, round(value.abs(), 3));
            if buf.len() >= fft_len {
                
                // Processing pure FFT algorithm
                let timer = Instant::now();
                fft.process(&mut buf);
                let elapsed = timer.elapsed();
                println!("Pure FFT elapsed  |  {:?}", elapsed);
                let fft_scalar: Vec<f64> = buf.iter().map(|complex| {
                    round(complex.abs() * y_scale, 3)
                }).collect();

                // Receiving FnVaFft results
                while receiver.read().unwrap().received().read().unwrap().len() < fft_scalar.len() {
                    thread::sleep(Duration::from_millis(10));
                }
                let received = receiver.read().unwrap().received().read().unwrap().to_vec();
                receiver.write().unwrap().clear_received();
                println!("FnVaFft elapsed  |  {:?}, \t received: {}", va_fft_elapsed, received.len());
                for point in &received {
                    va_fft_buf.push(point.as_double().value)
                }

                if let Err((result, target)) = compare_vecs(&va_fft_buf, &fft_scalar)  {
                    log::error!("FnVaFft({} sec) error \n result: {:?} \n target {:?}", t, result, target);
                    // log::error!("FnVaFft({} sec) error \n result: {:?} \n target {:?}", t, va_fft_buf, fft_scalar);
                }
                va_fft_buf = vec![];
                // println!("{}  |  {:?}", t, fft_scalar);
                // freq index  amplitude
                let mut sub_results = vec![];
                for (i, amplitude) in fft_scalar.iter().enumerate() {
                    // freq corresponding to index `i`
                    match frequencies.get(i) {
                        Some(freq_i) => {
                            match nierest_freq(*freq_i, &test_freqs) {
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
                buf = vec![];
                fft_operations += 1;
            }
            steps += 1;
        }
        let mut error_limit = ErrorLimit::new((fft_operations as f64 * 0.2).round() as usize);
        // Report
        println!("Total fft frequencies: {}", frequencies.len());
        println!("Total test freqs ({}):", test_freqs.len());
        test_freqs.iter().for_each(|(freq, amp)| {
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
            let targets = test_freqs.iter();
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
        receiver.read().unwrap().exit();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        receiver_handle.wait().unwrap();
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
