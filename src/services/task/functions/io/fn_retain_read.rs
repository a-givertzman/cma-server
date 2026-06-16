use sal_core::error::Error;
use sal_sync::services::entity::{Name, Point, PointTxId};
use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}};
use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult, TaskRetain}};
///
/// ### Function | `FnRetainRead`
/// 
/// Чтение значения `Point` из журнала на диске.
/// По умолчанию читает данные только в первый цикл вычислений,
/// далее возвращает закэшированное значение.
/// 
/// **Особенности работы:**
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `default`: Запасной источник данных, если файл на диске отсутствует.
/// - `every-cycle`: При `true` читает актуальные данные из журнала на каждом такте вычислений.
/// 
/// **Пример:**
/// ```yaml
/// fn Retain:
///     key: 'OperatingCycleId'
///     input fn Acc:
///         initial fn Retain:      # Тут происходит чтение
///             default: const int 0
///             key: 'OperatingCycleId'
///         input: opCycleIsDone
/// ```
#[derive(Debug)]
pub struct FnRetainRead {
    txid: usize,
    kind: FnKind,
    key: String,
    retain: Arc<TaskRetain>,
    every_cycle: bool,
    default: Option<FnOutRef>,
    cache: Option<Point>,
    id: String,
}
//
impl FnRetainRead {
    ///
    /// ### Returns `FnRetainRead` new instance
    /// - `parent`: Идентификатор родительского узла.
    /// - `retain`: `TaskRetain`, разделяемый контекст доступа к дисковому журналу.
    /// - `key`: Уникальный ключ переменной в журнале.
    /// - `every_cycle`: Если true, чтение будет выполняться непрерывно, иначе — единоразово при старте.
    /// - `default`: Резервный узел. Будет использован, если ключа в журнале нет.
    pub fn new(parent: &Name, retain: Arc<TaskRetain>, key: impl Into<String>, every_cycle: bool, default: Option<FnOutRef>) -> Result<Self, Error> {
        let id = format!("{}/FnRetainRead{}", parent.join(), COUNT.fetch_add(1, Ordering::Relaxed));
        Ok(Self {
            txid: PointTxId::from_str(&id),
            kind: FnKind::Fn,
            key: key.into(),
            retain,
            every_cycle,
            default,
            cache: None,
            id,
        })
    }
}
//
impl FnOut for FnRetainRead {
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
        if let Some(default) = &self.default {
            inputs.append(&mut default.borrow().inputs());
        }
        inputs
    }
    //
    fn out(&mut self) -> FnResult<FnFlow, String> {
        let mut flow = FlowContext::new();
        if !self.every_cycle {
            if let Some(val) = self.cache.as_ref() {
                return flow.wrap_old(val.clone());
            }
        }
        if let Some(val) = self.retain.get(&self.key) {
            if self.every_cycle {
                if let Some(cached) = &self.cache {
                    if cached.value() == val.value() && cached.status() == val.status() {
                        return flow.wrap_old(val);
                    }
                }
            }
            self.cache = Some(val.clone());
            return flow.wrap_new(val);
        } else {
            if let Some(default) = self.default.as_ref() {
                if self.every_cycle {
                    let Some(val) = flow.map(default.borrow_mut().out())? else { return Ok(None) };
                    return flow.wrap(val);
                } else {
                    let Some(val) = flow.ignore(default.borrow_mut().out())? else { return Ok(None) };
                    self.cache = Some(val.clone());
                    return flow.wrap_new(val);
                }
            }
            Ok(None)
        }
    }
    //
    fn reset(&mut self) {
        if let Some(default) = &self.default {
            default.borrow_mut().reset();
        }
        self.cache = None;
    }
}
///
/// Global static counter of FnRetainRead instances
static COUNT: AtomicUsize = AtomicUsize::new(1);
///
/// Basic Tests
#[cfg(test)]
mod tests {
    use sal_sync::services::Service;
    use sal_sync::services::entity::{Cot, PointHlr, Status};
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::io::Write;
    use std::env::temp_dir;
    #[derive(Debug)]
    struct MockNode {
        flow: FnResult<FnFlow, String>,
        inputs_called: usize,
    }
    impl FnOut for MockNode {
        fn id(&self) -> String { "mock".to_string() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec!["mock_point".to_string()] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.inputs_called += 1;
            self.flow.clone()
        }
        fn reset(&mut self) {}
    }
    fn create_mock(flow: FnFlow) -> FnOutRef {
        Rc::new(RefCell::new(MockNode { flow: Ok(Some(flow)), inputs_called: 0 }))
    }
    fn mock_point_int(id: &str, val: i64) -> Point {
        Point::Int(PointHlr::new(0, id, val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
}
#[cfg(test)]
mod testsы {
    use sal_core::dbg::Dbg;
use sal_sync::services::entity::{Cot, PointHlr, Status};
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::services::task::TaskRetainConf;
    #[derive(Debug)]
    struct MockNode {
        flow: FnResult<FnFlow, String>,
        inputs_called: usize,
    }
    impl FnOut for MockNode {
        fn id(&self) -> String { "mock".to_string() }
        fn kind(&self) -> FnKind { FnKind::Fn }
        fn inputs(&self) -> Vec<String> { vec!["mock_point".to_string()] }
        fn out(&mut self) -> FnResult<FnFlow, String> {
            self.inputs_called += 1;
            self.flow.clone()
        }
        fn reset(&mut self) {}
    }
    fn create_mock(flow: FnFlow) -> Rc<RefCell<MockNode>> {
        Rc::new(RefCell::new(MockNode { flow: Ok(Some(flow)), inputs_called: 0 }))
    }
    fn mock_point_int(id: &str, val: i64) -> Point {
        Point::Int(PointHlr::new(0, id, val, Status::Ok, Cot::Inf, chrono::Utc::now()))
    }
    fn dummy_retain(dbg: impl Into<String>) -> Arc<TaskRetain> {
        Arc::new(TaskRetain::mock(dbg, []))
    }
    #[test]
    fn test_retain_read() {
        let parent = "FnRetainRead";
        let name = Name::new(parent, "test_retain_read");
        let dbg = Dbg::new(parent, name.me());
        let retain = dummy_retain(&dbg); // Пустой кэш
        let point = mock_point_int("test", 42);
        let default_node = create_mock(FnFlow::New(point.clone()));
        let mut node = FnRetainRead::new(&name, retain, "key", false, Some(default_node.clone())).unwrap();
        // Такт 1: Читаем из default (так как кэш пуст), сохраняем в self.cache
        let flow1 = node.out().unwrap().unwrap();
        assert!(flow1.is_new());
        assert_eq!(flow1.value().as_int().value, 42);
        // Такт 2: every_cycle = false, возвращаем Old из кэша (к default больше не обращаемся)
        let flow2 = node.out().unwrap().unwrap();
        assert!(!flow2.is_new());
        assert_eq!(flow2.value().as_int().value, 42);
        // Проверяем ленивое вычисление: default_node был опрошен ровно 1 раз
        assert_eq!(default_node.borrow().inputs_called, 1);
    }
    #[test]
    fn test_retain_read_every_cycle_true() {
        let parent = "FnRetainRead";
        let name = Name::new(parent, "test_retain_read_every_cycle");
        let dbg = Dbg::new(parent, name.me());
        let retain = dummy_retain(&dbg); // Пустой кэш
        let point = mock_point_int("test", 100);
        let default_node = create_mock(FnFlow::New(point.clone()));
        let mut node = FnRetainRead::new(&name, retain, "key", true, Some(default_node.clone())).unwrap();
        // Такт 1: Читаем из default (flow.map)
        let flow1 = node.out().unwrap().unwrap();
        assert!(flow1.is_new());
        // Такт 2: Меняем состояние default на Old
        default_node.borrow_mut().flow = Ok(Some(FnFlow::Old(point.clone())));
        let flow2 = node.out().unwrap().unwrap();
        // Поскольку every_cycle = true, узел честно транслирует Old от default
        assert!(!flow2.is_new());
        // default_node был опрошен 2 раза
        assert_eq!(default_node.borrow().inputs_called, 2);
    }
}
