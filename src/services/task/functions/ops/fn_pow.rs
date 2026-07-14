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
/// ### Function | `FnPow`
/// 
/// Выполняет операцию вычисления математической степени `v1 ^ v2` над входящими точками данных.
/// Динамически повышает тип данных до наиболее точного.
/// 
/// **Example**
/// ```yaml
/// fn Pow:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Pow:
///     in1: point double '/App/Service/Point.Name1'
///     in2: point double '/App/Service/Point.Name2'
#[derive(Debug)]
pub struct FnPow {
    txid: usize,
    kind: FnKind,
    inputs: [FnOutRef; 2],
    id: String,
}
// 
impl FnPow {
    ///
    /// Returns `FnPow` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать два входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnPow{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
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
impl FnOut for FnPow { 
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
        let value = v1.pow(v2).map_err(|_| format!("{}.out | Can't pow {:?} ^ {:?}", self.id, v1, v2))?;
        match value {
            NumValue::Bool(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value as i64))),
            NumValue::Int(value) => flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, value))),
            NumValue::Real(value) => {
                if value.is_nan() || value.is_infinite() {
                    return Err(format!("{}.out | Math error: NaN or Infinite result", self.id));
                }
                flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, value)))
            }
            NumValue::Double(value) => {
                if value.is_nan() || value.is_infinite() {
                    return Err(format!("{}.out | Math error: NaN or Infinite result", self.id));
                }
                flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, value)))
            }
        }
    }
    //
    fn hard_reset(&mut self) {
        for input in &self.inputs {
            input.borrow_mut().hard_reset();
        }
    }
    //
    fn reset(&mut self) {}
}
///
/// Global static counter of FnPow instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use sal_sync::services::entity::{Status, Cot};
    #[derive(Debug)]
    struct MockNode {
        id: String,
        result: FnResult<FnFlow, String>,
    }
    impl MockNode {
        fn new(id: &str, result: FnResult<FnFlow, String>) -> FnOutRef {
            Rc::new(RefCell::new(Self {
                id: id.to_string(),
                result,
            }))
        }
    }
    impl FnOut for MockNode {
        fn id(&self) -> String {
            self.id.clone()
        }
        fn kind(&self) -> FnKind {
            FnKind::Fn
        }
        fn inputs(&self) -> Vec<String> {
            vec![]
        }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.result.clone()
        }
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }
    fn mock_point_int(val: i64) -> Point {
        Point::Int(PointHlr::new(0, "test", val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    fn mock_point_real(val: f32) -> Point {
        Point::Real(PointHlr::new(0, "test", val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    fn extract_value(flow: FnResult<FnFlow, String>) -> NumValue {
        match flow.unwrap().unwrap() {
            FnFlow::New(p) | FnFlow::Old(p) => match p {
                Point::Int(hlr) => NumValue::Int(hlr.value),
                Point::Real(hlr) => NumValue::Real(hlr.value),
                Point::Double(hlr) => NumValue::Double(hlr.value),
                Point::Bool(hlr) => NumValue::Bool(hlr.value.0),
                _ => panic!("Unexpected point type"),
            },
        }
    }
    #[test]
    fn test_pow_classical_math() {
        let base = MockNode::new("base", Ok(Some(FnFlow::New(mock_point_int(2)))));
        let exp = MockNode::new("exp", Ok(Some(FnFlow::New(mock_point_int(3)))));
        let mut pow_node = FnPow::new("parent", vec![base, exp]).unwrap();
        let res = pow_node.out();
        assert!(matches!(res, Ok(Some(FnFlow::New(_)))));
        assert_eq!(extract_value(res), NumValue::Int(8));
    }
    #[test]
    fn test_pow_flow_old() {
        let base = MockNode::new("base", Ok(Some(FnFlow::Old(mock_point_int(2)))));
        let exp = MockNode::new("exp", Ok(Some(FnFlow::Old(mock_point_int(3)))));
        let mut pow_node = FnPow::new("parent", vec![base, exp]).unwrap();
        let res = pow_node.out();
        assert!(matches!(res, Ok(Some(FnFlow::Old(_)))));
        assert_eq!(extract_value(res), NumValue::Int(8));
    }
    #[test]
    fn test_pow_nan_protection() {
        let base = MockNode::new("base", Ok(Some(FnFlow::New(mock_point_real(-4.0)))));
        let exp = MockNode::new("exp", Ok(Some(FnFlow::New(mock_point_real(0.5)))));
        let mut pow_node = FnPow::new("parent", vec![base, exp]).unwrap();
        let res = pow_node.out();
        assert!(res.is_err());
    }
    #[test]
    fn test_pow_cold_mode() {
        let base = MockNode::new("base", Ok(None));
        let exp = MockNode::new("exp", Ok(Some(FnFlow::New(mock_point_int(3)))));
        let mut pow_node = FnPow::new("parent", vec![base, exp]).unwrap();
        let res = pow_node.out();
        assert!(res.unwrap().is_none());
    }
}
