use concat_string::concat_string;
use sal_core::error::Error;
use sal_sync::services::{entity::{Point, PointHlr, Status}, types::Bool};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{
    domain::{Edge, EdgeDetector, FnOutRef, TryTo},
    services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult},
};
///
/// ### Function | `FnRisingEdge`
/// 
/// Детектор положительного (переднего) фронта
/// 
/// - `input`: Последовательность `false -> true` - активирует выход на один такт
#[derive(Debug)]
pub struct FnRisingEdge {
    id: String,
    kind: FnKind,
    input: FnChange,
    edge: EdgeDetector,
    prev: Option<(bool, Status)>,
}
//
impl FnRisingEdge {
    ///
    /// Returns `FnRisingEdge` new instance
    /// - `input`: Последовательность `false -> true` - активирует выход на один такт
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnRisingEdge{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
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
impl FnOut for FnRisingEdge { 
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
        let status = input.status();
        let val: bool = (&input).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid input ", err.to_string()))?;
        let value = match self.edge.add(val) {
            Some(Edge::Rising) => true,
            _ => false,
        };
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
/// Global static counter of FnRisingEdge instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Test
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use chrono::Utc;
    use sal_sync::services::entity::{Status, Cot};
    /// Вспомогательная заглушка для имитации входных данных графа
    #[derive(Debug)]
    struct MockEdgeInput {
        queue: Vec<FnResult<FnFlow, String>>,
    }
    impl FnOut for MockEdgeInput {
        fn id(&self) -> String {
            "mock_input".to_string()
        }
        fn kind(&self) -> FnKind {
            FnKind::Fn
        }
        fn inputs(&self) -> Vec<String> {
            vec![]
        }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            if self.queue.is_empty() {
                Ok(None)
            } else {
                self.queue.remove(0)
            }
        }
        fn hard_reset(&mut self) {
            self.queue.clear();
        }
        fn reset(&mut self) {
            self.queue.clear();
        }
    }
    /// Хелпер для быстрой сборки булевой точки
    fn create_test_point(val: bool) -> Point {
        Point::Bool(PointHlr::new(1, "source", Bool(val), Status::Ok, Cot::Inf, Utc::now()))
    }
    #[test]
    fn test_rising_edge_pulse_and_hold() {
        let mock = Rc::new(RefCell::new(MockEdgeInput {
            queue: vec![
                Ok(Some(FnFlow::New(create_test_point(false)))), // 1. Исходное состояние
                Ok(Some(FnFlow::New(create_test_point(true)))),  // 2. Положительный фронт
                Ok(Some(FnFlow::Old(create_test_point(true)))),  // 3. Сигнал удерживается (тишина)
                Ok(Some(FnFlow::New(create_test_point(false)))), // 4. Спад сигнала
            ]
        }));
        let mut detector = FnRisingEdge::new("test_parent", mock.clone() as FnOutRef);
        // Такт 1: Инициализация. Первый запуск всегда New, так как кэш пуст.
        let res1 = detector.out().unwrap().unwrap();
        assert!(res1.is_new(), "Первый запуск (инициализация) всегда должен генерировать New");
        if let Point::Bool(p) = res1.value() {
            assert_eq!(p.value.0, false, "Значение должно быть false");
        }
        // Такт 2: Прилетел true! Ловим передний фронт
        let res2 = detector.out().unwrap().unwrap();
        assert!(res2.is_new(), "Фронт обязан сгенерировать событие New");
        if let Point::Bool(p) = res2.value() {
            assert_eq!(p.value.0, true, "На фронте значение импульса должно быть true");
        }
        // Такт 3: На входе всё еще true (Old). Импульс должен сброситься в false!
        let res3 = detector.out().unwrap().unwrap();
        assert!(res3.is_new(), "Сброс импульса обязан принудительно выдать событие New для SCADA");
        if let Point::Bool(p) = res3.value() {
            assert_eq!(p.value.0, false, "Импульс должен упасть в false");
        }
        // Такт 4: Сигнал упал в false. Состояние выхода не изменилось.
        let res4 = detector.out().unwrap().unwrap();
        assert!(!res4.is_new(), "Спад сигнала после сброшенного импульса не должен генерировать New");
    }
    #[test]
    fn test_rising_edge_cold_mode_reset() {
        let mock = Rc::new(RefCell::new(MockEdgeInput {
            queue: vec![
                Ok(Some(FnFlow::New(create_test_point(true)))),  // Взводим
                Ok(None),                                        // Обрыв связи / Отключение по FnEnable
                Ok(Some(FnFlow::New(create_test_point(true)))),  // Снова взводим
            ]
        }));
        let mut detector = FnRisingEdge::new("test_parent", mock.clone() as FnOutRef);
        let _ = detector.out().unwrap();
        // Проверяем реакцию на режим Cold Mode
        let res_none = detector.out().unwrap();
        assert!(res_none.is_none(), "В режиме Cold Mode узел обязан вернуть None и остановить поток данных");
        // После сброса новое появление true снова должно отработаться как чистый фронт
        let res_restart = detector.out().unwrap().unwrap();
        assert!(res_restart.is_new(), "После сна узел должен корректно отработать новый фронт");
    }
    #[test]
    fn test_rising_edge_status_change() {
        let mock = Rc::new(RefCell::new(MockEdgeInput {
            queue: vec![
                Ok(Some(FnFlow::New(create_test_point(false)))),
                Ok(Some(FnFlow::New(Point::Bool(PointHlr::new(1, "source", Bool(false), Status::Invalid, Cot::Inf, Utc::now()))))),
            ]
        }));
        let mut detector = FnRisingEdge::new("test_parent", mock.clone() as FnOutRef);
        let _ = detector.out().unwrap(); // Инициализация
        let res = detector.out().unwrap().unwrap();
        assert!(res.is_new(), "Смена статуса качества обязана генерировать New, даже если физическое значение спит");
        assert_eq!(res.value().status(), Status::Invalid);
    }
}
