use std::sync::atomic::{AtomicUsize, Ordering};
use sal_sync::services::entity::{Point, PointHlr};
use crate::domain::{EdgeDetector, FnOutRef};
use crate::services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult};

///
/// ### Function | FnCount
/// 
/// Счетчик передних фронтов логического сигнала.
/// Увеличивает внутреннее значение на 1 каждый раз, когда входной сигнал 
/// меняет значение с false на true. Сохраняет состояние между тактами.
#[derive(Debug)]
pub struct FnCount {
    id: String,
    kind: FnKind,
    input: FnOutRef,
    edge: EdgeDetector,
    count: Option<i64>,
    initial: Option<FnOutRef>,
}
// 
impl FnCount {
    ///
    /// Creates new instance of the `FnCount`
    /// 
    /// * `parent` - Идентификатор родительского узла для генерации уникального ID.
    /// * `initial` - Начальное значение счетчика (Опциональный).
    /// * `input` - Входной сигнал для подсчета фронтов (bool/number).
    pub fn new(parent: impl Into<String>, initial: Option<FnOutRef>, input: FnOutRef) -> Self {
        Self { 
            id: format!("{}/FnCount{}", parent.into(), COUNT.fetch_add(1, Ordering::Relaxed)),
            kind: FnKind::Fn,
            input,
            edge: EdgeDetector::new(),
            count: None,
            initial,
        }
    }
    ///
    /// Возвращает `Point` `p` с обновленными `name` и `value`  
    fn point_with(p: &Point, name: impl Into<String>, value: i64) -> Point {
        Point::Int(PointHlr::new(p.txid(), name, value, p.status(), p.cot(), p.ts()))
    }
}
// 
impl FnOut for FnCount {
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
        inputs.append(&mut self.input.borrow().inputs());
        if let Some(initial) = &self.initial {
            inputs.append(&mut initial.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        let Some(input) = flow.map(self.input.borrow_mut().out())? else { return Ok(None) };
        let mut count = match self.count {
            Some(count) => count,
            None => {
                let val = if let Some(initial) = &self.initial {
                    let Some(initial) = initial.borrow_mut().out()? else { return Ok(None) };
                    initial.into_value().to_int().as_int().value
                } else {
                    0
                };
                self.count = Some(val);
                val
            }
        };
        if !flow.is_new() {
            return flow.wrap(Self::point_with(&input, &self.id, count));
        };
        let val = input.to_bool().as_bool().value.0;
        let Some(edge) = self.edge.add(val) else {
            return flow.wrap_old(Self::point_with(&input, &self.id,count));
        };
        if edge.is_rising() {
            count += 1;
            self.count = Some(count);
            log::trace!("{}.out | value: {:?}", self.id, count);
            flow.wrap(Self::point_with(&input, &self.id, count))
        } else {
            log::trace!("{}.out | value: {:?}", self.id, count);
            flow.wrap_old(Self::point_with(&input, &self.id, count))
        }
    }
    //
    fn hard_reset(&mut self) {
        self.count = None;
        self.edge.reset();
        if let Some(initial) = &self.initial { initial.borrow_mut().hard_reset(); };
        self.input.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {
        self.count = None;
        self.edge.reset();
    }
}
///
/// Global static counter of FnCount instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::services::task::{FnFlow, FnKind, FnOut, FnResult};
    use sal_sync::services::entity::{Point, PointHlr};
    #[derive(Debug)]
    struct FakeInput {
        point: Option<Point>,
        is_new: bool,
    }
    impl FakeInput {
        fn new() -> Rc<RefCell<Self>> {
            Rc::new(RefCell::new(Self { point: None, is_new: false }))
        }
        fn set(&mut self, value: i64, is_new: bool) {
            let p = Point::Int(PointHlr::new_int(0, "test", value));
            self.point = Some(p);
            self.is_new = is_new;
        }
    }
    impl FnOut for FakeInput {
        fn id(&self) -> String { "FakeInput".to_string() }
        fn kind(&self) -> FnKind { FnKind::Input }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            if let Some(p) = &self.point {
                if self.is_new {
                    Ok(Some(FnFlow::New(p.clone())))
                } else {
                    Ok(Some(FnFlow::Old(p.clone())))
                }
            } else {
                Ok(None)
            }
        }
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }
    #[test]
    fn test_fn_count_logic() {
        let fake_input = FakeInput::new();
        let mut fn_count = FnCount::new("root", None, fake_input.clone());
        // 1. Инициализация (Холодный старт с false)
        fake_input.borrow_mut().set(0, true); // 0 = false, status = New
        let res = fn_count.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_))); // EdgeDetector инициализируется, фронта нет
        assert_eq!(res.into_value().to_int().as_int().value, 0);
        // 2. Передний фронт (false -> true, status = New)
        fake_input.borrow_mut().set(1, true); // 1 = true
        let res = fn_count.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_))); // Фронт найден, возвращаем New
        assert_eq!(res.into_value().to_int().as_int().value, 1);
        // 3. Задний фронт (true -> false, status = New)
        fake_input.borrow_mut().set(0, true); // 0 = false
        let res = fn_count.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_))); // Спад сигнала, счетчик спит, возвращаем Old
        assert_eq!(res.into_value().to_int().as_int().value, 1);
        // 4. Повторный передний фронт (false -> true, status = New)
        fake_input.borrow_mut().set(1, true);
        let res = fn_count.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)));
        assert_eq!(res.into_value().to_int().as_int().value, 2);
        // 5. Игнорирование контекстного спама (сигнал true, но status = Old)
        // Имитируем вызов из соседней ветки, когда этот датчик не обновлялся
        fake_input.borrow_mut().set(1, false); 
        let res = fn_count.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_))); // Должен вернуть Old, так как вход не New
        assert_eq!(res.into_value().to_int().as_int().value, 2); // Значение не изменилось
    }
}