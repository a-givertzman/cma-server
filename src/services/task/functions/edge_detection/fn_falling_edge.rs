use concat_string::concat_string;
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, Status}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{Edge, EdgeDetector, FnOutRef, TryTo},
    services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnFallingEdge`
/// 
/// Детектор отрицательного (заднего) фронта
/// 
/// - `input`: Последовательность `true -> false` - активирует выход на один такт
#[derive(Debug)]
pub struct FnFallingEdge {
    id: String,
    kind: FnKind,
    input: FnChange,
    edge: EdgeDetector,
    prev: Option<(bool, Status)>,
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
            prev: None,
        }
    }    
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
}
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
            self.prev = None;
            return Ok(None);
        };
        let val: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let value = match self.edge.add(val) {
            Some(Edge::Falling) => true,
            _ => false,
        };
        let status = input.status();
        let is_changed = self.prev.map_or(
            true,
            |(prev_value, prev_status)| prev_value != value || prev_status != status
        );
        self.prev = Some((value, status));
        let point = Point::Bool(Self::point_with(&input, &self.id, Bool(value)));
        // log::trace!("{}.out | value: {:#?}", self.id, point);
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn hard_reset(&mut self) {
        self.edge.reset();
        self.prev = None;
        self.input.hard_reset();
    }
    //
    fn reset(&mut self) {
        self.edge.reset();
        self.prev = None;
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
        /// Устанавливает кастомный поинт для проверки статусов и ошибок
        fn set_custom_flow(&mut self, point: Point, is_new: bool) {
            self.flow = Some(if is_new { FnFlow::New(point) } else { FnFlow::Old(point) });
        }
    }
    impl FnOut for MockInput {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.flow.clone()) }
        fn hard_reset(&mut self) { self.flow = None; }
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
        assert!(matches!(out, FnFlow::New(_)), "Первое значение должно быть New \nresult: {:?} \ntarget: FnFlow::New(_)", out);
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
        edge_node.hard_reset();
        // Подаем false. Так как состояние было сброшено, узел не должен воспринять 
        // это как спад с true на false. Он воспримет это как инициализацию false.
        mock.borrow_mut().set_flow(false, true);
        let out = edge_node.out().unwrap().unwrap();
        assert_eq!(extract_bool(&out), false, "После reset задний фронт не должен детектироваться");
    }
    #[test]
    fn test_status_change_forces_new() {
        let mock = Rc::new(RefCell::new(MockInput::new("in1")));
        let mut edge_node = FnFallingEdge::new("parent", mock.clone());
        mock.borrow_mut().set_flow(false, true);
        let _ = edge_node.out().unwrap();
        let invalid_point = Point::Bool(PointHlr::new(0, "in1", Bool(false), Status::Invalid, Cot::Inf, chrono::Utc::now()));
        mock.borrow_mut().set_custom_flow(invalid_point, true);
        let out = edge_node.out().unwrap().unwrap();
        assert!(matches!(out, FnFlow::New(_)), "Изменение статуса на Invalid должно генерировать New");
        match out {
            FnFlow::New(p) => assert_eq!(p.status(), Status::Invalid),
            _ => unreachable!(),
        }
    }
    #[test]
    fn test_cascades_error_on_invalid_type() {
        let mock = Rc::new(RefCell::new(MockInput::new("in1")));
        let mut edge_node = FnFallingEdge::new("parent", mock.clone());
        let int_point = Point::String(PointHlr::new(0, "in1", "42".into(), Status::Ok, Cot::Inf, chrono::Utc::now()));
        mock.borrow_mut().set_custom_flow(int_point, true);
        let err = edge_node.out();
        assert!(matches!(err, Err(_)), "Ожидается ошибка типа \nresult: {:?} \ntarget: Err(_)", err);
        assert!(err.unwrap_err().contains("Invalid input"), "Ожидается ошибка типа");
    }
}
