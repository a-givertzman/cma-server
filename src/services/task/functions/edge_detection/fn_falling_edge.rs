use concat_string::concat_string;
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{Edge, EdgeDetector, FnOutRef, TryTo},
    services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnFallingEdge`
/// 
/// Детектор отричательного (заднего) фронта
/// 
/// - `input`: Последовательность `true -> false` - активирует выход на один такт
#[derive(Debug)]
pub struct FnFallingEdge {
    id: String,
    kind: FnKind,
    input: FnChange,
    edge: EdgeDetector,
    value: EdgeDetector,
}
//
impl FnFallingEdge {
    ///
    /// Returns `FnFallingEdge` new instance
    /// - `input`: Последовательность `true -> false` - активирует выход на один такт
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnFallingEdge{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input: FnChange::new(input),
            edge: EdgeDetector::new(),
            value: EdgeDetector::new(),
        }
    }    
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.timestamp())
    }
}
//
// 
impl FnOut for FnFallingEdge { 
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
        let input = self.input.out();
        let flow = FlowContext::new();
        let Some(input) = flow.ignore(input)? else {
            self.edge.reset();
            return Ok(None);
        };
        let val: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let value = match self.edge.add(val) {
            Some(Edge::Falling) => true,
            _ => false,
        };
        let is_changed = self.value.add(value).is_some();
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value)));
        // log::trace!("{}.out | value: {:#?}", self.id, point);
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn reset(&mut self) {
        self.edge.reset();
        self.value.reset();
        self.input.reset();
    }
}
///
/// Global static counter of FnFallingEdge instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    use sal_sync::services::types::Bool;
    use std::cell::RefCell;
    use std::rc::Rc;
    /// Простая заглушка для имитации входящего потока данных
    #[derive(Debug)]
    struct MockInput {
        id: String,
        flow: Option<FnFlow>,
    }
    impl MockInput {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                flow: None,
            }
        }
        /// Устанавливает следующее значение, которое отдаст узел
        fn set_flow(&mut self, value: bool, is_new: bool) {
            // Timestamp и txid здесь условные, так как мы проверяем логику фронта
            let point = Point::Bool(PointHlr::new(0, &self.id, Bool(value), Status::Ok, Cot::Inf, chrono::Utc::now()));
            self.flow = Some(if is_new {
                FnFlow::New(point)
            } else {
                FnFlow::Old(point)
            });
        }
    }
    impl FnOut for MockInput {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.flow.clone()) }
        fn reset(&mut self) { self.flow = None; }
    }
    // Хелпер для быстрого извлечения значения из FnFlow
    fn extract_bool(flow: &FnFlow) -> bool {
        match flow {
            FnFlow::New(Point::Bool(p)) | FnFlow::Old(Point::Bool(p)) => p.value.0,
            _ => panic!("Expected Point::Bool"),
        }
    }
    #[test]
    fn test_falling_edge_pulse_generation() {
        let mock = Rc::new(RefCell::new(MockInput::new("in1")));
        let mut edge_node = FnFallingEdge::new("parent", mock.clone());
        // Такт 1: Инициализация значением false
        mock.borrow_mut().set_flow(false, true);
        let out = edge_node.out().unwrap().unwrap();
        assert!(matches!(out, FnFlow::New(_)), "Первое значение должно быть New");
        assert_eq!(extract_bool(&out), false);
        // Такт 2: Переход в true (передний фронт) -> импульса быть не должно
        mock.borrow_mut().set_flow(true, true);
        let out = edge_node.out().unwrap().unwrap();
        assert!(matches!(out, FnFlow::Old(_)), "Значение не изменилось, должно быть Old");
        assert_eq!(extract_bool(&out), false);
        // Такт 3: Переход в false (задний фронт) -> ДОЛЖЕН БЫТЬ ИМПУЛЬС
        mock.borrow_mut().set_flow(false, true);
        let out = edge_node.out().unwrap().unwrap();
        assert!(matches!(out, FnFlow::New(_)), "Сгенерирован импульс, статус должен быть New");
        assert_eq!(extract_bool(&out), true);
        // Такт 4: Стабилизация в false -> снятие импульса
        // Важно: на входе данные "старые" (Old), но узел должен выдать New(false) 
        // для сброса своего импульса
        mock.borrow_mut().set_flow(false, false);
        let out = edge_node.out().unwrap().unwrap();
        assert!(matches!(out, FnFlow::New(_)), "Сброс импульса, статус должен быть New");
        assert_eq!(extract_bool(&out), false);
        // Такт 5: Продолжение false -> тишина
        mock.borrow_mut().set_flow(false, false);
        let out = edge_node.out().unwrap().unwrap();
        assert!(matches!(out, FnFlow::Old(_)), "Тишина, статус должен быть Old");
        assert_eq!(extract_bool(&out), false);
    }
    #[test]
    fn test_ignore_empty_input() {
        let mock = Rc::new(RefCell::new(MockInput::new("in1")));
        let mut edge_node = FnFallingEdge::new("parent", mock.clone());
        // Если входящий поток пуст (None)
        let out = edge_node.out().unwrap();
        assert!(out.is_none(), "Если на входе None, на выходе должно быть None");
    }
    #[test]
    fn test_reset_clears_internal_state() {
        let mock = Rc::new(RefCell::new(MockInput::new("in1")));
        let mut edge_node = FnFallingEdge::new("parent", mock.clone());
        // Взводим триггер
        mock.borrow_mut().set_flow(true, true);
        let _ = edge_node.out().unwrap();
        // Сбрасываем узел
        edge_node.reset();
        // Подаем false. Так как состояние было сброшено, узел не должен воспринять 
        // это как спад с true на false. Он воспримет это как инициализацию false.
        mock.borrow_mut().set_flow(false, true);
        let out = edge_node.out().unwrap().unwrap();
        assert_eq!(extract_bool(&out), false, "После reset задний фронт не должен детектироваться");
    }
}
