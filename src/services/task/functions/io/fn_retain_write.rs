use sal_core::error::Error;
use sal_sync::services::entity::{Name, Point};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::{domain::{FnOutRef, Sender}, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult, RetainEvent}};
///
/// ### Function | `FnRetainWrite`
/// 
/// Запись значения `Point` на диск через фоновый процесс `TaskRetain`.
/// Пишет строго по изменению значения, статуса или метки времени.
/// Для вычислений прозрачна, не вносит изменений в поток.
/// 
/// **Особенности работы:**
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `default`: Запасной источник данных, если вход молчит (`Ok(None)`),
///    то узел попытается вернуть и записать на диск `default`.
///
/// **Пример:**
/// ```yaml
/// fn Retain:            # Тут происходит запись
///     key: 'OperatingCycleId'
///     input fn Acc:
///         initial fn Retain:
///             default: const int 0
///             key: 'OperatingCycleId'
///         input: opCycleIsDone
/// ```
#[derive(Debug)]
pub struct FnRetainWrite {
    kind: FnKind,
    key: String,
    default: Option<FnOutRef>,
    input: FnOutRef,
    cache: Option<Point>,
    send: Sender<RetainEvent>,
    id: String,
}
//
impl FnRetainWrite {
    ///
    /// ### Returns `FnRetainWrite` new instance
    /// - `parent`: Идентификатор родительского узла
    /// - `send`: Канал для отправки событий записи в TaskRetain
    /// - `key`: Уникальный ключ переменной для сохранения
    /// - `default`: Запасной источник данных
    /// - `input`: Входной сигнал
    pub fn new(parent: &Name, send: Sender<RetainEvent>, key: impl Into<String>, default: Option<FnOutRef>, input: FnOutRef) -> Result<Self, Error> {
        let id = format!("{}/FnRetainWrite{}", parent.join(), COUNT.fetch_add(1, Ordering::Relaxed));
        Ok(Self {
            kind: FnKind::Fn,
            key: key.into(),
            default,
            input,
            cache: None,
            send,
            id,
        })
    }
}
//
impl FnOut for FnRetainWrite {
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
        let mut inputs = self.input.borrow().inputs();
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let input = self.input.borrow_mut().out();
        let default = self.default.as_ref().map(|d| d.borrow_mut().out());
        let mut flow = FlowContext::new();
        let point = if let Some(input) = flow.map(input)? {
            Some(input)
        } else if let Some(default) = default {
            flow.ignore(default)?
        } else {
            None
        };
        let Some(point) = point else {
            return Ok(None);
        };
        let is_changed = match self.cache.as_ref() {
            Some(cache) => cache.value() != point.value() || cache.status() != point.status() || cache.ts() != point.ts(),
            None => true,
        };
        if is_changed {
            self.cache = Some(point.clone());
            if let Err(err) = self.send.send(RetainEvent::new(self.key.clone(), point.clone())) {
                log::error!("{}.out | Can't store value '{}': {:?}", self.id, point.name(), err);
            }
        }
        flow.wrap(point)
    }
    //
    fn hard_reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().hard_reset();
        }
        self.input.borrow_mut().hard_reset();
        self.cache = None;
    }
    fn reset(&mut self) {
        self.cache = None;
    }
}
///
/// Global static counter of FnRetainWrite instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Baisic Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::task::FnFlow;
    use sal_sync::services::entity::{Point, PointHlr, Cot, Status};
    use std::{cell::RefCell, rc::Rc};
    #[derive(Debug)]
    struct MockNode {
        id: String,
        flow: Option<FnFlow>,
        called: usize,
    }
    impl MockNode {
        fn new(id: &str, flow: Option<FnFlow>) -> Self {
            Self { id: id.to_string(), flow, called: 0 }
        }
    }
    impl FnOut for MockNode {
        fn id(&self) -> String { self.id.clone() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec![] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.called += 1;
            Ok(self.flow.clone())
        }
        fn hard_reset(&mut self) {}
        fn reset(&mut self) {}
    }
    fn mock_point(val: i64) -> Point {
        Point::Int(PointHlr::new(1, "test", val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    #[test]
    fn test_eager_evaluation_and_deduplication() {
        let (tx, rx) = crate::domain::unbounded();
        let value = FnFlow::New(mock_point(10));
        let input = Rc::new(RefCell::new(MockNode::new("in", Some(value.clone()))));
        let default = Rc::new(RefCell::new(MockNode::new("def", Some(FnFlow::New(mock_point(5))))));
        let mut retain = FnRetainWrite::new(&Name::from("test"), tx, "key", Some(default.clone()), input.clone()).unwrap();
        // Такт 1: Идут новые данные
        let res = retain.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::New(_)));
        assert_eq!(rx.len(), 1);
        assert_eq!(input.borrow().called, 1);
        assert_eq!(default.borrow().called, 1, "Запасной вход обязан быть опрошен!");
        // Такт 2: Данные не изменились (дубликат в потоке)
        input.borrow_mut().flow = Some(value);
        let res2 = retain.out().unwrap().unwrap();
        assert!(matches!(res2, FnFlow::New(_)));
        assert_eq!(rx.len(), 1, "Диск должен быть защищен от записи дубликатов");
    }
    #[test]
    fn test_default_fallback_ignores_flow() {
        let (tx, rx) = crate::domain::unbounded();
        let input = Rc::new(RefCell::new(MockNode::new("in", None))); // Основной вход обрывается
        let default = Rc::new(RefCell::new(MockNode::new("def", Some(FnFlow::New(mock_point(42))))));
        let mut retain = FnRetainWrite::new(&Name::from("test"), tx, "key", Some(default.clone()), input.clone()).unwrap();
        let res = retain.out().unwrap().unwrap();
        assert!(matches!(res, FnFlow::Old(_)), "Резервное значение должно быть завернуто в Old");
        assert_eq!(res.value().as_int().value, 42);
    }
}
