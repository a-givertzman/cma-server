use chrono::Utc;
use concat_string::concat_string;
use derivative::Derivative;
use egui::ahash::HashMapExt;
use indexmap::IndexMap;
use rustfft::{num_complex::ComplexFloat, Fft, FftPlanner};
use sal_sync::{collections::FxHashMap, services::{
    entity::{
        Cot, Name,
        Point, PointConf, PointConfFilter, PointConfType, PointHlr, PointTxId,
        Status, ToPoint,
    }, task::functions::{FnConfKind, FnConfOptions, FnConfPointType, FnConfig}, types::Bool, LinkName, Services
}, sync::channel::Sender};
use std::{cell::RefCell, rc::Rc, str::FromStr, sync::{atomic::{AtomicUsize, Ordering}, Arc}};
use crate::{
    domain::{filter::{filter::{Filter, FilterEmpty}, filter_threshold::FilterThreshold}, format::FormatPoint, FnInOutRef},
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut}, fn_const::FnConst, fn_input::FnInput, fn_kind::FnKind, fn_result::FnResult, io::fn_retain::FnRetain
    }
};
use super::fft_buff::FftBuf;
///
/// ### Function | FFT analysis
/// - `enable` - enables the activity
/// - `len` - length of the FFT sequence processing at a time, also defining number of frequencies returned from the FFT
/// - `input` - Point's caming from Vibro-analitics micro-controller
/// - `point_conf` - config of the sent Point's, if not specified - default '/parent/Fft.freq' type 'Real' will be sent
/// - Returns value from `enable` input
/// 
/// **Description**
/// 
///   Used for convertion sequence of measured (with sampl freq) samples
/// into the squence of amplitudes of frequences same length / 2
/// 
///   This means if in the sampling period we have `N` samples,
/// then after fft we will have `N/2` numbers, representing amplitude
/// of each frequence in the range of `0..N/2`
/// 
///   If filtering used, then result of each fft processing will
/// returns only frequences, wich amplitudes differs to prevouse result
/// 
/// **Database table example**
/// ```ignore
///   id  | timestamp | value
///   --  | --        | --
///   int | timestamp | any
/// ```
/// 
/// **Timestamp example:**
///   - `1985-04-12 23:20:50.52`
///   - `2025-06-20 13:51:27.086998288 UTC`
/// 
/// **Config example**
/// 
/// ```yaml
/// fn VaFft:
///     enable: const bool true                 # optional, default true
///     send-to: /AppTest/MultiQueue.in-queue   # Send `Point` to the specified service.queue
///     format: UPDATE public.fft SET (id, timestamp, value) = ({{in.name}}, {{in.timestamp}}, {{in.value}});       # Convert Point to formated string, into SQL for example
///     filter: 
///     conf point Fft:                     # Conf for Point's to be exported (by sent-to) full name will be: '/App/Task/Fft.freq', use '/' to have 'freq' only (`freq` will replaced by it's index if sampling freq is not specified)
///         type: 'Real'                    # Double / Real / Int
///     input: point real /App/Sensor1      # Input signal of measured samples
///     sampl-freq: 300000                  # Sampling freq, optionally can be specified to have a name of point contains a freq instead of index in the sufix
///     len: 30000                          # Length of the FFT sequence processing at a time, also defining number of frequencies returned from the FFT
///     window: 512                         # Not used for now, reserved for future
///     filter:                             # Filter conf, for each frequency to be filtered on fly
///         threshold: 0.5                  #   absolute threshold delta
///         factor: 1.5                     #   multiplier for absolute threshold delta - in this case the delta will be accumulated
/// ```
/// 
/// References
/// [Restore FFT frequences](https://stackoverflow.com/a/4371627/17986285)
#[derive(Derivative)]
#[derivative(Debug)]
pub struct FnVaFft {
    txid: usize,
    id: String,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    /// Point config for exported Point's
    point_conf: PointConf,
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
    sampl_freq: Option<usize>,
    /// Retain for store    (name,   input,      out)
    retain: FxHashMap<String, (FnInOutRef, FnRetain)>,
    /// FFT Freq (name, filter)
    filters: Vec<(String, Box<dyn Filter<Item = f64>>)>,
    tx_send: Option<Sender<Point>>,
    format: Option<FormatPoint>,
    format_key: String,
    // first: Option<()>,
}
//
//
impl FnVaFft {
    ///
    /// Creates new instance of the FnVaFft
    #[allow(unused)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, input: FnInOutRef, conf: FnConfig, services: Arc<Services>) -> Self {
        let parent = parent.into();
        let name = Name::new(&parent, format!("FnVaFft-{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let dbg = name.join();      //format!("{}/FnVaFft{}", parent, COUNT.fetch_add(1, Ordering::Relaxed));
        let txid = PointTxId::from_str(&name.join());
        let fft_size = match conf.param("len") {
            Some(len) => len.as_param().conf.as_u64().unwrap() as usize,
            None => panic!("{}.new | Parameter 'len' - missed", dbg),
        };
        log::debug!("{}.new | fft_len: {:?}", dbg, fft_size);
        let sampl_freq = match conf.param("sampl-freq") {
            Some(freq) => Some(freq.as_param().conf.as_u64().unwrap() as usize),
            None => {
                log::info!("{}.new | Parameter 'freq' - missed, index of corresponding freq will used for naming", dbg);
                None
            }
        };
        log::debug!("{}.new | sampl_freq: {:?}", dbg, sampl_freq);
        let point_conf = Self::parse_point_conf(parent, &dbg, &conf);
        log::debug!("{}.new | point_conf: {:#?}", dbg, point_conf);
        let threshold_conf = Self::parse_threshold_conf(&dbg, &conf);
        log::debug!("{}.new | threshold: {:#?}", dbg, threshold_conf);
        let send_to = Self::parse_send_to(&dbg, &conf, &services);
        let (format, format_key) = Self::parse_format(&dbg, &conf);
        if format.is_some() { log::debug!("{}.new | format_key: {:#?}", dbg, format_key); }
        let fft_buf = FftBuf::new(fft_size);
        let fft_freqs: Vec<String> = match sampl_freq {
            Some(sampl_freq) => (0..fft_size / 2).map(|i| format!("{:?}", fft_buf.freq_of(sampl_freq, i)) ).collect(),
            None => (0..fft_size / 2).map(|i| format!("{i}") ).collect(),
        };
        let mut retain = FxHashMap::new();
        let filters = (0..fft_size / 2).map(|i| {
            let freq_name = match fft_freqs.get(i) {
                Some(freq) => {
                    match &point_conf.name.split('/').last() {
                        Some(name) => {
                            if name.is_empty() {
                                freq.to_owned()
                            } else {
                                concat_string!(name, "-", freq)
                            }
                        }
                        None => freq.to_owned()
                    }
                }
                None => panic!("{}.out | Freq index {} out of the fft_size {}", dbg, i, fft_size),
            };
            let retain_input = Self::retain_input(&dbg, txid, &freq_name);
            let mut fn_retain_store = FnRetain::new(
                &name,
                "assets/testing/retain/",
                enable.clone(),
                false,
                &freq_name,
                None,
                Some(retain_input.clone()),
            );
            let mut fn_retain_load = FnRetain::new(
                &name,
                "assets/testing/retain/",
                enable.clone(),
                false,
                &freq_name,
                Some(Rc::new(RefCell::new(Box::new(
                    FnConst::new(&name.join(), 0.0.to_point(txid, &name.join())),
                )))),
                None,
            );
            let retained = match fn_retain_load.out() {
                FnResult::Ok(val) => Some(val.as_double().value),
                FnResult::None => None,
                FnResult::Err(err) => {
                    log::warn!("{dbg}.new | Initial | {freq_name}: error: {:?}", err);
                    None
                }
            };
            log::debug!("{dbg}.new | Initial | {freq_name}: {:?}", retained);
            retain.insert(freq_name.clone(), (retain_input, fn_retain_store));
            (freq_name, Self::build_filter(threshold_conf.clone(), retained))
        }).collect();
        Self {
            txid,
            id: dbg,
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
            retain,
            filters,
            tx_send: send_to,
            format,
            format_key,
        }
    }
    ///
    /// Rturns the input for retain
    fn retain_input(parent: impl Into<String>, txid: usize, freq_name: &str) -> FnInOutRef {
        Rc::new(RefCell::new(Box::new(
            FnInput::new(
                parent,
                txid,
                &mut FnConfig {
                    name: freq_name.to_string(),
                    inputs: IndexMap::new(),
                    type_: FnConfPointType::Double,
                    options: FnConfOptions::default(),
                },
            )
        )))
    }
    ///
    /// Returns Threshold (key filter)
    fn build_filter(conf: Option<PointConfFilter>, initial: Option<f64>) -> Box<dyn Filter<Item = f64>> {
        match conf {
            Some(conf) => {
                Box::new(
                    FilterThreshold::<1, f64>::new(initial, conf.threshold, conf.factor.unwrap_or(0.0))
                )
            }
            None => Box::new(FilterEmpty::<1, f64>::new(None)),
        }
    }
    ///
    /// Returns Conf for Point's to be exported (by send-to) full name will be: /App/Task/Fft.freq
    fn parse_point_conf(parent: impl Into<String>, self_id: &str, conf: &FnConfig) -> PointConf {
        match conf.clone().input_conf("conf") {
            Ok(conf) => match conf {
                FnConfKind::PointConf(conf) => match conf.conf.type_ {
                    PointConfType::Int | PointConfType::Real | PointConfType::Double => conf.conf.clone(),
                    _ => panic!("{}.new | Invalid Point type: '{:?}' in {:#?}", self_id, conf.conf.type_, conf.conf),
                }
                _ => panic!("{}.new | Invalid Point config in: {:?}", self_id, conf.name()),
            }
            Err(_) => PointConf::from_yaml(&Name::new(parent, ""), &serde_yaml::from_str(r#"
                conf point FFT:
                    type: 'Real'
            "#).unwrap()),
        }
    }
    ///
    /// Returns Threshold config
    fn parse_threshold_conf(self_id: &str, conf: &FnConfig) -> Option<PointConfFilter> {
        match conf.param("filter") {
            Some(threshold) => match threshold {
                FnConfKind::Param(threshold) => match serde_yaml::from_value(threshold.conf.clone()) {
                    Ok(threshold) => {
                        let threshold: PointConfFilter = threshold;
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
            None => {
                log::warn!("{}.new | Threshold filter config missed in: {:?}", self_id, conf);
                None
            },
        }
    }
    ///
    /// Returns send_to
    fn parse_send_to(self_id: &str, conf: &FnConfig, services: &Arc<Services>) -> Option<Sender<Point>> {
        match conf.param("send-to") {
            Some(send_to) => {
                match send_to {
                    FnConfKind::Param(send_to) => {
                        let send_to = LinkName::from_str(send_to.conf.as_str().unwrap()).unwrap();
                        log::debug!("{}.new | send-to: {:?}", self_id, send_to.name());
                        services.get_link(&send_to).map_or(None, |send| Some(send))
                    }
                    _ => {
                        log::warn!("{}.new | Parameter 'send-to' - invalid type (string expected): {:#?}", self_id, send_to);
                        None
                    }
                }
            }
            None => {
                log::warn!("{}.new | Parameter 'send-to' - missed in {:#?}", self_id, conf);
                None
            },
        }
    }
    ///
    /// Returns format config
    fn parse_format(dbg: &str, conf: &FnConfig) -> (Option<FormatPoint>, String) {
        match conf.param("format") {
            Some(conf) => {
                let conf = conf.as_param().conf.as_str().unwrap().to_owned();
                log::debug!("{dbg}.new | format: {conf}");
                let format = FormatPoint::new(&conf);
                let format_key = format.names().into_iter().enumerate().fold(String::new(), |prev, (i, (_, (name, _)))| {
                    if (i > 0) & (prev != name) {
                        panic!("{dbg}.new | format '{conf}' has diferent inputs: '{prev}' and '{name}', but must have single");
                    }
                    name
                });
                (Some(format), format_key)
            }
            None => (None, String::new())
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
                                filter.add(amplitude.abs() * self.amp_factor);
                                if let Some(value) = filter.pop() {
                                    // let amplitude = amplitude.abs() * self.amp_factor;
                                    // log::trace!("{}.out | amplitude: {:#?}", self.id, amplitude);
                                    let point = Point::Double(PointHlr::new(
                                        self.txid,
                                        freq_name,
                                        value,
                                        input.status(),
                                        input.cot(),
                                        input.timestamp(),
                                    ));
                                    if let Some((retain_input, retain)) = self.retain.get_mut(freq_name) {
                                        retain_input.borrow_mut().add(&point);
                                        retain.out();
                                    }
                                    let point = match &mut self.format {
                                        Some(format) => {
                                            // log::debug!("{}.out | fft.process format.names: {:#?}", self.id, format.names());
                                            for (key, _) in format.names() {
                                                format.insert(&key, point.clone());
                                            }
                                            Point::String(PointHlr::new(
                                                self.txid,
                                                freq_name,
                                                format.out(),
                                                input.status(),
                                                input.cot(),
                                                input.timestamp(),
                                            ))
                                        }
                                        None => {
                                            Point::Double(PointHlr::new(
                                                self.txid,
                                                freq_name,
                                                value,
                                                input.status(),
                                                input.cot(),
                                                input.timestamp(),
                                            ))
                                        },
                                    };
                                    log::trace!("{}.out | point: {:#?}", self.id, point);
                                    Self::send(&self.id, &self.tx_send, point);
                                }
                            }
                            None => log::error!("{}.out | Fft filter index {} out of size {}", self.id, index, self.filters.len()),
                        }
                    }
                }
                // self.first = None;
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
                self.txid,
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
///
/// Global static counter of FnVaFft instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
