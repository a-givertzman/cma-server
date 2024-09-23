#[cfg(test)]

mod fn_va_fft {
    use core::f64;
    use std::{cell::RefCell, f64::consts::PI, rc::Rc, sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use concat_in_place::strcat;
    use rustfft::{num_complex::ComplexFloat, Fft, FftPlanner};
    use sal_sync::services::{
            entity::{name::Name, object::Object, point::{point::ToPoint, point_config_filters::PointConfigFilter, point_tx_id::PointTxId}}, retain::{retain_conf::RetainConf, retain_point_conf::RetainPointConf},
            service::service::Service, task::functions::conf::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions},
        };
    use testing::stuff::{max_test_duration::TestDuration, wait::WaitTread};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::fn_config::FnConfig, core_::{filter::{filter::{Filter, FilterEmpty}, filter_threshold::FilterThreshold}, types::fn_in_out_ref::FnInOutRef},
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
    // const YELLOW: &str = "\x1b[0;33m";
    // const NC: &str = "\x1b[0m";
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
            (     16,            16,    2,      vec![(  3.0, 50.0), (  5.0, 150.0), (   6.0, 200.0)]),
            (    128,           128,    2,      vec![( 16.0, 50.0), ( 36.0, 150.0), (  62.0, 200.0)]),
            (    256,           256,    2,      vec![(  2.0, 50.0), (  4.0, 150.0), (  12.0, 200.0), (  37.0, 20.0), (  112.0, 12.0), (  126.0, 15.0)]),
            ( 10_000,        10_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (4998.0, 300.0)]),
            ( 30_000,        30_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (9000.0, 200.0), (12000.0, 200.0), (14998.0, 300.0)]),
            (300_000,       300_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (9000.0, 200.0), (12000.0, 200.0), (24000.0, 250.0), (64000.0, 264.0), (120000.0, 280.0), (149998.0, 300.0)]),
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
            let export_point_name = "Fft";
            let conf = serde_yaml::from_str(&format!(r#"
                fn VaFft:
                    enable: const bool true         # optional, default true
                    send-to: {}.in-queue
                    conf point {}:                 # full name will be: /App/Task/Ffr.freq
                        type: 'Double'
                    input: point string /AppTest/Exit
                    freq: {}                        # Sampling freq
                    len: {}                         # Length of the                         
            "#, receiver_name, export_point_name, sampl_freq, fft_size)).unwrap();
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
    
            let fft: Arc<dyn Fft<f64>> = FftPlanner::new().plan_fft_forward(fft_size);
            let mut fft_buf = FftBuf::new(fft_size, sampl_freq);
            log::debug!("main | fft_buf.sampling_freq: {}", fft_buf.sampl_freq());
            assert!(fft_buf.sampl_freq() == sampl_freq, "\nresult: {:?}\ntarget: {:?}", fft_buf.sampl_freq(), sampl_freq);
            let fft_amp_factor = fft_buf.amp_factor();
            log::debug!("main | fft_buf.amp_factor: {}", fft_amp_factor);
            assert!(fft_amp_factor == 1.0 / ((fft_size as f64) / 2.0), "\nresult: {:?}\ntarget: {:?}", fft_amp_factor, 1.0 / ((fft_size as f64) / 2.0));
            let fft_freqs: Vec<String> = (0..fft_size / 2).map(|i| format!("{:?}", fft_buf.freq_of(i)) ).collect();
            let mut fft_filters: Vec<(String, Box<dyn Filter<Item = f64>>)> = (0..fft_size / 2).map(|i| {
                let freq_name = match fft_freqs.get(i) {
                    Some(freq) => strcat!(self_id export_point_name "." freq),
                    None => panic!("{}.out | Freq index {} out of the fft_size {}", self_id, i, fft_size),
                };
                (freq_name, filter(None))
            }).collect();
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
                log::trace!("main | {}  freq: {}  FnVaFft Elapsed: {:?}", step, sampl_freq, time.elapsed());

                match fft_buf.add(value) {
                    Some(buf) => {
                        // Pure FFT process
                        log::trace!("main | t: {:.4},  buf: {:?}", t, buf);
                        let time = Instant::now();
                        fft.process(buf);
                        log::debug!("main | freq: {}  Pure FFT Elapsed: {:?}", sampl_freq, time.elapsed());
                        // log::debug!("main | t: {:.4},  fft: {:?}", t, buf);
                        let mut fft_scalar: Vec<f64> = vec![];  //buf.iter().take(fft_size / 2).skip(1).map(|val| val.abs() * fft_amp_factor).collect();
                        for (index, val) in buf.iter().take(fft_size / 2).skip(1).enumerate() {
                            match fft_filters.get_mut(index) {
                                Some((_freq_name, filter)) => {
                                    filter.add(val.abs() * fft_amp_factor);
                                    if filter.is_changed() {
                                        fft_scalar.push(filter.value());
                                    }
                                }
                                None => panic!("main | fft_filters index {} out of size {}", index, fft_filters.len()),
                            }
                        }
                        log::trace!("main | t: {:.4},  fft_scalar: {:?}", t, fft_scalar.iter().map(|v| format!("{:.3}", v)).collect::<Vec<String>>());
                        ffts.push(fft_scalar.clone());

                        // Receiving FnVaFft results
                        let time = Instant::now();
                        while receiver.read().unwrap().received().read().unwrap().len() < fft_scalar.len() {
                            thread::sleep(Duration::from_millis(3));
                        }
                        let received = receiver.read().unwrap().received().read().unwrap().to_vec();
                        receiver.write().unwrap().clear_received();
                        log::debug!("main | FnVaFft received in {:?}, \t received: {}", time.elapsed(), received.len());
                        log::trace!("main | FnVaFft received: {:?}", received.iter().map(|v| format!("{:.3}", v.as_double().value)).collect::<Vec<String>>());
                        let mut va_fft_buf = vec![];
                        for point in &received {
                            va_fft_buf.push(point.as_double().value)
                        }

                        log::debug!("main |           target: {:?}", fft_scalar.iter().filter_map(|val| {
                            if *val > 0.0 {
                                Some(format!("{:.3}", val))
                            } else {
                                None
                            }
                        }).collect::<Vec<String>>());
                        log::debug!("main | FnVaFft received: {:?}", received.iter().filter_map(|v| {
                            let val = v.as_double().value;
                            if val > 0.0 {
                                Some(format!("{:.3}", v.as_double().value))
                            } else {
                                None
                            }
                        }).collect::<Vec<String>>());

                        if let Err((result, target)) = compare_vecs(&va_fft_buf, &fft_scalar)  {
                            panic!("main | FnVaFft({} sec) error \n result: {:?} \n target {:?}", t, result, target);
                            // log::error!("FnVaFft({} sec) error \n result: {:?} \n target {:?}", t, va_fft_buf, fft_scalar);
                        }
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
        }
        // assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        test_duration.exit();
    }
    ///
    /// Returns Threshold (key filter)
    fn filter(conf: Option<PointConfigFilter>) -> Box<dyn Filter<Item = f64>> {
        match conf {
            Some(conf) => {
                Box::new(
                    FilterThreshold::new(0.0f64, conf.threshold, conf.factor.unwrap_or(0.0))
                )
            }
            None => Box::new(FilterEmpty::<f64>::new(0.0)),
        }
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
