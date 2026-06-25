use function_name::named;
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointTxId};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{FnOutRef, PointMeta}, err, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | `FnAdd`
/// 
/// Возвращает сумму всех входов,
/// динамически повышая тип данных до наиболее точного.
/// 
/// **Example**
/// ```yaml
/// fn Add:
///     input1: point int '/App/Service/Point.Name1'
///     input2: point int '/App/Service/Point.Name2'
/// fn Add:
///     in1: point real '/App/Service/Point.Name1'
///     in2: point real '/App/Service/Point.Name2'
///     in3: point real '/App/Service/Point.Name3'
/// ```
#[derive(Debug)]
pub struct FnAdd {
    txid: usize,
    id: String,
    kind: FnKind,
    inputs: Vec<FnOutRef>,
}
//
// 
impl FnAdd {
    ///
    /// Returns `FnAdd` new instance
    /// - `parent` - Идентификатор родительского узла
    /// - `inputs` - Вектор входных сигналов, должен содержать не менее одного входа
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, inputs: Vec<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnAdd{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed));
        if inputs.len() < 1 {
            return Err(Error::new(&id, "new").err("At least one input must be specified"));
        }
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind:FnKind::Fn,
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
impl FnOut for FnAdd {
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
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut inputs = Vec::with_capacity(self.inputs.len());
        for input in self.inputs.iter() {
            inputs.push(input.borrow_mut().out());
        }
        let mut flow = FlowContext::new();
        let mut has_real = false;
        let mut has_double = false;
        let mut i64_value = 0;
        let mut f32_value = 0.0;
        let mut f64_value = 0.0;
        let mut meta = PointMeta::default();
        for input in inputs {
            let Some(input) = flow.map(input)? else { return Ok(None) };
            meta = meta.update_latest(&input).update_status(&input);
            match input {
                Point::Int(p) => i64_value = i64::checked_add(i64_value, p.value)
                    .ok_or_else(|| err!(self.id, "Overflow: `{:?} + {:?}`", i64_value, p.value).to_string())?,
                Point::Real(p) => {
                    if p.value.is_nan() {
                        return Err(format!("{}.out | Invalid input '{}': NAN, expected bool or number", self.id, p.name));
                    }
                    has_real = true;
                    f32_value += p.value;
                }
                Point::Double(p) => {
                    if p.value.is_nan() {
                        return Err(format!("{}.out | Invalid input '{}': NAN, expected bool or number", self.id, p.name));
                    }
                    has_double = true;
                    f64_value += p.value;
                }
                Point::Bool(_) | Point::String(_) | Point::Bytes(_) => return Err(format!("{}.out | Invalid input type '{:?}', expected number", self.id, input.typ())),
            }
        }
        if has_double {
            flow.wrap(Point::Double(Self::point_with(self.txid, &meta, &self.id, i64_value as f64 + f32_value as f64 + f64_value)))
        } else if has_real {
            flow.wrap(Point::Real(Self::point_with(self.txid, &meta, &self.id, i64_value as f32 + f32_value)))
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
/// Global static counter of FnAdd instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use chrono::Utc;
    use sal_sync::services::entity::{Cot, Status};
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
    fn mock_point_int(val: i64) -> Point {
        Point::Int(PointHlr::new(0, "test", val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    fn mock_point_real(val: f32) -> Point {
        Point::Real(PointHlr::new(0, "test", val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    #[test]
    fn test_add_successful_type_promotion() {
        let n1 = MockNode::new("n1", Ok(Some(FnFlow::New(Point::Int(PointHlr::new(1, "n1", 10, Status::Ok, Cot::Inf, Utc::now()))))));
        let n2 = MockNode::new("n2", Ok(Some(FnFlow::New(Point::Real(PointHlr::new(2, "n2", 5.5, Status::Ok, Cot::Inf, Utc::now()))))));
        let mut add = FnAdd::new("parent", vec![n1.clone(), n2.clone()]).unwrap();
        let res = add.out().unwrap().unwrap();
        if let FnFlow::New(Point::Real(p)) = res {
            assert_eq!(p.value, 15.5);
        } else {
            panic!("Expected Point::Real result");
        }
    }
    #[test]
    fn test_fetch_phase_polls_all_inputs_unconditionally() {
        let n1 = MockNode::new("n1", Ok(None));
        let n2 = MockNode::new("n2", Ok(Some(FnFlow::New(Point::Int(PointHlr::new(2, "n2", 5, Status::Ok, Cot::Inf, Utc::now()))))));
        let mut add = FnAdd::new("parent", vec![n1.clone(), n2.clone()]).unwrap();
        let _ = add.out();
        assert_eq!(n1.borrow().called, 1);
        assert_eq!(n2.borrow().called, 1);
    }
    #[test]
    fn test_nan_input_returns_error() {
        let n1 = MockNode::new("test_nan", Ok(Some(FnFlow::New(mock_point_real(f32::NAN)))));
        let mut adder = FnAdd::new("root", vec![n1]).unwrap();
        assert!(adder.out().is_err());
    }
}
