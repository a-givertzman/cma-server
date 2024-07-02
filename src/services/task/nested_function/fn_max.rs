use std::sync::atomic::{AtomicUsize, Ordering};
use log::trace;
use crate::core_::{
    point::point_type::PointType,
    types::fn_in_out_ref::FnInOutRef,
};
use super::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult};
///
/// Returns an max value (in Double) of the input
#[derive(Debug)]
pub struct FnMax {
    id: String,
    kind: FnKind,
    enable: Option<FnInOutRef>,
    input: FnInOutRef,
    max: Option<PointType>,
}
//
// 
impl FnMax {
    ///
    /// Creates new instance of the FnMax
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, enable: Option<FnInOutRef>, input: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnMax{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            enable,
            input,
            max: None,
        }
    }
}
//
// 
impl FnIn for FnMax {}
//
// 
impl FnOut for FnMax {
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
        inputs.append(&mut self.input.borrow().inputs());
        inputs
    }
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
        // trace!("{}.out | enable: {:?}", self.id, enable);
        if enable {
            let input = self.input.borrow_mut().out();
            // trace!("{}.out | input: {:?}", self.id, input);
            match input {
                FnResult::Ok(input) => {
                    trace!("{}.out | max: {:?}", self.id, self.max);
                    // let max = self.max.clone().unwrap_or_else(|| {
                    //     self.max = Some(input.clone());
                    //     input.clone()
                    // });
                    let max = self.max.get_or_insert(input.clone());
                    match &input {
                        PointType::Bool(input_val) => {
                            let max_val = max.try_as_bool().unwrap_or_else(|_| panic!("{}.out | Incompitable types: max: '{:?}', input: '{:?}'", self.id, max.type_(), input.type_()));
                            if input_val.value.0 > max_val.value.0 {
                                *max = input;
                            }
                        }
                        PointType::Int(input_val) => {
                            let max_val = max.try_as_int().unwrap_or_else(|_| panic!("{}.out | Incompitable types: max: '{:?}', input: '{:?}'", self.id, max.type_(), input.type_()));
                            if input_val.value > max_val.value {
                                *max = input;
                            }
                        }
                        PointType::Real(input_val) => {
                            let max_val = max.try_as_real().unwrap_or_else(|_| panic!("{}.out | Incompitable types: max: '{:?}', input: '{:?}'", self.id, max.type_(), input.type_()));
                            if input_val.value > max_val.value {
                                *max = input;
                            }
                        }
                        PointType::Double(input_val) => {
                            let max_val = max.try_as_double().unwrap_or_else(|_| panic!("{}.out | Incompitable types: max: '{:?}', input: '{:?}'", self.id, max.type_(), input.type_()));
                            if input_val.value > max_val.value {
                                *max = input;
                            }
                        }
                        PointType::String(_) => panic!("{}.out | Input of type 'String' is not suppoted in: '{}'", self.id, input.name()),
                    }
                }
                FnResult::None => {}
                FnResult::Err(err) => return FnResult::Err(err),
            };
            self.max.clone().map_or(FnResult::None, |max| FnResult::Ok(max))
        } else {
            FnResult::None
        }
    }
    //
    fn reset(&mut self) {
        self.max = None;
        if let Some(enable) = &self.enable {
            enable.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnMax {}
///
/// Global static counter of FnMax instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
