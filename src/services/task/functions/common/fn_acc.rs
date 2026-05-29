use sal_sync::services::entity::{Point, PointHlr, PointType};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::domain::FnOutRef;
use crate::services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult};
///
/// Accumulates numeric incoming Point's value
/// - if input is not numeric - returns Err
/// - if input is bool, false = 0, true = 1
#[derive(Debug)]
pub struct FnAcc {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    acc: Option<Point>,
    initial: Option<FnOutRef>,
}
// 
impl FnAcc {
    ///
    /// Creates new instance of the FnAcc
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, initial: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnAcc{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
            acc: None,
            initial,
        }
    }
}
// 
impl FnOut for FnAcc {
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
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.borrow().inputs());
        }
        inputs.append(&mut self.input.borrow().inputs());
        inputs
    }
    ///
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        // trace!("{}.out | input: {:?}", self.id, input);
        let acc = match self.acc.as_ref() {
            Some(acc) => acc.clone(),
            None => {
                let acc = if let Some(initial) = &self.initial {
                    let Some(initial) = initial.borrow_mut().out()? else { return Ok(None) };
                    initial.into_value()
                } else {
                    match input.type_() {
                        PointType::Bool | PointType::Int => Point::Int(PointHlr::new(
                            input.txid(), &input.name(), 0, input.status(), input.cot(), input.timestamp(),
                        )),
                        PointType::Real => Point::Real(PointHlr::new(
                            input.txid(), &input.name(), 0.0, input.status(), input.cot(), input.timestamp(),
                        )),
                        PointType::Double => Point::Double(PointHlr::new(
                            input.txid(), &input.name(), 0.0, input.status(), input.cot(), input.timestamp(),
                        )),
                        _ => return Err(format!("{}.out | Invalid input type '{:?}', expected number", self.id, input.type_())),
                    }
                };
                self.acc = Some(acc.clone());
                acc
            }
        };
        if !flow.is_new() {
            return flow.wrap(acc);
        };
        let acc = match &input {
            Point::Bool(_) => acc + input.to_int(),
            _ => acc + input,
        };
        log::trace!("{}.out | out: {:?}", self.id, acc);
        self.acc = Some(acc.clone());
        flow.wrap(acc)
    }
    fn reset(&mut self) {
        if let Some(initial) = &self.initial {
            initial.borrow_mut().reset();
        }
        self.acc = None;
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnAcc instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
