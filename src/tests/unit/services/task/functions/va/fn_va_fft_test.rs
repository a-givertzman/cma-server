#[cfg(test)]

use core::f64;
use std::{cell::RefCell, f64::consts::PI, rc::Rc, sync::{Arc, Once}, thread, time::{Duration, Instant}};
use concat_string::concat_string;
use rustfft::{num_complex::ComplexFloat, Fft, FftPlanner};
use sal_sync::{math::AproxEq, services::{
    conf::{ConfTree, ServicesConf}, entity::{Name, Object, PointConfFilter, PointTxId, ToPoint}, task::functions::{FnConfKind, FnConfOptions, FnConfPointType, FnConfig}, Service, Services
}, thread_pool::ThreadPool};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    domain::{filter::{filter::{Filter, FilterEmpty}, filter_threshold::FilterThreshold}, FnInOutRef},
    services::task::{
        {FnOut, FnInput, FftBuf, FnVaFft},
        TaskTestReceiver,
    }, tests::tools::{plot, SeriesKind},
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
    Rc::new(RefCell::new(
        FnInput::new("test", 0, &mut conf)
    ))
}
///
/// Testing FftBuf with empty filter
#[test]
fn empty_filter() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    // init_each();
    log::debug!("");
    let dbg = "empty_filter-test";
    let self_name = Name::new("", dbg);
    let tx_id = PointTxId::from_str(&dbg);
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(60));
    test_duration.run().unwrap();
    let test_data = [
        // sampl_freq   fft_size    ffts    target
        (     12,            12,    1,      vec![(  2.0, 50.0), (  3.0, 150.0), (   4.0, 200.0)]),
        (     16,            16,    2,      vec![(  3.0, 50.0), (  5.0, 150.0), (   6.0, 200.0)]),
        (    128,           128,    4,      vec![( 16.0, 50.0), ( 36.0, 150.0), (  62.0, 200.0)]),
        (    256,           256,    3,      vec![(  2.0, 50.0), (  4.0, 150.0), (  12.0, 200.0), ( 37.0,  20.0), (112.0, 12.0), (  126.0, 15.0)]),
        ( 10_000,        10_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (4998.0, 300.0)]),
        ( 30_000,        30_000,   20,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 140.0), (4000.0, 200.1), (9000.0, 210.2), (12000.0, 220.3), (14998.0, 300.0)]),
        (300_000,       300_000,    2,      vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 201.1), (9000.0, 202.2), (12000.0, 203.3), (24000.0, 250.0), (64000.0, 264.0), (120000.0, 280.0), (149998.0, 300.0)]),
    ];
    let tp = ThreadPool::new(dbg, Some(12));
    for (sampl_freq, fft_size, target_ffts, target_freqs) in test_data {
        let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), Some(tp.scheduler())));
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            usize::MAX,
        ));
        let receiver_name = receiver.name().join();
        log::debug!("{} | receiver: '{}'", dbg, receiver_name);
        services.insert(receiver.clone());
        //
        // Configuring FnVaFft
        let enable = init_each(Some("true"), FnConfPointType::Bool);
        let fn_va_fft_input = init_each(None, FnConfPointType::Double);
        let export_point_name = "/";
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
        let conf = match FnConfig::from_yaml(dbg, &self_name, &conf, &mut vec![]) {
            FnConfKind::Fn(conf) => conf,
            _ => panic!("{} | Wrong VaFft config: {:#?}", dbg, conf),
        };
        let mut fn_va_fft = FnVaFft::new(dbg, Some(enable), fn_va_fft_input.clone(), conf, services.clone());

        //
        // Runing all services
        services.run().unwrap();
        receiver.run().unwrap();
        thread::sleep(Duration::from_millis(50));
        log::debug!("{} | All services started", dbg);

        let fft: Arc<dyn Fft<f64>> = FftPlanner::new().plan_fft_forward(fft_size);
        let mut fft_buf = FftBuf::new(fft_size);
        // log::debug!("{dbg} | fft_buf.sampling_freq: {}", fft_buf.sampl_freq());
        // assert!(fft_buf.sampl_freq() == sampl_freq, "\nresult: {:?}\ntarget: {:?}", fft_buf.sampl_freq(), sampl_freq);
        let fft_amp_factor = fft_buf.amp_factor();
        log::debug!("{dbg} | fft_buf.amp_factor: {}", fft_amp_factor);
        assert!(fft_amp_factor == 1.0 / ((fft_size as f64) / 2.0), "\nresult: {:?}\ntarget: {:?}", fft_amp_factor, 1.0 / ((fft_size as f64) / 2.0));
        let fft_freqs: Vec<String> = (0..fft_size / 2).map(|i| format!("{:?}", fft_buf.freq_of(sampl_freq, i)) ).collect();
        let mut fft_filters: Vec<(String, Box<dyn Filter<Item = f64>>)> = (0..fft_size / 2).map(|i| {
            let freq_name = match fft_freqs.get(i) {
                Some(freq) => concat_string!(dbg, export_point_name, ".", freq),
                None => panic!("{}.out | Freq index {} out of the fft_size {}", dbg, i, fft_size),
            };
            (freq_name, filter(None))
        }).collect();
        // reference FFT's, calculated locally
        let mut ref_ffts: Vec< Vec<f64> > = vec![];
        // FFT's, calculated by FnVaFft
        let mut received_ffts: Vec< Vec<f64> > = vec![];
        // sequences used for plotting charts
        let mut plot_values: Vec<(f64, f64)> = vec![];
        let mut plot_ffts: Vec<Vec<(f64, f64)>> = vec![];
        for step in 0..fft_size * target_ffts {
            let t = FftBuf::time(sampl_freq, step);
            let value = target_freqs.iter().fold(0.0, |val, (freq, amp)| {
                val + amp * (2. * PI *  freq * t).sin()
            });
            plot_values.push((t, value));
            // FnVaFft process
            let time = Instant::now();
            // add new sample to the fn_va_fft input
            fn_va_fft_input.borrow_mut().add(&value.to_point(tx_id, &format!("t: {}", t)));
            // process fn_va_fft, if changes detected on inner fft filters, it will be sent to the receiver
            fn_va_fft.out();
            log::trace!("{dbg} | {}  freq: {}  FnVaFft Elapsed: {:?}", step, sampl_freq, time.elapsed());
            match fft_buf.add(value) {
                Some(buf) => {
                    // Pure FFT process
                    log::trace!("{dbg} | t: {:.4},  buf: {:?}", t, buf);
                    let time = Instant::now();
                    fft.process(buf);
                    log::debug!("{dbg} | freq: {}  Pure FFT Elapsed: {:?}", sampl_freq, time.elapsed());
                    // log::debug!("{dbg} | t: {:.4},  fft: {:?}", t, buf);
                    let mut fft_scalar: Vec<f64> = vec![];  //buf.iter().take(fft_size / 2).skip(1).map(|val| val.abs() * fft_amp_factor).collect();
                    for (index, val) in buf.iter().take(fft_size / 2).skip(1).enumerate() {
                        match fft_filters.get_mut(index) {
                            Some((_freq_name, filter)) => {
                                if let Some(filter_value) = filter.add(val.abs() * fft_amp_factor) {
                                    fft_scalar.push(filter_value);
                                }
                            }
                            None => panic!("{dbg} | fft_filters index {} out of size {}", index, fft_filters.len()),
                        }
                    }
                    log::trace!("{dbg} | t: {:.4},  fft_scalar: {:?}", t, fft_scalar.iter().map(|v| format!("{:.3}", v)).collect::<Vec<String>>());
                    ref_ffts.push(fft_scalar.clone());

                    // Receiving FnVaFft results
                    let time = Instant::now();
                    while receiver.received().len() < fft_scalar.len() {
                        thread::sleep(Duration::from_millis(3));
                    }
                    let received = receiver.drain(0..fft_scalar.len());
                    log::debug!("{dbg} | FnVaFft received in {:?}, \t received: {}", time.elapsed(), received.len());
                    log::trace!("{dbg} | FnVaFft received: {:?}", received.iter().map(|v| format!("{:.3}", v.as_double().value)).collect::<Vec<String>>());
                    let mut va_fft_buf = vec![];
                    let mut plot_ffts_step = vec![];
                    for point in &received {
                        va_fft_buf.push(point.as_double().value);
                        let val = point.as_double().value;
                        let freq: f64 = point.name().parse().unwrap();
                        println!("\t| received  {:.3}, {:.4}", freq, val);
                        if val > 1.0 {
                            plot_ffts_step.push((freq, val + (received_ffts.len() as f64) * 10.0));
                        }
                    }
                    if !plot_ffts_step.is_empty() {
                        plot_ffts.push(plot_ffts_step);
                    }
                    if !va_fft_buf.is_empty() {
                        received_ffts.push(va_fft_buf.clone());
                    }

                    log::debug!("{dbg} |           target: {:?}", fft_scalar.iter().filter_map(|val| {
                        (*val > 1.0).then(|| format!("{:.3}", val))
                    }).collect::<Vec<String>>());
                    log::debug!("{dbg} | FnVaFft received: {:?}", received.iter().filter_map(|point| {
                        let val = point.as_double().value;
                        // let freq: f64 = v.name().parse().unwrap();
                        // println!("\t| received  {:.3}, {:?}", freq, val);
                        (val > 1.0).then(|| format!("{:.3}", val))
                    }).collect::<Vec<String>>());

                    if let Err((result, target)) = compare_vecs(&va_fft_buf, &fft_scalar, None)  {
                        panic!("{dbg} | FnVaFft({} sec) error \n result: {:?} \n target {:?}", t, result, target);
                        // log::error!("FnVaFft({} sec) error \n result: {:?} \n target {:?}", t, va_fft_buf, fft_scalar);
                    }
                }
                None => {
                    log::trace!("{dbg} | t: {:.4}", t);
                },
            };
        }
        
        receiver.exit();
        services.exit();
        services.wait().unwrap();
        receiver.wait().unwrap();

        let path = "src/tests/unit/services/task/functions/va/empty-filter";
        plot(format!("{path}/values-{:?}.png", sampl_freq), None, vec![plot_values], SeriesKind::Both).unwrap();
        plot(format!("{path}/ffts-{:?}.png", sampl_freq), None, plot_ffts, SeriesKind::Points).unwrap();

        log::debug!("{dbg} | ffts: \n\t target: {}, \n\t ref calculated: {}, \n\t  va calculated: {}", target_ffts, ref_ffts.len(), received_ffts.len());
        let result = ref_ffts.len();
        let target = target_ffts;
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        let result = received_ffts.len();
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    // assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    test_duration.exit();
}
///
/// Testing FftBuf with absolute threshold filter
#[test]
#[ignore = "!!! TO BE FIXED !!!"]
fn absolute_filter() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    // init_each();
    log::debug!("");
    let dbg = "absolute_filter-test";
    let self_name = Name::new("", dbg);
    let tx_id = PointTxId::from_str(&dbg);
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(60));
    test_duration.run().unwrap();
    let test_data = [
        //sampl_freq  fft_size  ffts  threshold     target_ffts     target
        // (     12,         12,    1,   5.0,          1,              vec![(  2.0, 50.0), (  3.0, 150.0), (   4.0, 200.0)]),
        // (     16,         16,    2,   5.0,          1,              vec![(  3.0, 50.0), (  5.0, 150.0), (   6.0, 200.0)]),
        // (    128,        128,    4,   5.0,          1,              vec![( 16.0, 50.0), ( 36.0, 150.0), (  62.0, 200.0)]),
        // (    256,        256,    4,   5.0,          1,              vec![(  2.0, 50.0), (  4.0, 150.0), (  12.0, 200.0), ( 37.0,  20.0), (112.0,  12.0), ( 126.0,  15.0)]),
        // ( 10_000,     10_000,    2,   5.0,          1,              vec![(  5.0,  5.1), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.0), (4998.0, 300.0)]),
        ( 30_000,     30_000,    2,   5.0,          1,              vec![(  5.0,  5.1), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 200.1), (9000.0, 220.2), (12000.0, 230.3), (14998.0, 300.0)]),
        // (300_000,    300_000,    2,   5.0,          1,              vec![(  5.0,  5.0), ( 10.0,  10.0), (  50.0,  50.0), (100.0, 100.0), (400.0, 150.0), (4000.0, 201.1), (9000.0, 202.2), (12000.0, 203.3), (24000.0, 250.0), (64000.0, 264.0), (120000.0, 280.0), (149998.0, 300.0)]),
    ];
    let tp = ThreadPool::new(dbg, Some(8));
    for (sampl_freq, fft_size, ffts, threshold, target_ffts, target_freqs) in test_data {
        let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::empty()), Some(tp.scheduler())));
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            usize::MAX,
        ));
        let receiver_name = receiver.name().join();
        log::debug!("{} | receiver: '{}'", dbg, receiver_name);
        services.insert(receiver.clone());
        //
        // Configuring FnVaFft
        let enable = init_each(Some("true"), FnConfPointType::Bool);
        let fn_va_fft_input = init_each(None, FnConfPointType::Double);
        let export_point_name = "/";
        let conf = serde_yaml::from_str(&format!(r#"
            fn VaFft:
                enable: const bool true         # optional, default true
                send-to: {}.in-queue
                conf point {}:                 # full name will be: /App/Task/Ffr.freq
                    type: 'Double'
                input: point string /AppTest/Exit
                freq: {}                        # Sampling freq
                len: {}                         # Length of the
                filter:
                    threshold: {:?}
        "#, receiver_name, export_point_name, sampl_freq, fft_size, threshold)).unwrap();
        let conf = match FnConfig::from_yaml(dbg, &self_name, &conf, &mut vec![]) {
            FnConfKind::Fn(conf) => conf,
            _ => panic!("{} | Wrong VaFft config: {:#?}", dbg, conf),
        };
        let mut fn_va_fft = FnVaFft::new(dbg, Some(enable), fn_va_fft_input.clone(), conf, services.clone());

        //
        // Runing all services
        services.run().unwrap();
        receiver.run().unwrap();
        thread::sleep(Duration::from_millis(50));
        log::debug!("{} | All services started", dbg);

        let fft: Arc<dyn Fft<f64>> = FftPlanner::new().plan_fft_forward(fft_size);
        let mut fft_buf = FftBuf::new(fft_size);
        // log::debug!("{dbg} | fft_buf.sampling_freq: {}", fft_buf.sampl_freq());
        // assert!(fft_buf.sampl_freq() == sampl_freq, "\nresult: {:?}\ntarget: {:?}", fft_buf.sampl_freq(), sampl_freq);
        let fft_amp_factor = fft_buf.amp_factor();
        log::debug!("{dbg} | fft_buf.amp_factor: {}", fft_amp_factor);
        assert!(fft_amp_factor == 1.0 / ((fft_size as f64) / 2.0), "\nresult: {:?}\ntarget: {:?}", fft_amp_factor, 1.0 / ((fft_size as f64) / 2.0));
        let fft_freqs: Vec<String> = (0..fft_size / 2).map(|i| format!("{:?}", fft_buf.freq_of(sampl_freq, i)) ).collect();
        let mut fft_filters: Vec<(String, Box<dyn Filter<Item = f64>>)> = (0..fft_size / 2).map(|i| {
            let freq_name = match fft_freqs.get(i) {
                Some(freq) => concat_string!(dbg, export_point_name, ".", freq),
                None => panic!("{}.out | Freq index {} out of the fft_size {}", dbg, i, fft_size),
            };
            (freq_name, filter(Some(PointConfFilter { threshold: threshold, factor: None })))
        }).collect();
        // reference FFT's, calculated locally
        let mut ref_ffts: Vec< Vec<f64> > = vec![];
        // FFT's, calculated by FnVaFft
        let mut received_ffts: Vec< Vec<f64> > = vec![];
        // sequences used for plotting charts
        let mut plot_values: Vec<(f64, f64)> = vec![];
        let mut plot_ffts: Vec<Vec<(f64, f64)>> = vec![];
        for step in 0..fft_size * ffts {
            let t = FftBuf::time(sampl_freq, step);
            let value = target_freqs.iter().fold(0.0, |val, (freq, amp)| {
                val + amp * (2. * PI *  freq * t).sin()
            });
            plot_values.push((t, value));

            // FnVaFft process
            let time = Instant::now();
            fn_va_fft_input.borrow_mut().add(&value.to_point(tx_id, &format!("t: {}", t)));
            fn_va_fft.out();
            log::trace!("{dbg} | {}  freq: {}  FnVaFft Elapsed: {:?}", step, sampl_freq, time.elapsed());

            match fft_buf.add(value) {
                Some(buf) => {
                    // Pure FFT process
                    log::trace!("{dbg} | t: {:.4},  buf: {:?}", t, buf);
                    let time = Instant::now();
                    fft.process(buf);
                    log::debug!("{dbg} | freq: {}  Pure FFT Elapsed: {:?}", sampl_freq, time.elapsed());
                    // log::debug!("{dbg} | t: {:.4},  fft: {:?}", t, buf);
                    let mut fft_scalar: Vec<f64> = vec![];  //buf.iter().take(fft_size / 2).skip(1).map(|val| val.abs() * fft_amp_factor).collect();
                    for (index, val) in buf.iter().take(fft_size / 2).skip(1).enumerate() {
                        match fft_filters.get_mut(index) {
                            Some((_freq_name, filter)) => {
                                if let Some(filter_value) = filter.add(val.abs() * fft_amp_factor) {
                                    fft_scalar.push(filter_value);
                                }
                            }
                            None => panic!("{dbg} | fft_filters index {} out of size {}", index, fft_filters.len()),
                        }
                    }
                    log::trace!("{dbg} | t: {:.4},  fft_scalar: {:?}", t, fft_scalar.iter().map(|v| format!("{:.3}", v)).collect::<Vec<String>>());
                    ref_ffts.push(fft_scalar.clone());

                    // Receiving FnVaFft results
                    let time = Instant::now();
                    while receiver.received().len() < fft_scalar.len() {
                        thread::sleep(Duration::from_millis(3));
                    }
                    let received = receiver.drain(0..fft_scalar.len());
                    log::debug!("{dbg} | FnVaFft received in {:?}, \t received: {}", time.elapsed(), received.len());
                    // log::debug!("{dbg} | FnVaFft received: {:?}", received.iter().map(|v| format!("{:.3}", v.as_double().value)).collect::<Vec<String>>());
                    let mut va_fft_buf = vec![];
                    let mut plot_ffts_step = vec![];
                    for point in &received {
                        va_fft_buf.push(point.as_double().value);
                        let val = point.as_double().value;
                        let freq: f64 = point.name().parse().unwrap();
                        println!("\t| received  {:.3}, {:.4}", freq, val);
                        if val > 1.0 {
                            plot_ffts_step.push((freq, val + (received_ffts.len() as f64) * 10.0));
                        }
                    }
                    if !plot_ffts_step.is_empty() {
                        plot_ffts.push(plot_ffts_step);
                    }
                    if !va_fft_buf.is_empty() {
                        received_ffts.push(va_fft_buf.clone());
                    }

                    log::debug!("{dbg} |           target: {:?}", fft_scalar.iter().filter_map(|val| {
                        (*val > 1.0).then(|| format!("{:.3}", val))
                    }).collect::<Vec<String>>());
                    log::debug!("{dbg} | FnVaFft received: {:?}", received.iter().filter_map(|point| {
                        let val = point.as_double().value;
                        // let freq: f64 = v.name().parse().unwrap();
                        // println!("\t| received  {:.3}, {:?}", freq, val);
                        (val > 1.0).then(|| format!("{:.3}", val))
                    }).collect::<Vec<String>>());

                    // if let Err((result, target)) = compare_vecs(&va_fft_buf, &fft_scalar, Some(3))  {
                    //     panic!("{dbg} | FnVaFft({} sec) error \n result: {:?} \n target: {:?}", t, result, target);
                    //     // log::error!("FnVaFft({} sec) error \n result: {:?} \n target: {:?}", t, va_fft_buf, fft_scalar);
                    // }
                }
                None => {
                    log::trace!("{dbg} | t: {:.4}", t);
                },
            };
        }

        receiver.exit();
        services.exit();
        services.wait().unwrap();
        receiver.wait().unwrap();

        let path = "src/tests/unit/services/task/functions/va/absolute-filter";
        plot(format!("{path}/values-{:?}.png", sampl_freq), None, vec![plot_values], SeriesKind::Both).unwrap();
        plot(format!("{path}/ffts-{:?}.png", sampl_freq), None, plot_ffts, SeriesKind::Points).unwrap();

        log::debug!("{dbg} | ffts: \n\t target: {}, \n\t ref calculated: {}, \n\t  va calculated: {}", target_freqs.len(), ref_ffts.len(), received_ffts.len());
        
        log::debug!("{dbg} | target_freqs: {:?}", target_freqs);
        log::debug!("{dbg} | received_ffts: {:?}", received_ffts);

        let result = ref_ffts.len();
        let target = ffts;
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        let result = received_ffts.len();
        let target = target_ffts;
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
    // assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    test_duration.exit();
}
///
/// Returns Threshold (key filter)
fn filter(conf: Option<PointConfFilter>) -> Box<dyn Filter<Item = f64>> {
    match conf {
        Some(conf) => {
            Box::new(
                FilterThreshold::<f64>::new(None, conf.threshold, conf.factor.unwrap_or(0.0))
            )
        }
        None => Box::new(FilterEmpty::<f64>::new(None)),
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
/// - `decimals` - number of fraction digits to be compared (aproximate comparasion) 
fn compare_vecs(v1: &[f64], v2: &[f64], decimals: Option<usize>) -> Result<(), (String, String)> {
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
                let equals = match decimals {
                    Some(decimals) => value1.aprox_eq(*value2, decimals),
                    None => value1 == value2,
                };
                if equals {
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
