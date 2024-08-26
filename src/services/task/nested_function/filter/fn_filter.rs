use std::sync::atomic::{AtomicUsize, Ordering};
use log::trace;
use crate::{
    core_::{point::point::Point, types::fn_in_out_ref::FnInOutRef},
    services::task::nested_function::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult},
};
///
/// Function | Returns filtered input or default value
/// - [pass] if true (or [pass] > 0) - current input value will returns from now on
/// - if default is not specified and filtered value not passed yet - default value of the input type returns
#[derive(Debug)]
pub struct FnFilter {
    id: String,
    // tx_id: usize,
    kind: FnKind,
    default: Option<FnInOutRef>,
    input: FnInOutRef,
    pass: FnInOutRef,
    state: Option<Point>,
}
//
//
impl FnFilter {
    ///
    /// Creates new instance of the FnFilter
    /// - id - just for proper debugging
    /// - input - incoming points
    pub fn new(parent: impl Into<String>, default: Option<FnInOutRef>, input: FnInOutRef, pass: FnInOutRef) -> Self {
        let self_id = format!("{}/FnFilter{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        Self {
            id: self_id.clone(),
            // tx_id: PointTxId::from_str(&self_id),
            kind: FnKind::Fn,
            default,
            input,
            pass,
            state: None,
        }
    }
}
//
//
impl FnIn for FnFilter {}
//
//
impl FnOut for FnFilter {
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
        inputs.append(&mut self.pass.borrow().inputs());
        inputs.append(&mut self.input.borrow().inputs());
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<Point, String> {
        let pass_point = self.pass.borrow_mut().out();
        trace!("{}.out | pass: {:?}", self.id, pass_point);
        let pass = match pass_point {
            FnResult::Ok(enable) => enable.to_bool().as_bool().value.0,
            FnResult::None => return FnResult::None,
            FnResult::Err(err) => return FnResult::Err(err),
        };
        if pass {
            let input = self.input.borrow_mut().out();
            trace!("{}.out | input: {:?}", self.id, input);
            match input {
                FnResult::Ok(input) => {
                    trace!("{}.out | Passed: {:?}", self.id, input);
                    self.state = Some(input.clone());
                    FnResult::Ok(input)
                }
                FnResult::None => FnResult::None,
                FnResult::Err(err) => FnResult::Err(err),
            }
        } else {
            FnResult::None
        }        
    }
    //
    fn reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
        self.pass.borrow_mut().reset();
    }
}
//
//
impl FnInOut for FnFilter {}
///
/// Global static counter of FnFilter instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
