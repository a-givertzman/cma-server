use chrono::Utc;
use derivative::Derivative;
use rustfft::{num_complex::{Complex, ComplexFloat}, Fft, FftPlanner};
use sal_sync::services::{
    entity::{cot::Cot, point::{point::Point, point_config::PointConfig, point_hlr::PointHlr, point_tx_id::PointTxId}, status::status::Status}, 
    types::bool::Bool,
};
use std::sync::{atomic::{AtomicUsize, Ordering}, mpsc::Sender, Arc};
use crate::{
    conf::fn_::fn_config::FnConfig, core_::types::fn_in_out_ref::FnInOutRef, services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    }
};
use super::unit_circle::UnitCircle;
///
/// Global static counter of FnVaFft instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Function | FFT analysis
/// - `enable` - enables the activity
/// - `len` - length of the sequence processing at a time, also defining the length of array returned from the FFT
/// - `input` - Point's caming from Vibro-analitics micro-controller
/// - Returns value from `enable` input
/// 
/// Example
/// 
/// ```yaml
/// fn VaFft:
///     enable: const bool true         # optional, default true
///     send-to: /AppTest/MultiQueue.in-queue
///     conf point Point.Name:          # full name will be: /App/Task/Point.Name
///         type: 'Bool'
///     input: point string /AppTest/Exit
///     freq: 300000                    # Sampling freq
///     window: 512
///     len: 30000                      # Length of the                         
/// ```
#[derive(Derivative)]
#[derivative(Debug)]
pub struct FnVaFft {
    tx_id: usize,
    id: String,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    point_conf: Option<PointConfig>,
    len: usize,
    input: FnInOutRef,
    #[derivative(Debug="ignore")]
    fft: Arc<dyn Fft<f64>>,
    #[derivative(Debug="ignore")]
    fft_buf: Vec<Complex<f64>>,
    unit_circle: UnitCircle,
    tx_send: Option<Sender<Point>>,
}
//
//
impl FnVaFft {
    ///
    /// Creates new instance of the FnVaFft
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, input: FnInOutRef, conf: FnConfig, point_conf: Option<PointConfig>, send: Option<Sender<Point>>) -> Self {
        let self_id = format!("{}/FnVaFft{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let len = match conf.param("len") {
            Ok(len) => len.as_param().conf.as_u64().unwrap() as usize,
            Err(_) => panic!("{}.new | Parameter 'len' - missed", self_id),
        };
        let freq = match conf.param("freq") {
            Ok(freq) => freq.as_param().conf.as_u64().unwrap() as usize,
            Err(_) => panic!("{}.new | Parameter 'freq' - missed", self_id),
        };
        Self {
            tx_id: PointTxId::from_str(&self_id),
            id: self_id,
            kind: FnKind::Fn,
            enable,
            point_conf,
            len,
            input,
            fft: FftPlanner::new().plan_fft_forward(len),
            fft_buf: vec![],
            unit_circle: UnitCircle::new(freq),
            tx_send: send,
        }
    }
    ///
    /// Sending FFT results as Point's to the external service if 'send-to' specified
    fn send(&self, point: Point) {
        if let Some(tx_send) = &self.tx_send {
            match tx_send.send(point) {
                Ok(_) => {
                    // log::trace!("{}.out | Point sent: {:#?}", self.id, point);
                }
                Err(err) => {
                    // log::error!("{}.out | Send error: {:#?}\n\t point: {:#?}", self.id, err, point);
                    log::error!("{}.out | Send error: {:#?}", self.id, err);
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
            Point::Double(point) => point.to_double().value,
            _ => {
                log::error!("{}.out | Invalid input type '{:?}' Point: {}", self.id, input.type_(), input.name());
                0.0
            }
        };
        if self.fft_buf.len() < self.len {
            let (_, _, unit_complex) = self.unit_circle.next();
            self.fft_buf.push(
                Complex {
                    re: value * unit_complex.re,
                    im: value * unit_complex.im,
                },
            )
        }
        if self.fft_buf.len() >= self.len {
            if enable {
                self.fft.process(self.fft_buf.as_mut_slice());
                for value in &self.fft_buf {
                    let point = Point::Double(PointHlr::new(
                        self.tx_id,
                        &self.id,
                        value.abs(),
                        input.status(),
                        input.cot(),
                        input.timestamp(),
                    ));
                    log::trace!("{}.out | point: {:#?}", self.id, value);
                    self.send(point);
                }
            }
            self.fft_buf = vec![];
        }
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
        self.unit_circle.reset();
        self.fft_buf = vec![];
    }
}
//
//
impl FnInOut for FnVaFft {}
