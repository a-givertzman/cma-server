use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use crate::{
    domain::{FnOutRef, PointMeta, Value},
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnDiv`
/// 
/// Возвращает результат деления `input1 / input2`,
/// динамически повышая тип данных до наиболее точного.
/// 
/// **Внимание (Целочисленное деление):** Если оба входа имеют тип `Int` или `Bool`, 
/// деление выполняется с отсечением дробной части (например, `5 / 2 = 2`). 
/// Для получения результата с плавающей точкой (например, `2.5`), хотя бы один 
/// из источников должен быть приведен к типу `Real` или `Double`.
/// 
/// **Example**
/// ```yaml
/// fn Div:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Div:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
#[derive(Debug)]
pub struct FnDiv {
    txid: usize,
    kind: FnKind,
    input1: FnOutRef,
    input2: FnOutRef,
    inputs: [FnOutRef; 2],
    id: String,
}
//
// 
impl FnDiv {
    ///
    /// Creates `FnDiv` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnDiv{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst));
        let inputs: [FnOutRef; 2] = inputs.try_into()
            .map_err(|_| Error::new(&id, "new").err("Two inputs must be specified"))?;
        Ok(Self { 
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            input1: inputs[0].clone(),
            input2: inputs[1].clone(),
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
// 
impl FnOut for FnDiv { 
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
        // TODO Div overflow check
        let (input1, input2) = (inputs.remove(0), inputs.remove(0));
        let Some(input) = flow.map(input1)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v1 = match input {
            Point::Bool(p) => Value::Bool(p.value.0),
            Point::Int(p) => Value::Int(p.value),
            Point::Real(p) => Value::Real(p.value),
            Point::Double(p) => Value::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let Some(input) = flow.map(input2)? else { return Ok(None) };
        meta = meta.update_latest(&input).update_status(&input);
        let v2 = match input {
            Point::Bool(p) => Value::Bool(p.value.0),
            Point::Int(p) => Value::Int(p.value),
            Point::Real(p) => Value::Real(p.value),
            Point::Double(p) => Value::Double(p.value),
            _ => return Err(concat_string::concat_string!(self.id, ".out | Invalid type '", input.typ().to_string(), "'")),
        };
        let value = (v1 / v2).map_err(|_| format!("{}.out | Can't div {:?} / {:?}", self.id, v1, v2))?;
        match value {
            Value::Bool(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value as i64))),
            Value::Int(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value))),
            Value::Real(value) => flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, value))),
            Value::Double(value) => flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, value))),
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
/// Global static counter of FnDiv instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Test
#[cfg(test)]
mod tests {
    use super::*;
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    use sal_sync::services::types::Bool;
    use std::cell::RefCell;
    use std::rc::Rc;
    #[derive(Debug)]
    struct MockNode {
        id: String,
        flow: FnResult<FnFlow, String>,
        called: usize,
    }
    impl MockNode {
        fn new(id: &str, flow: FnResult<FnFlow, String>) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self { id: id.to_string(), flow, called: 0 }))
        }
        fn new_none(id: &str) -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self { id: id.to_string(), flow: Ok(None), called: 0 }))
        }
    }
    impl FnOut for MockNode {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { return FnKind::Fn; }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.called += 1;
            self.flow.clone()
        }
        fn reset(&mut self) {}
    }
    fn mock_point_bool(id: &str, val: bool) -> Point {
        Point::Bool(PointHlr::new(0, id, Bool(val), Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    fn mock_point_int(id: &str, val: i64) -> Point {
        Point::Int(PointHlr::new(0, id, val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    fn mock_point_real(id: &str, val: f32) -> Point {
        Point::Real(PointHlr::new(0, id, val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    fn mock_point_double(id: &str, val: f64) -> Point {
        Point::Double(PointHlr::new(0, id, val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    #[test]
    fn test_fndiv_division_by_zero() {
        let in1 = MockNode::new("in1", Ok(Some(FnFlow::New(mock_point_int("in1", 10)))));
        let in2 = MockNode::new("in2", Ok(Some(FnFlow::New(mock_point_int("in2", 0)))));
        let mut div = FnDiv::new("test", vec![in1, in2]).unwrap();
        let res = div.out();
        assert!(res.is_err(), "Деление на ноль должно возвращать Err");
    }
    #[test]
    fn test_fndiv_integer_truncation() {
        let in1 = MockNode::new("in1", Ok(Some(FnFlow::New(mock_point_int("in1", 5)))));
        let in2 = MockNode::new("in2", Ok(Some(FnFlow::New(mock_point_int("in2", 2)))));
        let mut div = FnDiv::new("test", vec![in1, in2]).unwrap();
        let res = div.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)), "Результат должен быть New");
        match res.into_value() {
            Point::Int(p) => assert_eq!(p.value, 2, "Должно произойти усечение 5 / 2 = 2"),
            _ => panic!("Ожидался тип Point::Int"),
        }
    }
    #[test]
    fn test_fndiv_type_promotion() {
        let in1 = MockNode::new("in1", Ok(Some(FnFlow::New(mock_point_int("in1", 5)))));
        let in2 = MockNode::new("in2", Ok(Some(FnFlow::New(mock_point_real("in2", 2.0)))));
        let mut div = FnDiv::new("test", vec![in1, in2]).unwrap();
        let res = div.out().unwrap().unwrap();
        match res.into_value() {
            Point::Real(p) => assert_eq!(p.value, 2.5, "Тип должен повыситься до Real (2.5)"),
            _ => panic!("Ожидался тип Point::Real"),
        }
    }
    #[test]
    fn test_fndiv_cold_mode() {
        let in1 = MockNode::new("in1", Ok(Some(FnFlow::New(mock_point_int("in1", 10)))));
        let in2 = MockNode::new_none("in2");
        let mut div = FnDiv::new("test", vec![in1, in2]).unwrap();
        let res = div.out().unwrap();
        assert!(res.is_none(), "Если хотя бы один вход None (спячка), выход должен быть None");
    }
    #[test]
    fn test_fndiv_taint_tracking_old() {
        let in1 = MockNode::new("in1", Ok(Some(FnFlow::Old(mock_point_int("in1", 10)))));
        let in2 = MockNode::new("in2", Ok(Some(FnFlow::Old(mock_point_int("in2", 2)))));
        let mut div = FnDiv::new("test", vec![in1, in2]).unwrap();
        let res = div.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_)), "Если оба входа Old, выход тоже должен быть Old");
        match res.into_value() {
            Point::Int(p) => assert_eq!(p.value, 5),
            _ => panic!("Ожидался тип Point::Int"),
        }
    }
    #[test]
    fn test_fndiv_taint_tracking_mixed() {
        let in1 = MockNode::new("in1", Ok(Some(FnFlow::Old(mock_point_int("in1", 10)))));
        let in2 = MockNode::new("in2", Ok(Some(FnFlow::New(mock_point_int("in2", 2)))));
        let mut div = FnDiv::new("test", vec![in1, in2]).unwrap();
        let res = div.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)), "Если хотя бы один вход New, выход должен заразиться New");
    }
}
