use log::debug;
use concat_string::concat_string;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    core_::{point::point_type::PointType, types::fn_in_out_ref::FnInOutRef},
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind,
    },
};
use super::fn_result::FnResult;
///
/// Function | Just doing debug of values coming from inputs
/// - Returns value from the last input
#[derive(Debug)]
pub struct FnDebug {
    id: String,
    kind: FnKind,
    inputs: Vec<FnInOutRef>,
}
//
// 
impl FnDebug {
    ///
    /// Creates new instance of the FnDebug
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnInOutRef>) -> Self {
        Self { 
            id: format!("{}/FnDebug{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            inputs,
        }
    }    
}
//
// 
impl FnIn for FnDebug {}
//
// 
impl FnOut for FnDebug { 
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<PointType, String> {
        let mut inputs = self.inputs.iter();
        let mut value: PointType;
        // let first = .cloned();
        match inputs.next() {
            Some(first) => {
                let first = first.borrow_mut().out();
                match first {
                    FnResult::Ok(input) => {
                        value = input.to_owned();
                        debug!("{}.out | value: {:#?}", self.id, value);
                        while let Some(input) = inputs.next().cloned() {
                            let input = input.borrow_mut().out();
                            match input {
                                FnResult::Ok(input) => {
                                    value = input.clone();
                                    debug!("{}.out | value: {:#?}", self.id, value);
                                }
                                FnResult::None => return FnResult::None,
                                FnResult::Err(err) => return FnResult::Err(err),
                            }
                        }        
                    }
                    FnResult::None => return FnResult::None,
                    FnResult::Err(err) => return FnResult::Err(err),
                }
            }
            None => return FnResult::Err(concat_string!(self.id, ".out | No inputs found")),
        }
        FnResult::Ok(value)
    }
    //
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
//
// 
impl FnInOut for FnDebug {}
///
/// Global static counter of FnDebug instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
