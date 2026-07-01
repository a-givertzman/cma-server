use std::sync::atomic::{AtomicUsize, Ordering};
use concat_string::concat_string;
use function_name::named;
use sal_core::error::Error;
use sal_sync::services::entity::{Point, PointHlr, PointType};
use sal_sync::services::types::Bool;
use crate::domain::{FnOutRef, TryTo};
use crate::err_pass;
use crate::services::task::{FlowContext, FnChange, FnFlow, FnKind, FnOut, FnResult};
///
/// ### Function | `FnMin`
/// 
/// Вычисляет минимальное значение (Min) входного сигнала.
/// 
/// Особенности работы:
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `reset`: Сбрасывает накопленную сумму и счетчик если `> 0`.
/// - `input`: Источник входных данных. Выходной `Point` автоматически наследует 
///   тип данных входа (Bool, Int, Real или Double).
/// - Игнорирует нечисловые типы (возвращает `Err`).
#[derive(Debug)]
pub struct FnMin {
    id: String,
    kind: FnKind,
    reset: Option<FnOutRef>,
    input: FnChange,
    min: Option<Point>,
}
//
impl FnMin {
    ///
    /// Creates new instance of the `FnMin`
    /// * `parent` - Идентификатор родительского узла.
    /// * `reset` - Входной сигнал для сброса минимума (опциональный).
    /// * `input` - Входной числовой сигнал.
    #[allow(dead_code)]
    pub fn new(parent: impl Into<String>, reset: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnMin{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            reset,
            input: FnChange::new(input),
            min: None,
        }
    }
    ///
    /// Возвращает `PointHlr` с обновленными `name` и `value`
    #[inline]
    fn point_with<T>(p: &Point, name: impl Into<String>, value: T) -> PointHlr<T> {
        PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts())
    }
    ///
    /// Возвращает `Point` с обновленными `name` и `value` сохраняя тип
    #[inline]
    fn point(id: &str, input: &Point, val: f64) -> Result<Point, String> {
        match input.typ() {
            PointType::Bool => Ok(Point::Bool(Self::point_with(input, id, Bool(val != 0.0)))),
            PointType::Int => Ok(Point::Int(Self::point_with(input, id, val.round() as i64))),
            PointType::Real => Ok(Point::Real(Self::point_with(input, id, val as f32))),
            PointType::Double => Ok(Point::Double(Self::point_with(input, id, val))),
            _ => Err(concat_string!(id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        }
    }
}
// 
impl FnOut for FnMin {
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
            inputs.append(&mut reset.borrow().inputs());
        }
        inputs
    }
    //
    #[named]
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let mut is_reset = false;
        let input = self.input.out();
        let reset = self.reset.as_mut().map(|f| f.borrow_mut().out());
        if let Some(reset) = reset {
            if let Some(reset) = flow.ignore(reset)? {
                let reset: bool = (&reset).try_to().map_err(|err: Error| concat_string!(self.id, ".out | Invalid reset ", err.to_string()))?;
                if reset {
                    self.min = None;
                    is_reset = true;
                }
            }
        }
        let Some(input) = flow.map(input)? else { return Ok(None) };
        if !flow.is_new() && !is_reset {
            let Some(min) = self.min.as_ref() else { return Ok(None) };
            return flow.wrap_old(min.clone());
        }
        let value = match input.typ() {
            PointType::Bool | PointType::Int | PointType::Real | PointType::Double => input.to_double().as_double().value,
            _ => return Err(concat_string!(self.id, ".out | Invalid input type '", input.typ().to_string(), "'")),
        };
        let (is_changed, point) = if let Some(prev) = self.min.as_ref() {
            if value < prev.to_double().as_double().value || prev.status() != input.status() {
                let p = Self::point(&self.id, &input, value).map_err(|err| err_pass!(self.id, err).to_string())?;
                self.min = Some(p.clone());
                (true, p)
            } else {
                (false, prev.clone())
            }
        } else { 
            let p = Self::point(&self.id, &input, value).map_err(|err| err_pass!(self.id, err).to_string())?;
            self.min = Some(p.clone());
            (true, p)
        };
        log::trace!("{}.out | min: {:?}", self.id, self.min);
        if is_changed {
            flow.wrap_new(point)
        } else {
            flow.wrap_old(point)
        }
    }
    //
    fn hard_reset(&mut self) {
        self.min = None;
        self.input.hard_reset();
        if let Some(reset) = &mut self.reset {
            reset.borrow_mut().hard_reset();
        }
    }
    //
    fn reset(&mut self) {
        self.min = None;
    }
}
///
/// Global static counter of FnMin instances
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
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }
    fn mock_double(v: f64) -> Point {
        Point::Double(PointHlr::new(0, "test", v, Status::Ok, Cot::Inf, chrono::offset::Utc::now()))
    }
    fn mock_bool(v: bool) -> Point {
        Point::Bool(PointHlr::new(0, "test", Bool(v), Status::Ok, Cot::Inf, chrono::offset::Utc::now()))
    }
    #[test]
    fn test_fnmin_accumulates_and_holds() {
        let input = Rc::new(RefCell::new(MockNode { flow: Some(FnFlow::New(mock_double(20.0))) }));
        let mut min_node = FnMin::new("test", None, input.clone());
        let res1 = min_node.out().unwrap().unwrap();
        assert_eq!(res1.value().as_double().value, 20.0);
        assert!(matches!(res1, FnFlow::New(_)));
        // Подаем меньшее значение, пробиваем дно
        input.borrow_mut().flow = Some(FnFlow::New(mock_double(15.0)));
        let res2 = min_node.out().unwrap().unwrap();
        assert_eq!(res2.value().as_double().value, 15.0); 
        assert!(matches!(res2, FnFlow::New(_)), "Пробитие дна обязано генерировать New");
        // Подаем большее значение
        input.borrow_mut().flow = Some(FnFlow::New(mock_double(30.0)));
        let res3 = min_node.out().unwrap().unwrap();
        assert_eq!(res3.value().as_double().value, 15.0); // Минимум удержан
        assert!(matches!(res3, FnFlow::Old(_)), "Должен вернуть Old, так как минимум не пробит");
    }
    #[test]
    fn test_fnmin_bool_latch() {
        let input = Rc::new(RefCell::new(MockNode { flow: Some(FnFlow::New(mock_bool(true))) }));
        let mut min_node = FnMin::new("test", None, input.clone());
        min_node.out().unwrap(); // Захватили true (1.0)
        // Сигнал падает в false (0.0), пробивая логическое дно
        input.borrow_mut().flow = Some(FnFlow::New(mock_bool(false)));
        let res = min_node.out().unwrap().unwrap();
        assert_eq!(res.value().to_bool().as_bool().value.0, false);
        assert!(matches!(res, FnFlow::New(_)));
        // Сигнал возвращается в true, но защелка должна держать false (аварию)
        input.borrow_mut().flow = Some(FnFlow::New(mock_bool(true)));
        let res_held = min_node.out().unwrap().unwrap();
        assert_eq!(res_held.value().to_bool().as_bool().value.0, false);
        assert!(matches!(res_held, FnFlow::Old(_)));
    }
    #[test]
    fn test_fnmin_reset_edge() {
        let input = Rc::new(RefCell::new(MockNode { flow: Some(FnFlow::New(mock_double(10.0))) }));
        let reset = Rc::new(RefCell::new(MockNode { flow: Some(FnFlow::New(mock_bool(false))) }));
        let mut min_node = FnMin::new("test", Some(reset.clone()), input.clone());
        min_node.out().unwrap(); // min = 10.0
        // Передний фронт сброса на фоне старых данных
        reset.borrow_mut().flow = Some(FnFlow::New(mock_bool(true)));
        input.borrow_mut().flow = Some(FnFlow::Old(mock_double(10.0))); 
        let res_reset = min_node.out().unwrap().unwrap();
        assert!(matches!(res_reset, FnFlow::New(_)), "Сброс обязан сгенерировать New");
    }
}
