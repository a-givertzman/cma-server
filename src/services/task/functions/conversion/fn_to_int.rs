use sal_sync::services::{entity::{Point, PointHlr, PointType}, types::DebugTypeOf};
use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// ### Function | FnToInt
/// 
/// Converts input to Int
///  - bool: true -> 1, false -> 0
///  - real: 0.1 -> 0 | 0.5 -> 1 | 0.9 -> 1 | 1.1 -> 1
///  - string: try to parse int
#[derive(Debug)]
pub struct FnToInt {
    id: String,
    kind: FnKind,
    input: FnOutRef,
}
// 
impl FnToInt {
    ///
    /// Creates new instance of the `FnToInt`
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnToInt{}", parent.into(), COUNT.fetch_add(1, Ordering::SeqCst)),
            kind: FnKind::Fn,
            input,
        }
    }    
}
// 
impl FnOut for FnToInt { 
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
        self.input.borrow().inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        log::trace!("{}.out | input: {:#?}", self.id, input);
        let value: i64 = match &input {
            Point::Bool(_) | Point::Int(_) | Point::Real(_) | Point::Double(_) => input.to_int().as_int().value,
            Point::String(val) => {
                val.value.parse()
                    .map_err(|_| concat_string!(self.id, ".out | Invalid input '", val.value, "'"))?
            }
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        };
        flow.wrap(Point::Int(PointHlr::new(
            input.txid(),
            &self.id,
            value,
            input.status(),
            input.cot(),
            input.timestamp(),
        )))
    }
    //
    fn reset(&mut self) {
        self.input.borrow_mut().reset();
    }
}
///
/// Global static counter of FnToInt instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use sal_sync::services::{entity::{Cot, PointHlr, Status}, types::Bool};
    use std::{cell::RefCell, rc::Rc};
    #[derive(Debug)]
    struct MockInput {
        flow: FnFlow,
    }
    impl MockInput {
        pub fn set(&mut self, v: FnFlow) {
            self.flow = v;
        }
    }
    impl FnOut for MockInput {
        fn id(&self) -> String { "Mock".into() }
        fn kind(&self) -> FnKind { FnKind::Input }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(Some(self.flow.clone())) }
        fn reset(&mut self) {}
    }
    fn setup_node(input: Point) -> (FnToInt, Rc<RefCell<MockInput>>) {
        let producer = Rc::new(RefCell::new(MockInput { flow: FnFlow::New(input) }));
        let node = FnToInt::new("TestApp", producer.clone());
        (node, producer)
    }
    #[test]
    fn test_rounding_and_types() {
        let ts = chrono::offset::Utc::now();
        // Проверяем Real: 0.9 -> 1
        let pt_real = Point::Real(PointHlr::new(1, "In", 0.9, Status::Ok, Cot::Inf, ts));
        let (mut node, _) = setup_node(pt_real);
        let out = node.out().unwrap().unwrap();
        assert!(out.is_new());
        assert_eq!(out.into_value().as_int().value, 1, "0.9 must be rounded to 1");
        // Проверяем Real: 0.1 -> 0
        let pt_real2 = Point::Real(PointHlr::new(2, "In", 0.1, Status::Ok, Cot::Inf, ts));
        let (mut node, _) = setup_node(pt_real2);
        assert_eq!(node.out().unwrap().unwrap().into_value().as_int().value, 0);
        // Проверяем Bool: true -> 1
        let pt_bool = Point::Bool(PointHlr::new(3, "In", Bool(true), Status::Ok, Cot::Inf, ts));
        let (mut node, _) = setup_node(pt_bool);
        assert_eq!(node.out().unwrap().unwrap().into_value().as_int().value, 1);
    }
    #[test]
    fn test_taint_tracking_old_flow() {
        let ts = chrono::offset::Utc::now();
        let pt = Point::Int(PointHlr::new(1, "In", 42, Status::Ok, Cot::Inf, ts));
        let (mut node, producer) = setup_node(pt);
        // Первый такт - ожидаем New
        let out_new = node.out().unwrap().unwrap();
        assert!(out_new.is_new());
        // Переводим источник в Old (тишина)
        producer.borrow_mut().set(FnFlow::Old(Point::Int(PointHlr::new(1, "In", 42, Status::Ok, Cot::Inf, ts))));
        // Второй такт - ожидаем Old
        let out_old = node.out().unwrap().unwrap();
        assert!(!out_old.is_new(), "Pure node must propagate Old status");
        assert_eq!(out_old.into_value().as_int().value, 42);
    }
    #[test]
    fn test_string_parsing() {
        let ts = chrono::offset::Utc::now();
        // Успешный парсинг
        let pt_str = Point::String(PointHlr::new(1, "In", "123".to_string(), Status::Ok, Cot::Inf, ts));
        let (mut node_ok, _) = setup_node(pt_str);
        assert_eq!(node_ok.out().unwrap().unwrap().into_value().as_int().value, 123);
        // Ошибка парсинга (дробное число в строке)
        let pt_err = Point::String(PointHlr::new(2, "In", "12.5".to_string(), Status::Ok, Cot::Inf, ts));
        let (mut node_err, _) = setup_node(pt_err);
        assert!(node_err.out().is_err(), "Must fail on non-integer string");
    }
}
