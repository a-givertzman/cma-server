use sal_sync::services::entity::{Point, PointHlr, PointType};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::domain::FnOutRef;
use crate::services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult};
///
/// Accumulates numeric incoming Point's value
/// - if input is not numeric - returns Err
/// - if input is bool, false = 0, true = 1
#[derive(Debug)]
pub struct FnAcc {
    id: String,
    kind: FnKind,
    initial: Option<FnChange>,
    input: FnChange,
    acc: Option<Point>,
}
// 
impl FnAcc {
    ///
    /// Creates new instance of the FnAcc
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, initial: Option<FnOutRef>, input: FnOutRef) -> Self {
        // let f = FnChange::new(input);
        Self { 
            id: format!("{}/FnAcc{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            initial: initial.map(|f| FnChange::new(f)),
            input: FnChange::new(input),
            acc: None,
        }
    }
    ///
    /// Возвращает `Point` с обновленными `name` и `value`  
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.timestamp())
    }
}
// 
impl FnOut for FnAcc {
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
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.inputs());
        }
        inputs.append(&mut self.input.inputs());
        inputs
    }
    ///
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let input = self.input.out();
        let initial = self.initial.as_mut().map(|f| f.out());
        let Some(input) = flow.map(input)? else { return Ok(None) };
        // trace!("{}.out | input: {:?}", self.id, input);
        let acc = match self.acc.as_ref() {
            Some(acc) => acc.clone(),
            None => {
                let acc = if let Some(initial) = initial {
                    let Some(initial) = initial? else { return Ok(None) };
                    initial.into_value()
                } else {
                    match input.type_() {
                        PointType::Bool | PointType::Int => Point::Int(Self::point_with(&input, input.name(), 0)),
                        PointType::Real => Point::Real(Self::point_with(&input, input.name(), 0.0)),
                        PointType::Double => Point::Double(Self::point_with(&input, input.name(), 0.0)),
                        _ => return Err(format!("{}.out | Invalid input type '{:?}', expected number", self.id, input.type_())),
                    }
                };
                self.acc = Some(acc.clone());
                acc
            }
        };
        if !flow.is_new() {
            return flow.wrap(acc);
        };
        let acc = match &input {
            Point::Bool(_) => acc + input.to_int(),
            _ => acc + input,
        };
        log::trace!("{}.out | out: {:?}", self.id, acc);
        self.acc = Some(acc.clone());
        flow.wrap(acc)
    }
    fn reset(&mut self) {
        if let Some(initial) = &mut self.initial {
            initial.reset();
        }
        self.acc = None;
        self.input.reset();
    }
}
///
/// Global static counter of FnAcc instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use crate::services::task::{FnFlow, FnKind, FnOut, FnResult};
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    /// Фейковый источник для полного контроля над потоком FnFlow
    #[derive(Debug)]
    struct FakeOrigin {
        next_flow: Option<FnFlow>,
        resets: usize,
    }
    impl FakeOrigin {
        fn new() -> Self { Self { next_flow: None, resets: 0 } }
        fn push(&mut self, flow: FnFlow) { self.next_flow = Some(flow); }
    }
    impl FnOut for FakeOrigin {
        fn id(&self) -> String { "fake_origin".to_string() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.next_flow.clone()) }
        fn reset(&mut self) { self.resets += 1; }
    }
    /// Хелпер для генерации тестовых точек
    fn mock_point(val: f64) -> Point {
        Point::Double(PointHlr::new(0, "test", val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    #[test]
    fn test_acc_accumulates_strictly_on_new_and_sleeps_on_old() {
        let input = Rc::new(RefCell::new(FakeOrigin::new()));
        let mut acc_node = FnAcc::new("test", None, input.clone());
        // 1. Пришел New(5.0) -> Узел проснулся, посчитал, отдал New(5.0)
        input.borrow_mut().push(FnFlow::New(mock_point(5.0)));
        let res1 = acc_node.out().unwrap().unwrap();
        assert!(matches!(res1, FnFlow::New(p) if p.as_double().value == 5.0));
        // 2. Пришел Old(5.0) -> Событий нет. Узел спит, отдает Old(5.0). Сумма НЕ меняется!
        input.borrow_mut().push(FnFlow::Old(mock_point(5.0)));
        let res2 = acc_node.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::Old(p) if p.as_double().value == 5.0));
        // 3. Пришел New(10.0) -> Новое событие. Суммируем: 5.0 + 10.0 = 15.0
        input.borrow_mut().push(FnFlow::New(mock_point(10.0)));
        let res3 = acc_node.out().unwrap().unwrap();
        assert!(matches!(res3, FnFlow::New(p) if p.as_double().value == 15.0));
    }
    #[test]
    fn test_acc_initial_parameter_bypasses_taint_tracking() {
        let initial = Rc::new(RefCell::new(FakeOrigin::new()));
        let input = Rc::new(RefCell::new(FakeOrigin::new()));
        // Initial задает стартовые 100.0
        initial.borrow_mut().push(FnFlow::New(mock_point(100.0)));
        let mut acc_node = FnAcc::new("test", Some(initial.clone()), input.clone());
        // Подаем рабочее значение 10.0
        input.borrow_mut().push(FnFlow::New(mock_point(10.0)));
        // Ожидаем, что 100.0 подхватилось без заражения графа, и сложилось с 10.0
        let res = acc_node.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(p) if p.as_double().value == 110.0));
    }
    #[test]
    fn test_acc_reset_clears_state_and_rereads_initial() {
        let initial = Rc::new(RefCell::new(FakeOrigin::new()));
        let input = Rc::new(RefCell::new(FakeOrigin::new()));
        initial.borrow_mut().push(FnFlow::New(mock_point(50.0)));
        let mut acc_node = FnAcc::new("test", Some(initial.clone()), input.clone());
        // Делаем рабочий такт: 50.0 + 10.0 = 60.0
        input.borrow_mut().push(FnFlow::New(mock_point(10.0)));
        let _ = acc_node.out().unwrap(); 
        // Аппаратный сброс
        acc_node.reset();
        assert_eq!(initial.borrow().resets, 1, "Сброс должен дойти до initial");
        assert_eq!(input.borrow().resets, 1, "Сброс должен дойти до input");
        // После сброса подаем новое значение. Узел обязан заново прочитать initial (50.0)
        input.borrow_mut().push(FnFlow::New(mock_point(5.0)));
        let res = acc_node.out().unwrap().unwrap();
        // Результат должен быть 50.0 + 5.0 = 55.0 (а не 60 + 5)
        assert!(matches!(res, FnFlow::New(p) if p.as_double().value == 55.0));
    }
}
