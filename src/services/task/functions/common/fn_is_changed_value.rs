use sal_sync::{
    collections::FxHashMap,
    services::{entity::{Cot, {Point, PointHlr, PointTxId}, Status},
    types::Bool,
}};
use std::{collections::HashMap, hash::BuildHasherDefault, sync::atomic::{AtomicUsize, Ordering}};
use chrono::Utc;
use hashers::fx_hash::FxHasher;
use crate::{domain::{EdgeDetector, FnOutRef}, services::task::{FlowContext, FnFlow}};
use crate::services::task::{FnOut, FnKind, FnResult};
///
/// ### Function | Returns true if at least one input is changed from prev value
/// - Status changes will not be registered.
/// - Timestamp changes will not be registered.
/// 
/// Example
/// 
/// ```yaml
/// fn FnIsChangedValue:
///     input1: point real '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnIsChangedValue {
    id: String,
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    state: FxHashMap<String, Point>,
    edge: EdgeDetector,
}
// 
impl FnIsChangedValue {
    ///
    /// Creates a new instance of `FnIsChangedValue`.
    /// - `parent`: The namespace/path prefix for the ID.
    /// - `inputs`: Vector of references to upstream functions.
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Self {
        let id = format!("{}/FnIsChangedValue{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let txid = PointTxId::from_str(&id);
        Self { 
            id,
            txid,
            kind: FnKind::Fn,
            inputs,
            state: HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
            edge: EdgeDetector::new(),
        }
    }
}
//
impl FnOut for FnIsChangedValue {
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
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let mut val = false;
        // let state = FxHashMap::from_iter(self.state.iter().map(|(name, p)| (name, p.value())));
        // log::trace!("{}.out | state: {:#?}", self.id, state);
        for input in &self.inputs {
            if let Some(point) = flow.map(input.borrow_mut().out())? {
                let key = point.name();
                log::trace!("{}.out | input '{}': {:?}", self.id, key, point);
                if let Some(state) = self.state.get_mut(&key) {
                    if !point.cmp_value(state) {
                        log::trace!("{}.out | changed: {}  |  state '{:?}', value: {:?}", self.id, key, state.value(), point.value());
                        *state = point;
                        val = true;
                    }
                } else {
                    self.state.insert(key, point);
                    val = true;
                }
            }
        }
        let value = Point::Bool(PointHlr::new(
            self.txid,
            &self.id,
            Bool(val),
            Status::Ok,
            Cot::Inf,
            Utc::now(),
        ));
        _ = self.edge.add(val);
        if self.edge.is_rising() || self.edge.is_falling() {
            log::trace!("{}.out | value {:?} | {:?}", self.id, flow, value);
            return flow.wrap_new(value);
        }
        log::trace!("{}.out | value {:?} | {:?}", self.id, flow, value);
        flow.wrap(value)
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
        self.state.clear();
    }
}
///
/// Global static counter of FnIsChangedValue instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
