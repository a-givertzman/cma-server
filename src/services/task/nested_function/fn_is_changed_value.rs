use std::{collections::HashMap, hash::BuildHasherDefault, sync::atomic::{AtomicUsize, Ordering}};
use chrono::Utc;
use hashers::fx_hash::FxHasher;
use log::trace;
use crate::core_::{
    cot::cot::Cot, point::{point_hlr::PointHlr, point_tx_id::PointTxId, point_type::PointType},
    status::status::Status, types::{bool::Bool, fn_in_out_ref::FnInOutRef, map::HashMapFxHasher},
};
use super::{fn_::{FnIn, FnInOut, FnOut}, fn_kind::FnKind, fn_result::FnResult};
///
/// Function | Returns true if at least one input is changed from prev value
/// - status chcanges will not registered
/// - timestamp changes will not registered
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
    kind: FnKind,
    inputs: Vec<FnInOutRef>,
    state: HashMapFxHasher<String, PointType>,
}
//
// 
impl FnIsChangedValue {
    ///
    /// Creates new instance of the FnIsChangedValue
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnInOutRef>) -> Self {
        Self { 
            id: format!("{}/FnIsChangedValue{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind:FnKind::Fn,
            inputs,
            state: HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
        }
    }
}
//
// 
impl FnIn for FnIsChangedValue {}
//
// 
impl FnOut for FnIsChangedValue {
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
        let mut inputs = vec![];
        for input in &self.inputs {
            inputs.append(&mut input.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<PointType, String> {
        let tx_id = PointTxId::from_str(&self.id);
        let mut value = false;
        let state: HashMap<&String, testing::entities::test_value::Value> = self.state.iter().map(|(name, p)| (name, p.value())).collect();
        trace!("{}.out | state: {:#?}", self.id, state);
        for input in &self.inputs {
            let input = input.borrow_mut().out();
            match input {
                FnResult::Ok(input) => {
                    trace!("{}.out | input '{}': {:#?}", self.id, input.name(), input);
                    let state = self.state
                        .entry(input.name())
                        .or_insert_with(|| {
                            value = true;
                            input.clone()
                        });
                    if !input.cmp_value(state) {
                        trace!("{}.out | changed: {}  |  state '{:?}', value: {:?}", self.id, input.name(), state.value(), input.value());
                        *state = input;
                        value = true;
                    }
                }
                FnResult::None => return FnResult::None,
                FnResult::Err(err) => return FnResult::Err(err),
            }
        }
        trace!("{}.out | value: {:#?}", self.id, value);
        FnResult::Ok(PointType::Bool(
            PointHlr::new(
                tx_id,
                &format!("{}.out", self.id),
                Bool(value),
                Status::Ok,
                Cot::Inf,
                Utc::now(),
            )
        ))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
        self.state.clear();
    }
}
//
// 
impl FnInOut for FnIsChangedValue {}
///
/// Global static counter of FnIsChangedValue instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
