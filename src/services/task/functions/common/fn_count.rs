use std::sync::atomic::{AtomicUsize, Ordering};
use sal_sync::services::entity::{Point, PointHlr};
use crate::domain::FnOutRef;
use crate::services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult};
///
/// Counts number of raised fronts of boolean input
#[derive(Debug)]
pub struct FnCount {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    prev: bool,
    count: Option<i64>,
    initial: Option<FnOutRef>,
}
//
// 
impl FnCount {
    ///
    /// Creates new instance of the FnCount
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, initial: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnCount{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            input,
            prev: false,
            count: None,
            initial,
        }
    }
}
//
// 
impl FnOut for FnCount {
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
        inputs.append(&mut self.input.borrow().inputs());
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.borrow().inputs());
        }
        inputs
    }
    ///
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        let input_is_new = flow.is_new();

        let mut count = match self.count {
            Some(count) => count,
            None => {
                let val = if let Some(initial) = &self.initial {
                    let Some(initial) = flow.map(initial.borrow_mut().out())? else { return Ok(None) };
                    initial.to_int().as_int().value
                } else {
                    0
                };
                self.count = Some(val);
                val
            }
        };
        if !input_is_new {
            return flow.wrap(Point::Int(PointHlr::new(
                input.txid(),
                &format!("{}", self.id),
                count,
                input.status(),
                input.cot(),
                input.timestamp(),
            )));
        };
        let val = input.to_bool().as_bool().value.0;
        if !self.prev && val {
            count += 1;
            self.count = Some(count);
        }
        self.prev = val;
        log::trace!("{}.out | value: {:?}", self.id, count);
        self.count = Some(count.clone());
        flow.wrap(Point::Int(PointHlr::new(
            input.txid(),
            &format!("{}", self.id),
            count,
            input.status(),
            input.cot(),
            input.timestamp(),
        )))
    }
    fn reset(&mut self) {
        self.count = None;
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnCount instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
