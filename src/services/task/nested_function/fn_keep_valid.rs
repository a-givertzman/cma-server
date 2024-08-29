use log::trace;
use sal_sync::services::entity::point::point::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    core_::types::fn_in_out_ref::FnInOutRef,
    services::task::nested_function::{
        fn_::{FnIn, FnInOut, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Returns last valid keeped value
/// - if nothing keeped returns None
#[derive(Debug)]
pub struct FnKeepValid {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
    state: Option<Point>
}
//
// 
impl FnKeepValid {
    ///
    /// Creates new instance of the FnKeepValid
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnInOutRef) -> Self {
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
impl FnIn for FnKeepValid {}
//
// 
impl FnOut for FnKeepValid { 
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
        let input = self.input.borrow_mut().out();
        trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                trace!("{}.out | value: {:?}", self.id, &input);
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
//
// 
impl FnInOut for FnKeepValid {}
///
/// Global static counter of FnKeepValid instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
