use chrono::Utc;
use concat_in_place::strcat;
use derivative::Derivative;
use rustfft::{num_complex::ComplexFloat, Fft, FftPlanner};
use sal_sync::services::{
    entity::{cot::Cot, name::Name, point::{point::Point, point_config::PointConfig, point_config_filters::PointConfigFilter, point_config_type::PointConfigType, point_hlr::PointHlr, point_tx_id::PointTxId}, status::status::Status}, service::link_name::LinkName, types::bool::Bool
};
use std::{str::FromStr, sync::{atomic::{AtomicUsize, Ordering}, mpsc::Sender, Arc, RwLock}};
use crate::{
    conf::fn_::{fn_conf_kind::FnConfKind, fn_config::FnConfig}, core_::{filter::{filter::{Filter, FilterEmpty}, filter_threshold::FilterThreshold}, types::fn_in_out_ref::FnInOutRef}, services::{safe_lock::rwlock::SafeLock, services::Services, task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    }}
};

use super::fft_buff::FftBuf;
///
/// Global static counter of FnVaFft instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Function | FFT analysis
/// - `enable` - enables the activity
/// - `len` - length of the FFT sequence processing at a time, also defining number of frequencies returned from the FFT
/// - `input` - Point's caming from Vibro-analitics micro-controller
/// - `point_conf` - config of the sent Point's, if not specified - default '/parent/Fft.freq' type 'Real' will be sent
/// - Returns value from `enable` input
/// 
/// Example
/// 
/// ```yaml
/// fn VaFft:
///     enable: const bool true         # optional, default true
///     send-to: /AppTest/MultiQueue.in-queue
///     conf point Fft:                 # Conf for Point's to be exported full name will be: /App/Task/Fft.freq
///         type: 'Real'                # Double / Real / Int
///     input: point string /AppTest/Exit
///     freq: 300000                    # Sampling freq
///     len: 30000                      # Length of the FFT sequence processing at a time, also defining number of frequencies returned from the FFT
///     window: 512
///     filter:                 # Filter conf, for each frequency to be filtered on fly
///         threshold: 0.5      #   absolute threshold delta
///         factor: 1.5         #   multiplier for absolute threshold delta - in this case the delta will be accumulated
/// ```
/// 
/// References
/// [Restore FFT frequences](https://stackoverflow.com/a/4371627/17986285)
#[derive(Derivative)]
#[derivative(Debug)]
pub struct FnVaFft {
    tx_id: usize,
    id: String,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    /// Point config for exported Point's
    point_conf: PointConfig,
    fft_size: usize,
    input: FnInOutRef,
    #[derivative(Debug="ignore")]
    fft: Arc<dyn Fft<f64>>,
    /// Vector of frequences correponding to the FFT.len ( Sampling `freq` / `len`) 
    fft_freqs: Vec<String>,
    /// The factor to restore the amplitude from FFT results
    amp_factor: f64,
    #[derivative(Debug="ignore")]
    fft_buf: FftBuf,
    sampl_freq: usize,
    /// FFT Freq (name, filter)
    filters: Vec<(String, Box<dyn Filter<Item = f64>>)>,
    tx_send: Option<Sender<Point>>,
}
//
//
impl FnVaFft {
    ///
    /// Creates new instance of the FnVaFft
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, input: FnInOutRef, conf: FnConfig, services: Arc<RwLock<Services>>) -> Self {
        let parent = parent.into();
        let self_id = format!("{}/FnVaFft{}", parent, COUNT.fetch_add(1, Ordering::Relaxed));
        let fft_size = match conf.param("len") {
            Ok(len) => len.as_param().conf.as_u64().unwrap() as usize,
            Err(_) => panic!("{}.new | Parameter 'len' - missed", self_id),
        };
        log::debug!("{}.new | fft_len: {:?}", self_id, fft_size);
        let sampl_freq = match conf.param("freq") {
            Ok(freq) => freq.as_param().conf.as_u64().unwrap() as usize,
            Err(_) => panic!("{}.new | Parameter 'freq' - missed", self_id),
        };
        log::debug!("{}.new | sampl_freq: {:?}", self_id, sampl_freq);
        let point_conf = Self::export_point_conf(parent, &self_id, &conf);
        log::debug!("{}.new | point_conf: {:#?}", self_id, point_conf);
        let threshold_conf = Self::threshold_conf(&self_id, &conf);
        log::debug!("{}.new | threshold: {:#?}", self_id, threshold_conf);
        let send_to = Self::send_to_conf(&self_id, &conf, &services);
        log::debug!("{}.new | send_to: {:#?}", self_id, threshold_conf);
        let fft_buf = FftBuf::new(fft_size, sampl_freq);
        let fft_freqs: Vec<String> = (0..fft_size / 2).map(|i| format!("{:?}", fft_buf.freq_of(i)) ).collect();
        let filters = (0..fft_size / 2).map(|i| {
            let freq_name = match fft_freqs.get(i) {
                Some(freq) => strcat!(&point_conf.name "." freq),
                None => panic!("{}.out | Freq index {} out of the fft_size {}", self_id, i, fft_size),
            };
            (freq_name, Self::filter(threshold_conf.clone()))
        }).collect();
        Self {
            tx_id: PointTxId::from_str(&self_id),
            id: self_id,
            kind: FnKind::Fn,
            enable,
            point_conf,
            fft_size,
            input,
            fft: FftPlanner::new().plan_fft_forward(fft_size),
            fft_freqs,
            amp_factor: fft_buf.amp_factor(),
            fft_buf,
            sampl_freq,
            filters,
            tx_send: send_to,
        }
    }
    ///
    /// Returns Threshold (key filter)
    fn filter(conf: Option<PointConfigFilter>) -> Box<dyn Filter<Item = f64>> {
        match conf {
            Some(conf) => {
                Box::new(
                    FilterThreshold::<2, f64>::new(None, conf.threshold, conf.factor.unwrap_or(0.0))
                )
            }
            None => Box::new(FilterEmpty::<2, f64>::new(None)),
        }
    }
    ///
    /// Returns Export point_conf
    fn export_point_conf(parent: impl Into<String>, self_id: &str, conf: &FnConfig) -> PointConfig {
        match conf.clone().input_conf("conf") {
            Ok(conf) => match conf {
                FnConfKind::PointConf(conf) => match conf.conf.type_ {
                    PointConfigType::Int | PointConfigType::Real | PointConfigType::Double => conf.conf.clone(),
                    _ => panic!("{}.new | Invalid Point type: '{:?}' in {:#?}", self_id, conf.conf.type_, conf.conf),
                }
                _ => panic!("{}.new | Invalid Point config in: {:?}", self_id, conf.name()),
            }
            Err(_) => PointConfig::from_yaml(&Name::new(parent, ""), &serde_yaml::from_str(r#"
                conf point Fft:
                    type: 'Real'
            "#).unwrap()),
        }
    }
    ///
    /// Returns Threshold config
    fn threshold_conf(self_id: &str, conf: &FnConfig) -> Option<PointConfigFilter> {
        match conf.param("threshold") {
            Ok(threshold) => match threshold {
                FnConfKind::Param(threshold) => match serde_yaml::from_value(threshold.conf.clone()) {
                    Ok(threshold) => {
                        let threshold: PointConfigFilter = threshold;
                        Some(threshold)
                    }
                    Err(err) => {
                        log::warn!("{}.new | Invalid Threshold filter config in: {:?}, \n\t error: {:#?}", self_id, conf, err);
                        None
                    }
                }
                _ => {
                    log::warn!("{}.new | Invalid Threshold filter config in: {:?}", self_id, conf);
                    None
                }
            }
            Err(_) => {
                log::warn!("{}.new | Threshold filter config missed in: {:?}", self_id, conf);
                None
            },
        }
    }
    ///
    /// Returns send_to
    fn send_to_conf(self_id: &str, conf: &FnConfig, services: &Arc<RwLock<Services>>) -> Option<Sender<Point>> {
        match conf.param("send-to") {
            Ok(send_to) => {
                let send_to = match send_to {
                    FnConfKind::Param(send_to) => LinkName::from_str(send_to.conf.as_str().unwrap()).unwrap(),
                    _ => panic!("{}.new | Parameter 'send-to' - invalid type (string expected): {:#?}", self_id, send_to),
                };
                log::debug!("{}.new | send-to: {:?}", self_id, send_to);
                let services_lock = services.rlock(self_id);
                services_lock.get_link(&send_to).map_or(None, |send| Some(send))
            }
            Err(_) => {
                log::warn!("{}.new | Parameter 'send-to' - missed in {:#?}", self_id, conf);
                None
            },
        }
    }
    ///
    /// Sending FFT results as Point's to the external service if 'send-to' specified
    fn send(self_id: &str, tx_send: &Option<Sender<Point>>, point: Point) {
        if let Some(tx_send) = tx_send {
            match tx_send.send(point) {
                Ok(_) => {
                    // log::debug!("{}.out | Point sent: {:#?}", self_id, point);
                }
                Err(err) => {
                    // log::error!("{}.out | Send error: {:#?}\n\t point: {:#?}", self.id, err, point);
                    log::error!("{}.out | Send error: {:#?}", self_id, err);
                }
            };
        }
    }
    ///
    /// FFT processing
    fn fft_process(&mut self, enable: bool, input: &Point) {
        let value = match input {
            Point::Int(point) => point.to_double().value,
            Point::Real(point) => point.to_double().value,
            Point::Double(point) => point.value,
            _ => {
                log::error!("{}.out | Invalid input type '{:?}' Point: {}", self.id, input.type_(), input.name());
                0.0
            }
        };
        // let t = self.fft_buf.time();
        // log::trace!("{}.out | fft.process next t: {},  angle: {}, buf.len: {} ...", self.id, t, "--", self.fft_buf.len());
        // log::debug!("{}.out | t: {},  complex: {}", self.id, t, complex);
        match self.fft_buf.add(value) {
            Some(buf) => {
                log::debug!("{}.out | fft.process buf {:?}...", self.id, buf.len());
                if enable {
                    self.fft.process(buf);
                    // First elebent of fft_buf have to be skeeped because it refers to DC
                    for (index, amplitude) in buf.iter().take(self.fft_size / 2).skip(1).enumerate() {
                        match self.filters.get_mut(index) {
                            Some((freq_name, filter)) => {
                                filter.add(value);
                                if filter.is_changed() {
                                    let amplitude = amplitude.abs() * self.amp_factor;
                                    log::trace!("{}.out | amplitude: {:#?}", self.id, amplitude);
                                    let point = Point::Double(PointHlr::new(
                                        self.tx_id,
                                        &freq_name,
                                        amplitude,
                                        input.status(),
                                        input.cot(),
                                        input.timestamp(),
                                    ));
                                    log::trace!("{}.out | point: {:#?}", self.id, point);
                                    Self::send(&self.id, &self.tx_send, point);
                                }
                            }
                            None => log::error!("{}.out | Fft filter index {} out of size {}", self.id, index, self.filters.len()),
                        }
                    }
                }
            }
            None => {},
        };
    }
}
//
//
impl FnIn for FnVaFft {}
//
//
impl FnOut for FnVaFft {
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> &FnKind {
        &self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        self.input.borrow().inputs()
    }
    //
    //
    fn out(&mut self) -> FnResult<Point, String> {
        let (enable, en_point) = match &self.enable {
            Some(enable) => {
                let enable = enable.borrow_mut().out();
                match enable {
                    FnResult::Ok(enable) => (enable.to_bool().as_bool().value.0, Some(enable)),
                    FnResult::None => return FnResult::None,
                    FnResult::Err(err) => return FnResult::Err(err),
                }
            }
            None => (true, None),
        };
        log::trace!("{}.out | enable: {:?}", self.id, enable);
        let input = self.input.borrow_mut().out();
        log::trace!("{}.out | input: {:#?}", self.id, input);
        match &input {
            FnResult::Ok(input) => {
                self.fft_process(enable, input);
            }
            FnResult::None => {},
            FnResult::Err(err) => {
                log::trace!("{}.out | Input error: {:#?}", self.id, err);
            },
        }
        FnResult::Ok(match en_point {
            Some(point) => point,
            None => Point::Bool(PointHlr::new(
                self.tx_id,
                &self.id,
                Bool(enable),
                Status::Ok,
                Cot::Inf,
                Utc::now(),
            )),
        })
    }
    //
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
        self.fft_buf.reset();
    }
}
//
//
impl FnInOut for FnVaFft {}
