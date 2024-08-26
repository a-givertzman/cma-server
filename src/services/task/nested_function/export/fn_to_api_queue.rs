use sal_sync::services::entity::point::point::Point;
use std::sync::{mpsc::Sender, atomic::{AtomicUsize, Ordering}};
use log::{debug, error, trace};
use crate::{core_::types::fn_in_out_ref::FnInOutRef, services::task::nested_function::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult}};
///
/// Exports data from the input into the associated queue
#[derive(Debug)]
pub struct FnToApiQueue {
    id: String,
    kind: FnKind,
    input: FnInOutRef,
    tx_send: Sender<Point>,
    state: String,
}
///
/// 
static COUNT: AtomicUsize = AtomicUsize::new(1);
//
// 
impl FnToApiQueue {
    ///
    /// creates new instance of the FnToApiQueue
    /// - id - just for proper debugging
    /// - input - incoming points
    pub fn new(parent: impl Into<String>, input: FnInOutRef, send: Sender<Point>) -> Self {
        Self {  
            id: format!("{}/FnToApiQueue{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
            tx_send: send,
            state: String::new(),
        }
    }
}
//
//
impl FnIn for FnToApiQueue {}
//
// 
impl FnOut for FnToApiQueue {
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
    fn out(&mut self) -> FnResult<Point, String> {
        let input = self.input.borrow_mut().out();
        trace!("{}.out | input: {:?}", self.id, input);
        match input {
            FnResult::Ok(input) => {
                let sql = input.as_string().value;
                if sql != self.state {
                    self.state = sql.clone();
                    match self.tx_send.send(input.clone()) {
                        Ok(_) => {
                            debug!("{}.out | Sent sql: {}", self.id, sql);
                        }
                        Err(err) => {
                            error!("{}.out | Send error: {:?}\n\tsql: {:?}", self.id, err, sql);
                        }
                    };
                }
                FnResult::Ok(input)
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
//
// 
impl FnInOut for FnToApiQueue {}
