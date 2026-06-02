use sal_sync::{
    collections::FxHashMap,
    services::{entity::{Cot, Point, PointHlr, PointTxId, Status},
    types::Bool,
}};
use std::{collections::HashMap, hash::BuildHasherDefault, sync::atomic::{AtomicUsize, Ordering}};
use chrono::{DateTime, Utc};
use hashers::fx_hash::FxHasher;
use crate::{domain::{EdgeDetector, FnOutRef}, services::task::{FlowContext, FnFlow}};
use crate::services::task::{FnOut, FnKind, FnResult};
///
/// ### Function | FnIsChangedValue
/// 
/// - Returns true if at least one input is changed from prev value
/// - Status changes will not be registered.
/// - Timestamp changes will not be registered.
/// 
/// **Example**
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
        let mut fb_meta = PointMeta::default();
        let mut meta = None::<PointMeta>;
        let mut val = false;
        let mut has_active = false;
        for input in &self.inputs {
            if let Some(point) = flow.map(input.borrow_mut().out())? {
                has_active = true;
                fb_meta = fb_meta.update_latest(&point);
                let key = point.name();
                log::trace!("{}.out | input '{}': {:?}", self.id, key, point);
                if let Some(state) = self.state.get_mut(&key) {
                    if !point.cmp_value(state) {
                        log::trace!("{}.out | changed: {}  |  state '{:?}', value: {:?}", self.id, key, state.value(), point.value());
                        meta = Some(meta.unwrap_or_default().update_latest(&point));
                        *state = point;
                        val = true;
                    }
                } else {
                    meta = Some(meta.unwrap_or_default().update_latest(&point));
                    self.state.insert(key, point);
                    val = true;
                }
            }
        }
        if !has_active {
            return Ok(None);
        }
        let meta = meta.unwrap_or(fb_meta);
        let value = Point::Bool(PointHlr::new(
            self.txid,
            &self.id,
            Bool(val),
            meta.status,
            meta.cot,
            meta.ts,
        ));
        _ = self.edge.add(val);
        if self.edge.is_rising() || self.edge.is_falling() {
            log::trace!("{}.out | value {:?} | {:?}", self.id, flow, value);
            return flow.wrap_new(value);
        }
        log::trace!("{}.out | value {:?} | {:?}", self.id, flow, value);
        flow.wrap_old(value)
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
        self.state.clear();
        self.edge.reset();
    }
}
///
/// Global static counter of FnIsChangedValue instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Local container for Point meta
#[derive(Clone, Copy)]
struct PointMeta {
    pub status: Status,
    pub cot: Cot,
    pub ts: chrono::DateTime<Utc>,
}
impl PointMeta {
    pub fn update(self, p: &Point) -> Self {
        Self {
            status: p.status(),
            cot: p.cot(),
            ts: p.timestamp(),
        }
    }
    pub fn update_latest(self, p: &Point) -> Self {
        if p.timestamp() > self.ts {
            self.update(p)
        } else {
            self
        }
    }
}
impl Default for PointMeta {
    fn default() -> Self {
        Self { status: Status::Ok, cot: Cot::Inf, ts: DateTime::<Utc>::MIN_UTC }
    }
}
///
/// Basic tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::task::{FnFlow, FnKind, FnResult};
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    use std::cell::RefCell;
    use std::rc::Rc;
    use chrono::Utc;
    #[derive(Debug)]
    struct FakeInput {
        point: Option<Point>,
        is_new: bool,
    }
    impl FakeInput {
        fn new() -> Self {
            Self { point: None, is_new: false }
        }
        fn set(&mut self, val: i64, is_new: bool) {
            self.point = Some(Point::Int(PointHlr::new(
                0,
                "test_input",
                val,
                Status::Ok,
                Cot::Inf,
                Utc::now(),
            )));
            self.is_new = is_new;
        }
    }
    impl FnOut for FakeInput {
        fn id(&self) -> String { "fake_input".to_string() }
        fn kind(&self) -> FnKind { FnKind::Input }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            let p = self.point.clone().unwrap();
            if self.is_new {
                Ok(Some(FnFlow::New(p)))
            } else {
                Ok(Some(FnFlow::Old(p)))
            }
        }
        fn reset(&mut self) {}
    }
    #[test]
    fn test_edge_detection_sequence() {
        let input = Rc::new(RefCell::new(FakeInput::new()));
        let mut fn_node = FnIsChangedValue::new("TestNs", vec![input.clone() as FnOutRef]);
        // Такт 1: Приходят новые данные, значение изменилось (инициализация)
        input.borrow_mut().set(10, true);
        let res1 = fn_node.out().unwrap().unwrap();
        assert!(res1.is_new(), "T1: Ожидается New (передний фронт)");
        if let Point::Bool(p) = res1.value() {
            assert_eq!(p.value.0, true);
        } else {
            panic!("Ожидался тип Bool");
        }
        // Такт 2: Вход отдает старые данные (тишина в эфире)
        input.borrow_mut().set(10, false);
        let res2 = fn_node.out().unwrap().unwrap();
        assert!(res2.is_new(), "T2: Ожидается принудительный выброс New (задний фронт)");
        if let Point::Bool(p) = res2.value() {
            assert_eq!(p.value.0, false);
        } else {
            panic!("Ожидался тип Bool");
        }
        // Такт 3: Продолжение тишины
        input.borrow_mut().set(10, false);
        let res3 = fn_node.out().unwrap().unwrap();
        assert!(!res3.is_new(), "T3: Ожидается Old (состояние стабилизировалось)");
        if let Point::Bool(p) = res3.value() {
            assert_eq!(p.value.0, false);
        } else {
            panic!("Ожидался тип Bool");
        }
        // Такт 4: Значение снова меняется
        input.borrow_mut().set(20, true);
        let res4 = fn_node.out().unwrap().unwrap();
        assert!(res4.is_new(), "T4: Ожидается повторный New (передний фронт)");
        if let Point::Bool(p) = res4.value() {
            assert_eq!(p.value.0, true);
        } else {
            panic!("Ожидался тип Bool");
        }
    }
}
