use concat_string::concat_string;
use sal_sync::services::entity::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{
        FnOut, FnKind, FnResult
    },
};
///
/// Function | Just doing debug of values coming from inputs
/// - Returns value from the last input
#[derive(Debug)]
pub struct FnDebug {
    id: String,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
}
//
// 
impl FnDebug {
    ///
    /// Creates new instance of the FnDebug
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Self {
        Self { 
            id: format!("{}/FnDebug{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            inputs,
        }
    }    
}
//
// 
impl FnOut for FnDebug { 
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
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let mut inputs = self.inputs.iter();
        let mut value: Point;
        // let first = .cloned();
        match inputs.next() {
            Some(first) => {
                let first = first.borrow_mut().out();
                match first {
                    FnResult::Ok(input) => {
                        value = input.to_owned();
                        log::debug!("{}.out | value: {:#?}", self.id, value);
                        while let Some(input) = inputs.next().cloned() {
                            let input = input.borrow_mut().out();
                            match input {
                                FnResult::Ok(input) => {
                                    value = input.clone();
                                    log::debug!("{}.out | value: {:#?}", self.id, value);
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
///
/// Global static counter of FnDebug instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
