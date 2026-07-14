use sal_sync::{
    collections::FxHashMap,
    services::{entity::{Point, PointHlr, PointTxId},
    types::Bool,
}};
use testing::entities::test_value::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::{FnOutRef, PointMeta}, services::task::{FlowContext, FnFlow}};
use crate::services::task::{FnOut, FnKind, FnResult};
///
/// ### Function | `FnIsChangedValue`
/// 
/// - Returns true if at least one input is changed from prev value
/// - Status changes will not be registered.
/// - Timestamp changes will not be registered.
/// 
/// **Example**
/// ```yaml
/// fn IsChangedValue:
///     input1: point real '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnIsChangedValue {
    id: String,
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    state: FxHashMap<String, Value>,
    prev: Option<bool>,
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
            state: FxHashMap::default(),
            prev: None,
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
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter()
            .map(|input| input.borrow_mut().out()).collect();
        for input in inputs {
            if let Some(point) = flow.map(input)? {
                has_active = true;
                fb_meta = fb_meta.update_latest(&point);
                let key = point.name();
                log::trace!("{}.out | input '{}': {:?}", self.id, key, point);
                if let Some(state) = self.state.get_mut(&key) {
                    let value = point.value();
                    if value != *state {
                        log::trace!("{}.out | changed: {}  |  state '{:?}', value: {:?}", self.id, key, state, value);
                        meta = Some(meta.unwrap_or_default().update_latest(&point));
                        *state = value;
                        val = true;
                    }
                } else {
                    meta = Some(meta.unwrap_or_default().update_latest(&point));
                    self.state.insert(key, point.value());
                    val = true;
                }
            }
        }
        if !has_active {
            self.prev = None;
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
        // Если значение изменилось (val == true) -> это 100% свежий New(true).
        // Если значение не изменилось (val == false), но на прошлом такте было true -> мы обязаны опустить флаг через New(false).
        let is_changed = val || self.prev == Some(true);
        self.prev = Some(val);
        if is_changed {
            log::trace!("{}.out | FlowNew | value {:?} | {:?}", self.id, flow, value);
            return flow.wrap_new(value);
        }
        log::trace!("{}.out | FlowOld | value {:?} | {:?}", self.id, flow, value);
        flow.wrap_old(value)
    }
    //
    fn hard_reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().hard_reset();
        }
        self.state.clear();
        self.prev = None;
    }
    //
    fn reset(&mut self) {
        self.state.clear();
        self.prev = None;
    }
}
///
/// Global static counter of FnIsChangedValue instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
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
        fn hard_reset(&mut self) {}
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
    #[test]
    fn test_consecutive_changes() {
        let input = Rc::new(RefCell::new(FakeInput::new()));
        let mut fn_node = FnIsChangedValue::new("TestNs", vec![input.clone() as FnOutRef]);
        // Такт 1: Первое изменение
        input.borrow_mut().set(10, true);
        let res1 = fn_node.out().unwrap().unwrap();
        assert!(res1.is_new());
        assert_eq!(res1.value().to_bool().as_bool().value.0, true);
        // Такт 2: СРАЗУ ЖЕ второе изменение (импульс не должен потеряться)
        input.borrow_mut().set(20, true);
        let res2 = fn_node.out().unwrap().unwrap();
        assert!(res2.is_new(), "T2: Ожидается New, так как значение физически изменилось!");
        assert_eq!(res2.value().to_bool().as_bool().value.0, true);
        // Такт 3: Успокоились
        input.borrow_mut().set(20, false);
        let res3 = fn_node.out().unwrap().unwrap();
        assert!(res3.is_new(), "T3: Ожидается New(false) для снятия импульса");
        assert_eq!(res3.value().to_bool().as_bool().value.0, false);
    }
}
