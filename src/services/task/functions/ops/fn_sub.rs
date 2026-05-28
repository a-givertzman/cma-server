use sal_sync::services::entity::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{
        FnOut, FnKind, FnResult
    },
};
///
/// Function | Returns input1 - input2
#[derive(Debug)]
pub struct FnSub {
    id: String,
    kind: FnKind,
    input1: FnOutRef,
    input2: FnOutRef,
}
//
// 
impl FnSub {
    ///
    /// Creates new instance of the FnSub
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input1: FnOutRef, input2: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnSub{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input1,
            input2,
        }
    }    
}
//
// 
impl FnOut for FnSub { 
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
        let mut inputs = self.input1.borrow().inputs();
        inputs.extend(self.input2.borrow().inputs());
        inputs
    }
    //
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        // TODO Add overflow check
        let input1 = self.input1.borrow_mut().out();
        log::trace!("{}.out | input1: {:?}", self.id, &input1);
        let input1 = match input1 {
            FnResult::Ok(input1) => input1,
            FnResult::None => return FnResult::None,
            FnResult::Err(err) => return FnResult::Err(err),
        };
        let input2 = self.input2.borrow_mut().out();
        log::trace!("{}.out | input2: {:?}", self.id, &input2);
        let input2 = match input2 {
            FnResult::Ok(input2) => input2,
            FnResult::None => return FnResult::None,
            FnResult::Err(err) => return FnResult::Err(err),
        };
        let out = input1 - input2;
        log::trace!("{}.out | out: {:?}", self.id, &out);
        FnResult::Ok(out)
    }
    //
    //
    fn reset(&mut self) {
        self.input1.borrow_mut().reset();
        self.input2.borrow_mut().reset();
    }
}
///
/// Global static counter of FnSub instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
