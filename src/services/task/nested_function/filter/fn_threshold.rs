use std::sync::atomic::{AtomicUsize, Ordering};
use log::{debug, trace};
use crate::{
    conf::point_config::point_config_type::PointConfigType, core_::{
        point::{point::Point, point_type::PointType}, 
        types::fn_in_out_ref::FnInOutRef,
    }, services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    }
};
///
/// Function | Returns filtered input:
/// - if [enable] is specified and true, or not specified (default true)
/// - if [factor] is not specified:
///     - new input value returned if |prev - [input]| > [threshold]
/// - if [factor] is specified:
///     - each cycle: delta = |prev - [input]| * factor
///     - new input value returned if delta >= [threshold]
/// 
/// Example
/// 
/// ```yaml
/// fn Threshold:
///     enable: const bool true         # optional, default true
///     threshold: const real 0.5   # absolute threshold if [factor] is not specified
///     factor: 0.1                 # optional, use for integral threshold
///     input: point real '/App/Service/Point.Name'
/// ```
#[derive(Debug)]
pub struct FnThreshold {
    id: String,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    threshold: FnInOutRef,
    factor: Option<FnInOutRef>,
    input: FnInOutRef,
    value: Option<PointType>,
    delta: Point<f64>,
}
//
// 
impl FnThreshold {
    ///
    /// Creates new instance of the FnThreshold
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, threshold: FnInOutRef, factor: Option<FnInOutRef>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnThreshold{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            enable,
            threshold,
            factor,
            input,
            value: None,
            delta: Point::new_double(0, "", 0.0),
        }
    }    
}
//
// 
impl FnIn for FnThreshold {}
//
// 
impl FnOut for FnThreshold { 
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
        let mut inputs = vec![];
        if let Some(enable) = &self.enable {
            inputs.append(&mut enable.borrow().inputs());
        }
        inputs.append(&mut self.threshold.borrow().inputs());
        if let Some(factor) = &self.factor {
            inputs.append(&mut factor.borrow().inputs());
        }
        inputs.append(&mut self.input.borrow().inputs());
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<PointType, String> {
        let enable = match &self.enable {
            Some(enable) => match enable.borrow_mut().out() {
                FnResult::Ok(enable) => enable.to_bool().as_bool().value.0,
                FnResult::None => return FnResult::None,
                FnResult::Err(err) => return FnResult::Err(err),
            },
            None => true,
        };
        if enable {
            let threshold = self.threshold.borrow_mut().out();
            trace!("{}.out | threshold: {:?}", self.id, threshold);
            let threshold = match threshold {
                FnResult::Ok(threshold) => threshold.to_double().as_double(),
                FnResult::None => return FnResult::None,
                FnResult::Err(err) => return FnResult::Err(err),
            };
            let factor = match &self.factor {
                Some(factor) => {
                    let factor = factor.borrow_mut().out();
                    trace!("{}.out | factor: {:?}", self.id, factor);
                    match factor {
                        FnResult::Ok(factor) => Some(factor.to_double().as_double()),
                        FnResult::None => return FnResult::None,
                        FnResult::Err(err) => return FnResult::Err(err),
                    }
                }
                None => None,
            };
            let input = self.input.borrow_mut().out();
            trace!("{}.out | input: {:?}", self.id, input);
            match input {
                FnResult::Ok(input) => {
                    let input_type = input.type_();
                    let input = input.to_double().as_double();
                    match &mut self.value {
                        Some(value) => {
                            let delta = (input.clone() - value.to_double().as_double()).abs();
                            trace!("{}.out | Absolute delta: {}", self.id, delta.value);
                            if delta >= threshold {
                                *value = PointType::Double(input);
                                self.delta = Point::new_double(0, "", 0.0);
                            } else {
                                if let Some(factor) = factor {
                                    self.delta = self.delta.clone() + (delta * factor);
                                    debug!("{}.out | Integral delta: {}", self.id, self.delta.value);
                                    if self.delta >= threshold {
                                        self.value = Some(PointType::Double(input));
                                        self.delta = Point::new_double(0, "", 0.0);
                                    }
                                }
                            }
                        }
                        None => {
                            self.value = Some(PointType::Double(input));
                        }
                    }
                    let value = match &self.value {
                        Some(value) => match input_type {
                            PointConfigType::Int => value.to_int(),
                            PointConfigType::Real => value.to_real(),
                            PointConfigType::Double => value.to_double(),
                            _ => panic!("{}.out | Illegal type of input {:?}", self.id, input_type),
                        }
                        None => panic!("{}.out | Internal error - self.value is not initialised", self.id),
                    };
                    trace!("{}.out | value: {:?}", self.id, value);
                    FnResult::Ok(value)
                }
                FnResult::None => FnResult::None,
                FnResult::Err(err) => FnResult::Err(err),
            }
        } else {
            self.value = None;
            self.delta = Point::new_double(0, "", 0.0);
            FnResult::None
        }
    }
    //
    //
    fn reset(&mut self) {
        if let Some(enable) = &self.enable {
            enable.borrow_mut().reset();
        }
        self.threshold.borrow_mut().reset();
        if let Some(factor) = &self.factor {
            factor.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
        self.value = None;
        self.delta = Point::new_double(0, "", 0.0);
    }
}
//
// 
impl FnInOut for FnThreshold {}
///
/// Global static counter of FnThreshold instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
