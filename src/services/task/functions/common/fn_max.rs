use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use sal_sync::services::entity::{Point, PointHlr, PointType};
use sal_sync::services::types::Bool;
use crate::domain::{Edge, EdgeDetector, FnOutRef};
use crate::services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult};
///
/// ### Function | Max
/// 
/// Вычисляет максимальное значение (Max) входного сигнала.
/// 
/// Особенности работы:
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывант передачу данных (возвращает `None`).
/// - `reset`: Сбрасывает вычисленное значение по переднему фронту сигнала (переход 0 -> 1).
/// - `input`: Источник входных данных. Выходной `Point` автоматически наследует 
///   тип данных входа (Bool, Int, Real или Double).
/// - Игнорирует нечисловые типы (возвращает `Err`).
#[derive(Debug)]
pub struct FnMax {
    id: String,
    kind: FnKind,
    reset: Option<FnChange>,
    input: FnChange,
    max: Option<f64>,
    reset_edge: EdgeDetector,
}
//
// 
impl FnMax {
    ///
    /// Creates new instance of the `FnMax`
    /// * `parent` - Идентификатор родительского узла.
    /// * `reset` - Входной сигнал для сброса максимума (опциональный).
    /// * `input` - Входной числовой сигнал.
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnMax{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            reset: reset.map(FnChange::new),
            input: FnChange::new(input),
            max: None,
            reset_edge: EdgeDetector::new(),
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.timestamp())
    }
    ///
    /// Возвращает `Point` с обновленными `name` и `value` сохраняя тип
    #[inline]
    fn point(id: &str, input: &Point, val: f64) -> Result<Point, String> {
        match input.type_() {
            PointType::Bool => Ok(Point::Bool(Self::point_with(input, id, Bool(val != 0.0)))),
            PointType::Int => Ok(Point::Int(Self::point_with(input, id, val.round() as i64))),
            PointType::Real => Ok(Point::Real(Self::point_with(input, id, val as f32))),
            PointType::Double => Ok(Point::Double(Self::point_with(input, id, val))),
            _ => Err(concat_string!(id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        }
    }
}
//
// 
impl FnOut for FnMax {
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
        let mut inputs = self.input.inputs();
        if let Some(reset) = &self.reset {
            inputs.append(&mut reset.inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let mut force_recalc = false;
        let input = self.input.out();
        let reset = self.reset.as_mut().map(|f| f.out());
        if let Some(reset) = reset {
            if let Some(reset) = reset? {
                if let Some(Edge::Rising) = self.reset_edge.add(reset.into_value().to_bool().as_bool().value.0) {
                    self.max = None;
                    force_recalc = true;
                }
            }
        }
        let Some(input) = flow.map(input)? else { return Ok(None) };
        if !flow.is_new() && !force_recalc {
            let Some(max) = self.max else { return Ok(None) };
            return flow.wrap_old(Self::point(&self.id, &input, max)?);
        }
        let value = match input.type_() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.type_().to_string(), "'")),
        };
        let was_none = self.max.is_none();
        let max = *self.max.get_or_insert(value);
        if value > max {
            self.max = Some(value);
            log::trace!("{}.out | max: {:?}", self.id, self.max);
            let max = Self::point(&self.id, &input, value)?;
            flow.wrap_new(max)
        } else if was_none || force_recalc {
            log::trace!("{}.out | max: {:?}", self.id, self.max);
            let max = Self::point(&self.id, &input, max)?;
            flow.wrap_new(max)
        } else {
            log::trace!("{}.out | max: {:?}", self.id, self.max);
            let max = Self::point(&self.id, &input, max)?;
            flow.wrap_old(max)
        }
    }
    //
    fn reset(&mut self) {
        self.max = None;
        self.input.reset();
        if let Some(reset) = &mut self.reset {
            reset.reset();
        }
        self.reset_edge.reset();
    }
}
///
/// Global static counter of FnMax instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::task::{FnFlow, FnKind, FnOut, FnResult};
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    use sal_sync::services::types::Bool;
    use std::{cell::RefCell, rc::Rc};
    #[derive(Debug)]
    struct MockNode { flow: Option<FnFlow> }
    impl FnOut for MockNode {
        fn id(&self) -> String { "mock".to_string() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> { Ok(self.flow.clone()) }
        fn reset(&mut self) {}
    }
    fn mock_double(v: f64) -> Point {
        Point::Double(PointHlr::new(0, "test", v, Status::Ok, Cot::Inf, chrono::offset::Utc::now()))
    }
    fn mock_bool(v: bool) -> Point {
        Point::Bool(PointHlr::new(0, "test", Bool(v), Status::Ok, Cot::Inf, chrono::offset::Utc::now()))
    }
    #[test]
    fn test_fnmax_accumulates_and_holds() {
        let input = Rc::new(RefCell::new(MockNode { flow: Some(FnFlow::New(mock_double(10.0))) }));
        let mut max_node = FnMax::new("test", None, input.clone());
        let res1 = max_node.out().unwrap().unwrap();
        assert_eq!(res1.value().as_double().value, 10.0);
        assert!(matches!(res1, FnFlow::New(_)));
        // Подаем меньшее значение
        input.borrow_mut().flow = Some(FnFlow::New(mock_double(5.0)));
        let res2 = max_node.out().unwrap().unwrap();
        assert_eq!(res2.value().as_double().value, 10.0); // Максимум удержан
        assert!(matches!(res2, FnFlow::Old(_)), "Должен вернуть Old, так как максимум не пробит");
    }
    #[test]
    fn test_fnmax_bool_latch() {
        let input = Rc::new(RefCell::new(MockNode { flow: Some(FnFlow::New(mock_bool(false))) }));
        let mut max_node = FnMax::new("test", None, input.clone());
        max_node.out().unwrap();
        input.borrow_mut().flow = Some(FnFlow::New(mock_bool(true)));
        let res = max_node.out().unwrap().unwrap();
        assert_eq!(res.value().to_bool().as_bool().value.0, true);
        assert!(matches!(res, FnFlow::New(_)));
        // Сигнал ушел, защелка должна держать true
        input.borrow_mut().flow = Some(FnFlow::New(mock_bool(false)));
        let res_held = max_node.out().unwrap().unwrap();
        assert_eq!(res_held.value().to_bool().as_bool().value.0, true);
        assert!(matches!(res_held, FnFlow::Old(_)));
    }
    #[test]
    fn test_fnmax_reset_edge() {
        let input = Rc::new(RefCell::new(MockNode { flow: Some(FnFlow::New(mock_double(50.0))) }));
        let reset = Rc::new(RefCell::new(MockNode { flow: Some(FnFlow::New(mock_bool(false))) }));
        let mut max_node = FnMax::new("test", Some(reset.clone()), input.clone());
        max_node.out().unwrap(); // max = 50.0
        // Передний фронт сброса
        reset.borrow_mut().flow = Some(FnFlow::New(mock_bool(true)));
        input.borrow_mut().flow = Some(FnFlow::Old(mock_double(50.0))); // Данные не менялись
        let res_reset = max_node.out().unwrap().unwrap();
        assert!(matches!(res_reset, FnFlow::New(_)), "Сброс обязан сгенерировать New");
    }
}
