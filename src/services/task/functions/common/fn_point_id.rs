use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_sync::services::entity::{Point, PointConf, PointHlr};
use std::{hash::BuildHasherDefault, sync::atomic::{AtomicUsize, Ordering}};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef, 
    services::task::functions::{
        FnOut, FnKind, FnResult
    },
};
///
/// Function | Returns the ID of the point from input
/// 
/// Example
/// 
/// ```yaml
/// fn PointId:
///     input: point int /App/PointName
/// ```
#[derive(Debug)]
pub struct FnPointId {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    points: IndexMap<String, usize, BuildHasherDefault<FxHasher>>,
}
//
// 
impl FnPointId {
    ///
    /// Creates new instance of the FnPointId
    // #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef, points: Vec<PointConf>) -> Self {
        Self { 
            id: format!("{}/FnPointId{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
            points: points.into_iter().map(|p| {(p.name, p.id)}).collect(),
        }
    }    
}
//
// 
impl FnOut for FnPointId { 
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
                match self.points.get(&input.name()) {
                    Some(id) => {
                        log::debug!("{}.out | ID: {:?}", self.id, id);
                        FnResult::Ok(Point::Int(
                            PointHlr::new(
                                input.txid(),
                                &concat_string!(self.id, ".out"),
                                *id as i64,
                                input.status(),
                                input.cot(),
                                input.timestamp(),
                            )
                        ))
                    }
                    None => FnResult::Err(concat_string!(self.id, ".out | Point '", input.name(), "' - not found in configured points")),
                }
            }
            FnResult::None => FnResult::None,
            FnResult::Err(err) => FnResult::Err(err),
        }
    }
    //
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
