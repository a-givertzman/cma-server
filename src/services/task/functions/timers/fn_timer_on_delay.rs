use sal_core::error::Error;
use sal_sync::services::{conf::ConfDuration, entity::{Cot, Point, PointHlr}, types::Bool};
use std::{sync::atomic::{AtomicUsize, Ordering}, time::{Duration, Instant}};
use crate::{
    domain::FnOutRef,
    services::task::{FnOut, FnKind, FnResult},
};
///
/// Function | Retirns TRUE only after the Input has remained TRUE for the specified duration.
#[derive(Debug)]
pub struct FnTimerOnDelay {
    id: String,
    kind: FnKind,
    /// Input, activates behavior, if `false`, always returns Input value immediately, default `true`
    enable: Option<FnOutRef>,
    /// Input, delay
    delay: Duration,
    /// Input, value
    input: FnOutRef,
    time: Option<Instant>,
}
//
// 
impl FnTimerOnDelay {
    ///
    /// `enable` - Activates behavior, if false, always returns Input value immediately, default `true`
    /// `delay` = Time being waited for return `true` since input is `true`
    /// `input` - Input value, number or bool
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnOutRef>, delay: ConfDuration, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnTimerOnDelay{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            enable,
            delay: delay.to_duration(),
            input,
            time: None,
        }
    }
}
//
//
impl FnOut for FnTimerOnDelay {
    //
    fn id(&self) -> String {
        self.id.clone()
    }
    //
    fn kind(&self) -> FnKind {
        self.kind
    }
    //
    fn inputs(&self) -> Vec<String> {
        let mut inputs = vec![];
        if let Some(enable) = &self.enable {
            inputs.append(&mut enable.borrow().inputs());
        }
        inputs.append(& mut self.input.borrow().inputs());
        inputs
    }
    ///
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let enable = match &mut self.enable {
            Some(en) => match en.borrow_mut().out() {
                FnResult::Ok(en) => en.to_bool().as_bool().value.0,
                FnResult::None => return FnResult::None,
                FnResult::Err(err) => return FnResult::Err(err),
            }
            None => true,
        };
        let input = self.input.borrow_mut().out();
        match input {
            FnResult::Ok(input) => {
                // trace!("{}.out | input: {:?}", self.id, self.input.print());
                let val = match &input {
                    Point::Bool(p) => p.value.0,
                    Point::Int(p) => p.value > 0,
                    Point::Real(p) => p.value > 0.0,
                    Point::Double(p) => p.value > 0.0,
                    Point::String(_) => return FnResult::Err(Error::new(&self.id, "out").err("Input of type 'String' - isn't supported").to_string()),
                    Point::Bytes(_) => return FnResult::Err(Error::new(&self.id, "out").err("Input of type 'Bytes' - isn't supported").to_string()),
                };
                let out = if enable {
                    match val {
                        true => {
                            let elapsed = match &self.time {
                                Some(time) => time.elapsed(),
                                None => {
                                    let t = Instant::now();
                                    let elapsed = t.elapsed();
                                    self.time = Some(t);
                                    elapsed
                                }
                            };
                            elapsed > self.delay
                        }
                        false => {
                            self.time = None;
                            false
                        }
                    }
                } else {
                    self.time = None;
                    val
                };
                log::trace!("{}.out | out: {:?}", self.id, out);
                FnResult::Ok(Point::Bool(PointHlr::new(
                    input.txid(),
                    &format!("{}.out", self.id),
                    Bool(out),
                    input.status(),
                    Cot::Inf,
                    input.timestamp(),
                )))
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    //
    //
    fn reset(&mut self) {
        self.time = None;
        if let Some(enable) = &self.enable {
            enable.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
