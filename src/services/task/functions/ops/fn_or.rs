use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnOr`
/// 
/// Логическое сложение всех входящих сигналов. 
/// 
/// **Example**
/// 
/// ```yaml
/// fn Or:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Or:
///     in1: point bool '/App/Service/Point.Name1'
///     in2: point bool '/App/Service/Point.Name2'
///     in3: point bool '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnOr {
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
//
impl FnOr {
    ///
    /// Returns `FnOr` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `inputs`: Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnOr{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        if inputs.len() < 1 {
            return Err(Error::new(&id, "new").err("At least one input must be specified"));
        }
        Ok(Self { 
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `Point` с обновленными `txid`, `name`, `meta` и `value`
    #[inline]
    fn point_with(txid: usize, meta: &PointMeta, name: impl Into<String>, value: NumValue) -> Point {
        match value {
            NumValue::Bool(value) => Point::Bool(PointHlr::new(txid, name, Bool(value), meta.status, meta.cot, meta.ts)),
            NumValue::Int(value) => Point::Int(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            NumValue::Real(value) => Point::Real(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            NumValue::Double(value) => Point::Double(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
        }
    }
}
//
impl FnOut for FnOr {
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
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut flow = FlowContext::new();
        let mut value = NumValue::Bool(false);
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            let val: NumValue = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
            value = NumValue::Bool(value.or(&val).map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?);
        }
        flow.wrap(Self::point_with(self.txid, &meta, &self.id, value))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnOr instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};
    use super::*;
    use sal_sync::services::entity::{Cot, Status};
    #[derive(Debug)]
    struct MockOrigin {
        id: String,
        current_flow: FnResult<FnFlow, String>,
        was_called: bool,
        resets_count: usize,
    }
    impl MockOrigin {
        fn new(id: &str, flow: FnResult<FnFlow, String>) -> Self {
            Self { id: id.to_string(), current_flow: flow, was_called: false, resets_count: 0 }
        }
    }
    impl FnOut for MockOrigin {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![self.id.clone()] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.was_called = true;
            self.current_flow.clone()
        }
        fn reset(&mut self) { self.resets_count += 1; }
    }
    fn make_point(val: bool, status: Status) -> Point {
        Point::Bool(PointHlr::new(0, "test", Bool(val), status, Cot::Inf, chrono::offset::Utc::now()))
    }
    #[test]
    fn test_or_logic_and_taint_tracking() {
        let in1 = Rc::new(RefCell::new(MockOrigin::new("in1", Ok(Some(FnFlow::New(make_point(true, Status::Ok)))))));
        let in2 = Rc::new(RefCell::new(MockOrigin::new("in2", Ok(Some(FnFlow::Old(make_point(false, Status::Ok)))))));
        let mut node = FnOr::new("node", vec![in1.clone(), in2.clone()]).unwrap();
        let res = node.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)), "Если хотя бы один вход New, результат должен быть New");
        if let FnFlow::New(p) = res {
            assert_eq!(p.value().as_bool(), true, "true || false должно быть true");
        }
    }
    #[test]
    fn test_all_inputs_are_old_yields_old() {
        let in1 = Rc::new(RefCell::new(MockOrigin::new("in1", Ok(Some(FnFlow::Old(make_point(false, Status::Ok)))))));
        let in2 = Rc::new(RefCell::new(MockOrigin::new("in2", Ok(Some(FnFlow::Old(make_point(false, Status::Ok)))))));
        let mut node = FnOr::new("node", vec![in1.clone(), in2.clone()]).unwrap();
        let res = node.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_)), "Если все входы Old, результат должен оставаться Old");
        if let FnFlow::Old(p) = res {
            assert_eq!(p.value().as_bool(), false, "false || false должно быть false");
        }
    }
    #[test]
    fn test_fetch_phase_prevents_short_circuiting() {
        let in1 = Rc::new(RefCell::new(MockOrigin::new("in1", Ok(None))));
        let in2 = Rc::new(RefCell::new(MockOrigin::new("in2", Ok(Some(FnFlow::Old(make_point(true, Status::Ok)))))));
        let mut node = FnOr::new("node", vec![in1.clone(), in2.clone()]).unwrap();
        let res = node.out().unwrap();
        assert!(res.is_none(), "Если один из входов вернул None, весь узел выдает None");
        assert!(in1.borrow().was_called, "Первый узел должен быть опрошен");
        assert!(in2.borrow().was_called, "Второй узел обязан вызваться, предотвращая короткое замыкание");
    }
    #[test]
    fn test_reset_propagation() {
        let in1 = Rc::new(RefCell::new(MockOrigin::new("in1", Ok(None))));
        let in2 = Rc::new(RefCell::new(MockOrigin::new("in2", Ok(None))));
        let mut node = FnOr::new("node", vec![in1.clone(), in2.clone()]).unwrap();
        node.reset();
        assert_eq!(in1.borrow().resets_count, 1);
        assert_eq!(in2.borrow().resets_count, 1);
    }
}
