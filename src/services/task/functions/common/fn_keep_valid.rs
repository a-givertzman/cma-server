use sal_sync::services::entity::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::functions::{
        FnOut, FnKind, FnResult,
    },
};
///
/// Function | Returns last valid keeped value
/// - if nothing keeped returns None
#[derive(Debug)]
pub struct FnKeepValid {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    state: Option<Point>
}
//
// 
impl FnKeepValid {
    ///
    /// Creates new instance of the FnKeepValid
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnKeepValid{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
            state: None,
        }
    }    
}
//
// 
impl FnOut for FnKeepValid { 
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
        self.input.borrow().inputs()
    }
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = self.input.borrow_mut().out();
        log::trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                log::trace!("{}.out | value: {:?}", self.id, &input);
                self.state = Some(input.clone());
                FnResult::Ok(input)
            }
            _ => match &self.state {
                Some(value) => FnResult::Ok(value.clone()),
                None => FnResult::None,
            }
        }
    }
    //
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnKeepValid instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
