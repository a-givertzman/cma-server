use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use crate::{
    domain::{FnOutRef, PointMeta, NumValue},
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// ### Function | `FnSub`
/// 
/// Возвращает разность `input1 - input2`,
/// динамически повышая тип данных до наиболее точного.
/// 
/// **Example**
/// ```yaml
/// fn Sub:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Sub:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
/// ```
#[derive(Debug)]
pub struct FnSub {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
//
// 
impl FnSub {
    ///
    /// Creates `FnSub` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnSub{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
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
impl FnOut for FnSub { 
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
        // TODO Add overflow check
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1 = match input {
            Point::Bool(p) => NumValue::Bool(p.value.0),
            Point::Int(p) => NumValue::Int(p.value),
            Point::Real(p) => NumValue::Real(p.value),
            Point::Double(p) => NumValue::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2 = match input {
            Point::Bool(p) => NumValue::Bool(p.value.0),
            Point::Int(p) => NumValue::Int(p.value),
            Point::Real(p) => NumValue::Real(p.value),
            Point::Double(p) => NumValue::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let value = (v1 - v2).map_err(|_| format!("{}.out | Can't sub {:?} - {:?}", self.id, v1, v2))?;
        match value {
            NumValue::Bool(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value as i64))),
            NumValue::Int(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value))),
            NumValue::Real(value) => flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, value))),
            NumValue::Double(value) => flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, value))),
        }
    }
    //
    fn reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().reset();
        }
    }
}
///
/// Global static counter of FnSub instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use sal_sync::services::entity::{Cot, Status};
    #[derive(Debug)]
    struct MockNode {
        output: FnResult<FnFlow, String>,
    }
    impl FnOut for MockNode {
        fn id(&self) -> String { "mock".to_string() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { self.output.clone() }
        fn reset(&mut self) {}
    }
    fn make_point_int(val: i64) -> Point {
        Point::Int(PointHlr::new(0, "p", val, Status::Ok, Cot::Inf, chrono::offset::Utc::now()))
    }
    fn make_point_double(val: f64) -> Point {
        Point::Double(PointHlr::new(0, "p", val, Status::Ok, Cot::Inf, chrono::offset::Utc::now()))
    }
    fn make_point_real(val: f32) -> Point {
        Point::Real(PointHlr::new(0, "p", val, Status::Ok, Cot::Inf, chrono::offset::Utc::now()))
    }
    #[test]
    fn test_sub_type_promotion_and_math() {
        let n1 = Rc::new(RefCell::new(MockNode { output: Ok(Some(FnFlow::Old(make_point_int(10)))) }));
        let n2 = Rc::new(RefCell::new(MockNode { output: Ok(Some(FnFlow::New(make_point_real(2.5)))) }));
        let mut sub = FnSub::new("root", vec![n1, n2]).unwrap();
        let res = sub.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(Point::Real(_))));
        if let FnFlow::New(Point::Real(p)) = res {
            assert_eq!(p.value, 7.5f32);
        }
    }
    #[test]
    fn test_taint_tracking_propagation() {
        let n1 = Rc::new(RefCell::new(MockNode { output: Ok(Some(FnFlow::Old(make_point_int(5)))) }));
        let n2 = Rc::new(RefCell::new(MockNode { output: Ok(Some(FnFlow::Old(make_point_int(2)))) }));
        let mut sub = FnSub::new("root", vec![n1, n2]).unwrap();
        let res = sub.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_)));
    }
    #[test]
    fn test_short_circuit_none_break() {
        let n1 = Rc::new(RefCell::new(MockNode { output: Ok(None) }));
        let n2 = Rc::new(RefCell::new(MockNode { output: Ok(Some(FnFlow::New(make_point_int(5)))) }));
        let mut sub = FnSub::new("root", vec![n1, n2]).unwrap();
        let res = sub.out().unwrap();
        assert!(res.is_none());
    }
    #[test]
    fn test_nan_input_returns_err() {
        let n1 = Rc::new(RefCell::new(MockNode { output: Ok(Some(FnFlow::New(make_point_double(f64::NAN)))) }));
        let n2 = Rc::new(RefCell::new(MockNode { output: Ok(Some(FnFlow::New(make_point_int(5)))) }));
        let mut sub = FnSub::new("root", vec![n1, n2]).unwrap();
        let res = sub.out();
        assert!(res.is_err());
    }
}