use function_name::named;
use sal_core::error::Error;
use sal_sync::services::entity::{Name, Point, PointTxId};
use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}};
use crate::{domain::FnOutRef, services::task::{FlowContext, FnFlow, FnKind, FnOut, FnResult, TaskRetain}};
///
/// ### Function | `FnRetainRead`
/// 
/// Чтение значения `Point` с диска.
/// По умолчанию читает с диска только в первый цикл,
/// дальше возвращает закэшированное значение.
/// 
/// **Особенности работы:**
/// - `enable`: (Через `FnEnable`) При значении `false` (или 0) прерывает передачу данных (возвращает `None`).
/// - `default`: Запасной источник данных (вычисляется лениво), если файл на диске отсутствует.
/// - `every-cycle`: При `true` файл будет читаться с диска на каждом такте вычислений.
/// 
/// **Формат данных на диске:**
/// ```json
/// { "value": {"Bool": false}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": {"Int": 123}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": {"Real": 12.3}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": {"Double": 12.3}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// { "value": {"String": "String value"}, "status": 0, "ts": "2026-06-11T09:06:45.123456789Z" }
/// ```
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
    /// - `parent`: Идентификатор родительского узла
    /// - `path`: Путь к retain файлу для значения (`assets/retain/App/RecorderTask/retain_value.json`)
    /// - `every_cycle`: Если true, чтение будет выполняться в каждом цикле вычислений, иначе только один раз
    /// - `default`: Узел, значение которого будет использовано, если файл отсутствует
    #[named]
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
                    if cached.value() == val.value() {
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
    fn create_temp_json(name: &str, content: &str) -> PathBuf {
        let path = temp_dir().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }
    #[test]
    fn test_single_read_lazy_default() {
        let dbg = "test_single_read_lazy_default";
        let path = create_temp_json("test_retain_1.json", r#"{"value": {"Int": 42}, "status": 0, "ts": "2026-06-11T09:06:45Z"}"#);
        let default_mock = Rc::new(RefCell::new(MockNode { flow: Ok(Some(FnFlow::New(mock_point_int("def", 10)))), inputs_called: 0 }));
        let retain = Arc::new(TaskRetain::mock(dbg, [
            ("test_retain_1".into(), mock_point_int("test_retain_1", 0)),
        ]));
        retain.run().unwrap();
        let mut node = FnRetainRead::new(&Name::from("test"), retain.clone(), "test_retain_1", false, Some(default_mock.clone())).unwrap();
        // Такт 1: Файл существует, читаем один раз
        let res1 = node.out().unwrap().unwrap();
        assert!(res1.is_new());
        assert_eq!(res1.value().as_int().value, 42);
        // Такт 2: Отдаем кэш, статус Old
        let res2 = node.out().unwrap().unwrap();
        assert!(!res2.is_new()); // Old
        assert_eq!(res2.value().as_int().value, 42);
        // Убеждаемся, что default ни разу не опрашивался (Lazy Eval)
        assert_eq!(default_mock.borrow().inputs_called, 0);
        fs::remove_file(path).unwrap();
        retain.exit();
        retain.wait().unwrap();
    }
    #[test]
    fn test_every_cycle_spam_protection() {
        let path = create_temp_json("test_retain_2.json", r#"{"value": {"Int": 99}, "status": 0, "ts": "2026-06-11T09:06:45Z"}"#);
        let retain = Arc::new(TaskRetain::mock(dbg, [
            ("test_retain_1".into(), mock_point_int("test_retain_1", 0)),
        ]));
        let mut node = FnRetainRead::new(&Name::from("test"), retain.clone(), "test_retain_1", true, None).unwrap();
        // Такт 1: Первое чтение файла -> New
        let res1 = node.out().unwrap().unwrap();
        assert!(res1.is_new());
        assert_eq!(res1.value().as_int().value, 99);
        // Такт 2: Значение в файле не изменилось -> Old (защита от спама сработала)
        let res2 = node.out().unwrap().unwrap();
        assert!(!res2.is_new());
        assert_eq!(res2.value().as_int().value, 99);
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn test_transparent_proxy_for_default() {
        let missing_path = temp_dir().join("missing_retain_file.json");
        let default_mock = Rc::new(RefCell::new(MockNode { flow: Ok(Some(FnFlow::New(mock_point_int("def", 55)))), inputs_called: 0 }));
        let mut node = FnRetainRead::new(&Name::from("test"), &missing_path, true, Some(default_mock.clone())).unwrap();
        // Такт 1: Файла нет, every_cycle = true. Узел должен пробросить Old от default
        let res = node.out().unwrap().unwrap();
        assert!(!res.is_new()); // Строго Old, так как proxy пробросил статус default
        assert_eq!(res.value().as_int().value, 55);
        assert_eq!(default_mock.borrow().inputs_called, 1);
    }
}