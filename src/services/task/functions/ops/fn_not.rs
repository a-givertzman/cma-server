use concat_string::concat_string;
use sal_sync::services::{entity::{Point, PointHlr, PointType}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::FnOutRef,
    services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult}
};
///
/// ### Function | Not
///
/// Логическое отрицание. Инвертирует входное значение.
/// Если на входе `true` (или число > 0), на выходе `false`.
/// Если на входе `false` (или число 0), на выходе `true`.
///
/// - Возвращает: `Bool`
/// - Наследует `status`, `timestamp` и `txid` от входной точки.
///
/// **Example**
/// ```yaml
/// fn Not:
///     input: point int '/App/Service/Point.Name1'
/// fn Not:
///     input: point bool '/App/Service/Point.Name1'
/// ```
#[derive(Debug)]
pub struct FnNot {
    id: String,
    kind: FnKind,
    input: FnOutRef,
}
// 
impl FnNot {
    ///
    /// Creates new instance of `FnNot`
    /// * `parent` - Идентификатор родительского узла графа.
    /// * `input` - Входной сигнал для инверсии.
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnNot{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
        }
    }
}
//
impl FnOut for FnNot {
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
        let value = match input.type_() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_bool().as_bool().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        };
        flow.wrap(Point::Bool(PointHlr::new(
            input.txid(),
            &self.id,
            !value,
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
/// Global static counter of FnNot instances
pub static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::task::{FnFlow, FnKind, FnOut};
    use sal_sync::services::entity::{Point, PointHlr, Status, Cot};
    use std::cell::RefCell;
    use std::rc::Rc;
    #[derive(Debug)]
    struct MockInput {
        flow: Option<FnFlow>,
    }
    impl FnOut for MockInput {
        fn id(&self) -> String { "Mock".into() }
        fn kind(&self) -> FnKind { FnKind::Input }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.flow.clone()) }
        fn reset(&mut self) {}
    }
    #[test]
    fn test_fnnot_logic_and_taint_tracking() {
        let mock = Rc::new(RefCell::new(MockInput { flow: None }));
        let mut fn_not = FnNot::new("Test", mock.clone());
        // 1. Проверка режима Cold: источник отключен
        assert!(fn_not.out().unwrap().is_none(), "Должен пробрасывать тишину (None)");
        // 2. Инверсия New(true) -> New(false)
        let ts = chrono::offset::Utc::now();
        mock.borrow_mut().flow = Some(FnFlow::New(Point::Bool(PointHlr::new(1, "src", Bool(true), Status::Ok, Cot::Inf, ts))));
        let res = fn_not.out().unwrap().unwrap();
        assert!(res.is_new(), "Статус New должен сохраняться");
        assert_eq!(res.value().as_bool().value.0, false, "Логика должна инвертироваться");
        assert_eq!(res.value().timestamp(), ts, "Таймстемп должен сохраняться");
        // 3. Инверсия Old(false) -> Old(true) (Проверка гигиены FlowContext)
        mock.borrow_mut().flow = Some(FnFlow::Old(Point::Bool(PointHlr::new(2, "src", Bool(false), Status::Ok, Cot::Inf, ts))));
        let res = fn_not.out().unwrap().unwrap();
        assert!(!res.is_new(), "Статус Old не должен заражаться паразитным New");
        assert_eq!(res.value().as_bool().value.0, true, "Логика должна инвертироваться");
    }
}
