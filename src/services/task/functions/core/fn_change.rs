use crate::{domain::FnOutRef, services::task::{FnFlow, FnKind, FnOut, FnResult}};
use sal_sync::services::entity::Point;
use std::fmt::Debug;
///
/// ### Function | FnChange Decorator
/// 
/// Декоратор для контроля изменений потока данных.
/// Пропускает через себя вычисления внутреннего узла, но понижает статус `FnFlow::New` до `FnFlow::Old`,
/// если фактическое значение и статус качества точки остались неизменными.
#[derive(Debug)]
pub struct FnChange {
    input: FnOutRef,
    last_val: Option<Point>,
}
impl FnChange {
    /// Создает новый экземпляр декоратора FnChange.
    pub fn new(inp: FnOutRef) -> Self {
        Self { input: inp, last_val: None }
    }
}
impl FnOut for FnChange {
    fn id(&self) -> String { self.input.borrow().id() }
    fn kind(&self) -> FnKind { self.input.borrow().kind() }
    fn inputs(&self) -> Vec<String> { self.input.borrow().inputs() }
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let result = self.input.borrow_mut().out()?;
        match result {
            // Перехватываем всё, что содержит значение (и New, и Old)
            Some(FnFlow::New(point)) | Some(FnFlow::Old(point)) => {
                let is_changed = match &self.last_val {
                    Some(last) => {
                        // Смена статуса (например, Ok -> Invalid) так же важна, как и смена значения
                        last.value() != point.value() || last.status() != point.status()
                    },
                    None => true,
                };
                if is_changed {
                    // Значение изменилось! Принудительно генерируем New,
                    // даже если источник ошибочно или лениво прислал Old.
                    self.last_val = Some(point.clone());
                    Ok(Some(FnFlow::New(point)))
                } else {
                    // Значение старое. Принудительно гасим в Old,
                    // даже если источник спамит New.
                    Ok(Some(FnFlow::Old(point)))
                }
            }
            other => Ok(other),
        }
    }
    //
    fn hard_reset(&mut self) {
        self.last_val = None;
        self.input.borrow_mut().hard_reset();
    }
    //
    fn reset(&mut self) {
        self.last_val = None;
    }
}
///
/// Basic tests
#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

use super::*;
    use crate::services::task::{FnFlow, FnKind, FnOut, FnResult};
    use sal_sync::services::entity::{Cot, Point, PointHlr, Status};
    #[derive(Debug)]
    struct FakeOrigin {
        next_flow: Option<FnFlow>,
        hard_resets: usize,
        resets: usize,
    }
    impl FakeOrigin {
        fn new() -> Self {
            Self { next_flow: None, hard_resets: 0, resets: 0 }
        }
        fn push_new(&mut self, point: Point) {
            self.next_flow = Some(FnFlow::New(point));
        }
    }
    impl FnOut for FakeOrigin {
        fn id(&self) -> String { "fake_origin".to_string() }
        fn kind(&self) -> FnKind { FnKind::Var }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            Ok(self.next_flow.clone())
        }
        fn hard_reset(&mut self) {
            self.hard_resets += 1;
        }
        fn reset(&mut self) {
            self.resets += 1;
        }
    }
    // Вспомогательная функция для создания точек в тестах (замени на реальный конструктор Point)
    fn mock_point(val: f64, status: Status) -> Point {
        Point::Double(PointHlr::new(0, "test", val, status, Cot::Inf, chrono::offset::Utc::now()))
    }
    #[test]
    fn test_identical_values_are_muted_to_old() {
        let origin = Rc::new(RefCell::new(FakeOrigin::new()));
        // 1. Первый такт: новое значение, ожидаем New
        origin.borrow_mut().push_new(mock_point(25.5, Status::Ok));
        let mut filter = FnChange::new(origin.clone());
        let res1 = filter.out().unwrap().unwrap();
        assert!(matches!(res1, FnFlow::New(_)), "Первое событие всегда должно быть New");
        // 2. Второй такт: то же самое значение и статус
        // Внутренний узел (FakeOrigin) честно отдает New, так как он только вычисляет
        origin.borrow_mut().push_new(mock_point(25.5, Status::Ok));
        let res2 = filter.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::Old(_)), "Неизменное значение должно быть понижено до Old");
    }
    #[test]
    fn test_status_change_forces_new_flow() {
        let origin = Rc::new(RefCell::new(FakeOrigin::new()));
        // 1. Первый такт: нормальное значение
        origin.borrow_mut().push_new(mock_point(25.5, Status::Ok));
        let mut filter = FnChange::new(origin.clone());
        let _ = filter.out().unwrap();
        // 2. Второй такт: значение то же, но статус качества упал в Invalid
        origin.borrow_mut().push_new(mock_point(25.5, Status::Invalid));
        let res2 = filter.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::New(_)), "Смена статуса качества обязана генерировать New");
    }
    #[test]
    fn test_reset_clears_state_and_allows_new() {
        let origin = Rc::new(RefCell::new(FakeOrigin::new()));
        origin.borrow_mut().push_new(mock_point(10.0, Status::Ok));
        let mut filter = FnChange::new(origin.clone());
        // Прогоняем значение, оно кэшируется
        let _ = filter.out().unwrap();
        // Аппаратный сброс графа
        filter.hard_reset();
        assert_eq!(origin.borrow_mut().hard_resets, 1, "Сброс должен пробрасываться во внутренний узел");
        // Подаем то же самое значение после сброса
        origin.borrow_mut().push_new(mock_point(10.0, Status::Ok));
        let res = filter.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)), "После reset() совпадение со старым значением игнорируется, ожидаем New");
    }
    #[test]
    fn test_upgrades_old_to_new_if_value_changed() {
        let origin = Rc::new(RefCell::new(FakeOrigin::new()));
        // 1. Первый такт: нормальное новое значение
        origin.borrow_mut().push_new(mock_point(10.0, Status::Ok));
        let mut filter = FnChange::new(origin.clone());
        let res1 = filter.out().unwrap().unwrap();
        assert!(matches!(res1, FnFlow::New(_)), "Ожидаем начальный New");
        // 2. Второй такт: имитируем кривой узел или специфическую логику, 
        // которая вернула измененное значение, но со статусом Old
        origin.borrow_mut().next_flow = Some(FnFlow::Old(mock_point(99.9, Status::Ok)));
        let res2 = filter.out().unwrap().unwrap();
        // 3. Проверка генератора: FnChange обязан заметить 99.9 != 10.0 
        // и принудительно повысить статус до New
        match res2 {
            FnFlow::New(p) => assert_eq!(p.value().as_double(), 99.9, "Должен повысить статус до New"),
            _ => panic!("Ожидался статус New, так как значение изменилось!"),
        }
    }
}
