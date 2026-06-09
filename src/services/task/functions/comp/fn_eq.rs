use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, PointTxId}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta, Value},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnEq`
/// 
/// Выполняет операцию логического сравнения EQ (Equal) `v1 == v2`
/// Динамически приводит типы данных.
/// 
/// **Example**
/// ```yaml
/// fn Eq:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Eq:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnEq {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
impl FnEq {
    ///
    /// Returns `FnEq` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnEq{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id,
        })
    }
    ///
    /// Возвращает `PointHlr` с обновленными `txid`, `name` и `value`
    #[inline]
    fn point_with<T>(txid: usize, meta: &PointMeta, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(txid, name, value, meta.status, meta.cot, meta.ts)
    }
}
//
impl FnOut for FnEq {
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
        let mut inputs: Vec<FnResult<FnFlow, String>> = self.inputs.iter().map(|input| {
            input.borrow_mut().out()
        }).collect();
        let mut meta = PointMeta::default();
        let mut flow = FlowContext::new();
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1: Value = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2: Value = input.try_into().map_err(|err: Error| concat_string::concat_string!(self.id, ".out | ", err.to_string()))?;
        let value = v1 == v2;
        flow.wrap(Point::Bool(Self::point_with(self.txid, &meta, &self.id, Bool(value))))
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnEq instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use sal_sync::services::entity::{Cot, Status};

use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    #[derive(Debug)]
    struct MockInput {
        result: FnResult<FnFlow, String>,
        inputs_called: usize,
    }
    impl FnOut for MockInput {
        fn id(&self) -> String { "mock".to_string() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec!["mock_point".to_string()] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.inputs_called += 1;
            self.result.clone()
        }
        fn reset(&mut self) {}
    }
    fn create_mock(flow: FnFlow) -> FnOutRef {
        Rc::new(RefCell::new(MockInput { result: Ok(Some(flow)), inputs_called: 0 }))
    }
    fn create_none_mock() -> FnOutRef {
        Rc::new(RefCell::new(MockInput { result: Ok(None), inputs_called: 0 }))
    }
    #[test]
    fn test_eq_both_new_true() {
        let ts = chrono::Utc::now();
        let p1 = Point::Double(PointHlr::new(1, "p1", 10.5, Status::Ok, Cot::Inf, ts));
        let p2 = Point::Double(PointHlr::new(1, "p2", 5.5, Status::Ok, Cot::Inf, ts));
        let in1 = create_mock(FnFlow::New(p1));
        let in2 = create_mock(FnFlow::New(p2));
        let mut eq = FnEq::new("test", vec![in1, in2]).unwrap();
        let res = eq.out().unwrap();
        assert!(matches!(res, Some(FnFlow::New(_))));
        assert_eq!(res.unwrap().into_value().to_bool().as_bool().value.0, false);
    }
    #[test]
    fn test_eq_both_old_optimization() {
        let ts = chrono::Utc::now();
        let p1 = Point::Double(PointHlr::new(1, "p1", 5.5, Status::Ok, Cot::Inf, ts));
        let p2 = Point::Double(PointHlr::new(1, "p2", 5.5, Status::Ok, Cot::Inf, ts));
        let in1 = create_mock(FnFlow::Old(p1));
        let in2 = create_mock(FnFlow::Old(p2));
        let mut eq = FnEq::new("test", vec![in1, in2]).unwrap();
        let res = eq.out().unwrap();
        assert!(matches!(res, Some(FnFlow::Old(_))));
        assert_eq!(res.unwrap().into_value().to_bool().as_bool().value.0, true);
    }
    #[test]
    fn test_eq_one_new_triggers_recalculation() {
        let ts = chrono::Utc::now();
        let p1 = Point::Double(PointHlr::new(1, "p1", 3.0, Status::Ok, Cot::Inf, ts));
        let p2 = Point::Double(PointHlr::new(1, "p2", 4.0, Status::Ok, Cot::Inf, ts));
        let in1 = create_mock(FnFlow::New(p1));
        let in2 = create_mock(FnFlow::Old(p2));
        let mut eq = FnEq::new("test", vec![in1, in2]).unwrap();
        let res = eq.out().unwrap();
        assert!(matches!(res, Some(FnFlow::New(_))));
    }
    #[test]
    fn test_eq_none_short_circuit_propagation() {
        let ts = chrono::Utc::now();
        let p1 = Point::Double(PointHlr::new(1, "p1", 3.0, Status::Ok, Cot::Inf, ts));
        let in1 = create_mock(FnFlow::New(p1));
        let in2 = create_none_mock();
        let mut eq = FnEq::new("test", vec![in1, in2]).unwrap();
        let res = eq.out().unwrap();
        assert!(res.is_none());
    }
}
