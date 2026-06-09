use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, Value},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnBitAnd`
/// 
/// Побитовое логическое умножение всех входящих сигналов. 
/// 
/// **Example**
/// 
/// ```yaml
/// fn BitAnd:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn BitAnd:
///     in1: point bool '/App/Service/Point.Name1'
///     in2: point bool '/App/Service/Point.Name2'
///     in3: point bool '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnBitAnd {
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
//
impl FnBitAnd {
    ///
    /// Returns `FnBitAnd` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `inputs`: Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnBitAnd{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        return Err(Error::new(&id, "new").err("Isn't implemented yet"));
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
    /// - `txid`: Текущий идентификатор отправителя.
    /// - `meta`: Объединенные метаданные всех задействованных входов.
    /// - `name`: Имя формируемого сигнала.
    /// - `value`: Значение формируемого сигнала.
    #[inline]
    fn point_with(txid: usize, meta: &PointMeta, name: impl Into<String>, value: Value) -> Point {
        match value {
            Value::Bool(value) => Point::Bool(PointHlr::new(txid, name, Bool(value), meta.status, meta.cot, meta.ts)),
            Value::Int(value) => Point::Int(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            Value::Real(value) => Point::Real(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
            Value::Double(value) => Point::Double(PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)),
        }
    }
}
//
impl FnOut for FnBitAnd {
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
        unimplemented!();
        let inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut flow = FlowContext::new();
        let mut value = Value::Bool(true);
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            let val: Value = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
            value = Value::Bool(value .and(&val).map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?);
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
/// Global static counter of FnBitAnd instances
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
    fn test_and_true_and_true() {
        let in1 = Rc::new(RefCell::new(MockOrigin::new("in1", Ok(Some(FnFlow::New(make_point(true, Status::Ok)))))));
        let in2 = Rc::new(RefCell::new(MockOrigin::new("in2", Ok(Some(FnFlow::New(make_point(true, Status::Ok)))))));
        let mut and_node = FnBitAnd::new("task", vec![in1, in2]).unwrap();
        let res = and_node.out().unwrap().unwrap();
        match res {
            FnFlow::New(point) => {
                let val = point.as_bool().value.0;
                assert!(val);
            }
            _ => panic!("Expected New flow"),
        }
    }
    #[test]
    fn test_and_true_and_false() {
        let in1 = Rc::new(RefCell::new(MockOrigin::new("in1", Ok(Some(FnFlow::New(make_point(true, Status::Ok)))))));
        let in2 = Rc::new(RefCell::new(MockOrigin::new("in2", Ok(Some(FnFlow::New(make_point(false, Status::Ok)))))));
        let mut and_node = FnBitAnd::new("task", vec![in1, in2]).unwrap();
        let res = and_node.out().unwrap().unwrap();
        match res {
            FnFlow::New(point) => {
                let val = point.as_bool().value.0;
                assert!(!val);
            }
            _ => panic!("Expected New flow"),
        }
    }
    #[test]
    fn test_and_taint_tracking_old() {
        let in1 = Rc::new(RefCell::new(MockOrigin::new("in1", Ok(Some(FnFlow::Old(make_point(true, Status::Ok)))))));
        let in2 = Rc::new(RefCell::new(MockOrigin::new("in2", Ok(Some(FnFlow::Old(make_point(true, Status::Ok)))))));
        let mut and_node = FnBitAnd::new("task", vec![in1, in2]).unwrap();
        let res = and_node.out().unwrap().unwrap();
        match res {
            FnFlow::Old(point) => {
                let val = point.as_bool().value.0;
                assert!(val);
            }
            _ => panic!("Expected Old flow"),
        }
    }
    #[test]
    fn test_and_cold_mode() {
        let in1 = Rc::new(RefCell::new(MockOrigin::new("in1", Ok(None))));
        let in2 = Rc::new(RefCell::new(MockOrigin::new("in2", Ok(Some(FnFlow::New(make_point(true, Status::Ok)))))));
        let mut and_node = FnBitAnd::new("task", vec![in1, in2]).unwrap();
        let res = and_node.out().unwrap();
        assert!(res.is_none());
    }
}
