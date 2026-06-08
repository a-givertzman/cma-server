use std::sync::atomic::{AtomicUsize, Ordering};
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use crate::{
    domain::{FnOutRef, PointMeta},
    services::task::{
        FlowContext, FnFlow, FnKind, FnOut, FnResult
    },
};
///
/// ### Function | `FnMul`
/// 
/// Возвращает произведение всех входов,
/// динамически повышая тип данных до наиболее точного.
/// 
/// **Example**
/// ```yaml
/// fn Mul:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Mul:
///     in1: point bool '/App/Service/Point.Name1'
///     in2: point bool '/App/Service/Point.Name2'
///     in3: point bool '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnMul {
    txid: usize,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
    id: String,
}
// 
impl FnMul {
    ///
    /// Returns `FnMul` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnMul{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        if inputs.len() < 1 {
            return Err(Error::new(&id, "new").err("At least one input must be specified"));
        }
        Ok(Self { 
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            inputs,
            id
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
impl FnOut for FnMul { 
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
        let mut has_real = false;
        let mut has_double = false;
        let mut i64_value = 1;
        let mut f32_value = 1.0;
        let mut f64_value = 1.0;
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            match input {
                Point::Bool(p) => i64_value *= p.value.0 as i64,
                Point::Int(p) => i64_value *= p.value,
                Point::Real(p) => {
                    if p.value.is_nan() {
                        return Err(format!("{}.out | Invalid input '{}': NAN, expected bool or number", self.id, p.name));
                    }
                    has_real = true;
                    f32_value *= p.value;
                }
                Point::Double(p) => {
                    if p.value.is_nan() {
                        return Err(format!("{}.out | Invalid input '{}': NAN, expected bool or number", self.id, p.name));
                    }
                    has_double = true;
                    f64_value *= p.value;
                }
                Point::String(_) | Point::Bytes(_) => return Err(format!("{}.out | Invalid input type '{:?}', expected bool or number", self.id, input.type_())),
            }
        }
        if has_double {
            flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, i64_value as f64 * f32_value as f64 * f64_value)))
        } else if has_real {
            flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, i64_value as f32 * f32_value)))
        } else {
            flow.wrap(Point::Int(Self::point_with(self.txid, &meta, &self.id, i64_value)))
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
/// Global static counter of FnMul instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use sal_sync::services::entity::{Cot, Status};
use sal_sync::services::types::Bool;
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;
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
    fn test_mul_type_promotion_and_math() {
        let id= "test_mul_type_promotion";
        let flow = FlowContext::new();
        let in1 = MockNode::new(id, flow.wrap_new(mock_point_int("i1", 2)));
        let in2 = MockNode::new(id, flow.wrap_new(mock_point_real("i2", 3.5)));
        let mut mul = FnMul::new("Test", vec![in1.clone(), in2.clone()]).unwrap();
        let result = mul.out().unwrap().unwrap();
        // Ожидаем Real (f32) равный 2 * 3.5 = 7.0
        match result.into_value() {
            Point::Real(p) => assert_eq!(p.value, 7.0),
            _ => panic!("Expected Real type promotion"),
        }
    }
    #[test]
    fn test_mul_bool_gate_zeroing() {
        let id= "test_mul_bool_gate_zeroing";
        let flow = FlowContext::new();
        let in1 = MockNode::new(id, flow.wrap_new(mock_point_double("i1", 100.5,)));
        let in2 = MockNode::new(id, flow.wrap_new(mock_point_bool("i2", false.into())));
        let mut mul = FnMul::new("Test", vec![in1.clone(), in2.clone()]).unwrap();
        let result = mul.out().unwrap().unwrap();
        // Ожидаем обнуление из-за false
        match result.into_value() {
            Point::Double(p) => assert_eq!(p.value, 0.0),
            _ => panic!("Expected Double type with 0.0 value"),
        }
    }
    #[test]
    fn test_mul_nan_rejection() {
        let id= "test_mul_nan_rejection";
        let flow = FlowContext::new();
        let in1 = MockNode::new(id, flow.wrap_new(mock_point_real("i1", f32::NAN)));
        let mut mul = FnMul::new("Test", vec![in1.clone()]).unwrap();
        // Ожидаем ошибку при попытке переварить NaN
        assert!(mul.out().is_err());
    }
    #[test]
    fn test_mul_cold_mode_propagation() {
        let id= "test_mul_cold_mode_propagation";
        let flow = FlowContext::new();
        let in1 = MockNode::new_none(id); // Обрыв / Cold Me
        let in2 = MockNode::new(id, flow.wrap_new(mock_point_int("i2", 5)));
        let mut mul = FnMul::new("Test", vec![in1.clone(), in2.clone()]).unwrap();
        // Если хотя бы один вход None, весь узел отдает None
        assert!(mul.out().unwrap().is_none());
    }
}
