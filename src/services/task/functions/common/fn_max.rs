use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use sal_sync::services::entity::Point;
use crate::domain::FnOutRef;
use crate::services::task::{FnOut, FnKind, FnResult};
///
/// Returns an max value (in Double) of the input
#[derive(Debug)]
pub struct FnMax {
    id: String,
    kind: FnKind,
    reset: Option<FnOutRef>,
    input: FnOutRef,
    max: Option<Point>,
}
//
// 
impl FnMax {
    ///
    /// Creates new instance of the FnMax
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnMax{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            reset,
            input,
            max: None,
        }
    }
}
//
// 
impl FnOut for FnMax {
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
        let mut inputs = self.input.borrow().inputs();
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let enable = match &self.reset {
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
                    log::trace!("{}.out | max: {:?}", self.id, self.max);
                    let max = self.max.get_or_insert(input.clone());
                    match &input {
                        Point::Bool(input_val) => {
                            let max_val = max.try_as_bool().unwrap_or_else(|_| panic!("{}.out | Incompitable types: max: '{:?}', input: '{:?}'", self.id, max.type_(), input.type_()));
                            if input_val.value.0 > max_val.value.0 {
                                *max = input;
                            }
                        }
                        Point::Int(input_val) => {
                            let max_val = max.try_as_int().unwrap_or_else(|_| panic!("{}.out | Incompitable types: max: '{:?}', input: '{:?}'", self.id, max.type_(), input.type_()));
                            if input_val.value > max_val.value {
                                *max = input;
                            }
                        }
                        Point::Real(input_val) => {
                            let max_val = max.try_as_real().unwrap_or_else(|_| panic!("{}.out | Incompitable types: max: '{:?}', input: '{:?}'", self.id, max.type_(), input.type_()));
                            if input_val.value > max_val.value {
                                *max = input;
                            }
                        }
                        Point::Double(input_val) => {
                            let max_val = max.try_as_double().unwrap_or_else(|_| panic!("{}.out | Incompitable types: max: '{:?}', input: '{:?}'", self.id, max.type_(), input.type_()));
                            if input_val.value > max_val.value {
                                *max = input;
                            }
                        }
                        Point::String(_) => return FnResult::Err(concat_string!(self.id, ".out | Input of type 'String' is not suppoted in: '", input.name(), "'")),
                        Point::Bytes(_) => return FnResult::Err(concat_string!(self.id, ".out | Input of type 'Bytes' is not suppoted in: '", input.name(), "'")),
                    }
                }
                FnResult::None => {}
                FnResult::Err(err) => return FnResult::Err(err),
            };
            self.max.clone().map_or(FnResult::None, |max| FnResult::Ok(max))
        } else {
            self.max = None;
            FnResult::None
        }
    }
    //
    fn reset(&mut self) {
        self.max = None;
        if let Some(reset) = &mut self.reset {
            reset.borrow_mut().reset();
        }
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnMax instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
