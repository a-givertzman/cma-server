use log::trace;
use sal_sync::services::entity::point::point::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    core_::types::fn_in_out_ref::FnInOutRef,
    services::task::nested_function::{
        fn_::{FnInOut, FnIn, FnOut},
        fn_kind::FnKind, fn_result::FnResult,
    },
};
///
/// Function | Returns input1 ^ input2
#[derive(Debug)]
pub struct FnPow {
    id: String,
    kind: FnKind,
    input1: FnInOutRef,
    input2: FnInOutRef,
}
//
// 
impl FnPow {
    ///
    /// Creates new instance of the FnPow
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input1: FnInOutRef, input2: FnInOutRef) -> Self {
        Self { 
            id: format!("{}/FnPow{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input1,
            input2,
        }
    }    
}
//
// 
impl FnIn for FnPow {}
//
// 
impl FnOut for FnPow { 
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
        let mut inputs = self.input1.borrow().inputs();
        inputs.extend(self.input2.borrow().inputs());
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<Point, String> {
        // TODO Mul overflow check
        let input1 = self.input1.borrow_mut().out();
        trace!("{}.out | input1: {:?}", self.id, &input1);
        let input1 = match input1 {
            FnResult::Ok(input1) => input1,
            FnResult::None => return FnResult::None,
            FnResult::Err(err) => return FnResult::Err(err),
        };
        let input2 = self.input2.borrow_mut().out();
        trace!("{}.out | input2: {:?}", self.id, &input2);
        let input2 = match input2 {
            FnResult::Ok(input2) => input2,
            FnResult::None => return FnResult::None,
            FnResult::Err(err) => return FnResult::Err(err),
        };
        let out = input1.pow(input2);
        trace!("{}.out | out: {:?}", self.id, &out);
        FnResult::Ok(out)
    }
    //
    //
    fn reset(&mut self) {
        self.input1.borrow_mut().reset();
        self.input2.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnPow {}
///
/// Global static counter of FnPow instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
