use sal_sync::services::entity::Point;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::services::task::FnChange;
use crate::{
    domain::{FnOutRef},
    services::task::{FlowContext, FnFlow, functions::{
        FnKind, FnOut, FnResult
    }},
};
///
/// ### Function | Returns last good value, was `FnKeepValid`
/// 
/// - Удерживает последнее валидное значение оперативно
/// - Если источник возвращает `None` (отключен или молчит), отдает последнее сохраненное значение как `Old`.
/// - Если значение не изменилось, но источник активен, подавляет флаг и отдает `Old`.
#[derive(Debug)]
pub struct FnHold {
    id: String,
    kind: FnKind,
    input: FnChange,
    state: Option<Point>,
}
// 
impl FnHold {
    ///
    /// Creates new instance of the FnHold
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnHold{}", parent.into(), COUNT.fetch_add(1, Ordering::AcqRel)),
            kind: FnKind::Fn,
            input: FnChange::new(input),
            state: None,
        }
    }    
}
// 
impl FnOut for FnHold { 
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
        self.input.inputs()
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = flow.map(self.input.out())?;
        log::trace!("{}.out | input: {:?}", self.id, input);
        if let Some(point) = input {
            self.state = Some(point.clone());
            return flow.wrap(point);
        }
        if let Some(point) = &self.state {
            return flow.wrap_old(point.clone());
        }
        Ok(None)
    }
    //
    fn reset(&mut self) {
        self.state = None;
        self.input.reset();
    }
}
///
/// Global static counter of FnHold instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    use crate::services::task::{FnFlow, FnKind, FnOut, FnResult};
    #[derive(Debug)]
    struct FakeOrigin {
        next_flow: FnResult<FnFlow, String>,
        resets: usize,
    }
    impl FakeOrigin {
        fn new() -> Self {
            Self { next_flow: Ok(None), resets: 0 }
        }
        fn set_flow(&mut self, flow: FnResult<FnFlow, String>) {
            self.next_flow = flow;
        }
    }
    impl FnOut for FakeOrigin {
        fn id(&self) -> String { "fake_origin".to_string() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.next_flow.clone()
        }
        fn reset(&mut self) {
            self.resets += 1;
        }
    }
    fn mock_point(val: f64) -> Point {
        Point::Double(PointHlr::new(0, "test", val, Status::Ok, Cot::Inf, chrono::offset::Utc::now()))
    }
    #[test]
    fn test_fnhold_passes_new_and_caches_it() {
        let origin = Rc::new(RefCell::new(FakeOrigin::new()));
        let mut hold = FnHold::new("parent", origin.clone());
        origin.borrow_mut().set_flow(Ok(Some(FnFlow::New(mock_point(10.0)))));
        let res = hold.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)), "Первое валидное значение должно пройти как New");
        assert_eq!(hold.state.as_ref().unwrap().value(), mock_point(10.0).value(), "Значение должно быть закэшировано");
    }
    #[test]
    fn test_fnhold_returns_cached_old_on_none() {
        let origin = Rc::new(RefCell::new(FakeOrigin::new()));
        let mut hold = FnHold::new("parent", origin.clone());
        origin.borrow_mut().set_flow(Ok(Some(FnFlow::New(mock_point(42.0)))));
        let _ = hold.out().unwrap();
        origin.borrow_mut().set_flow(Ok(None));
        let res = hold.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_)), "При None на входе FnHold должен отдать кэш как Old");
        assert_eq!(res.into_value().to_double().as_double().value, 42.0);
    }
    #[test]
    fn test_fnhold_returns_none_if_empty_and_reset() {
        let origin = Rc::new(RefCell::new(FakeOrigin::new()));
        let mut hold = FnHold::new("parent", origin.clone());
        origin.borrow_mut().set_flow(Ok(None));
        let res = hold.out().unwrap();
        assert!(res.is_none(), "Если кэш пуст и на входе None, отдаем None");
        origin.borrow_mut().set_flow(Ok(Some(FnFlow::New(mock_point(55.0)))));
        let _ = hold.out().unwrap();
        hold.reset();
        assert_eq!(origin.borrow().resets, 1, "Сброс должен пробрасываться внутрь");
        origin.borrow_mut().set_flow(Ok(None));
        let res_after_reset = hold.out().unwrap();
        assert!(res_after_reset.is_none(), "После сброса кэш очищен, при None должны вернуть None");
    }
}
