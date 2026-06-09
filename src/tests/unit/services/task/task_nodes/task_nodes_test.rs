#[cfg(test)]

use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{Service, Services, conf::{ConfTree, ServicesConf}, entity::{Name, Object, Point, Status, ToPoint}}, sync::{Handles, Owner, channel::{self, Receiver, Sender}}};
use testing::entities::test_value::Value;
use std::{cell::RefCell, collections::HashMap, fmt::{Debug, Display}, rc::Rc, sync::{Arc, Once, atomic::{AtomicBool, AtomicUsize, Ordering}}, thread::{self}};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::services::task::{FlowContext, FnKind, FnResult, TaskConf, TaskEvalNode, TaskNodes};
///
///
static INIT: Once = Once::new();
///
/// once called initialisation
fn init_once() {
    INIT.call_once(|| {
        // implement your initialisation code to be called only once for current test file
    })
}
///
/// returns:
///  - Rc<RefCell<Box<dyn FnInOut>>>...
fn init_each() {
    // fn_ge::COUNT.reset();
    // fn_count::COUNT.reset();
    // sql_metric::COUNT.reset();
}
///
/// ### Тестирует подкапотные механизмы TaskNodes
/// - Как запустить
/// clear && cargo test -- --test-threads=1 --show-output
/// clear && cargo test task_nodes_test -- --test-threads=1 --show-output
///
#[test]
fn manual_eval() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let dbg = "manual_eval";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::new(dbg, 0);
    let conf = serde_yaml::from_str(r#"
        service Task Task1:
            cycle: 1 us
            in queue api-link:
                max-length: 10000
            # recv-queue: Receiver.queue
            let Var2:
                input: point int '/path/Point.Name2'
            let Var3:
                input: point int '/path/Point.Name3'
            fn Debug:
                input fn SqlMetric:
                    initial: 0.123      # начальное значение
                    table: table_name
                    sql: "{input.value}, {input1.value}, {input2.value}, {input3.value}"
                    input let Var1:
                        input: point int '/path/Point.Name1'
                    input1 fn Add:
                            input1: Var1
                            input2: const int 1001
                    input2: Var2
                    input3: Var3
            fn Ge:
                input1: point int '/path/Point.Name2'
                input2: point int '/path/Point.Name3'
            fn Count:
                input: point int  '/path/Point.Name1'
    "#).unwrap();
    let conf = TaskConf::from_yaml(&self_name, &conf);
    log::trace!("conf: {:?}", conf);
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), None));
    let mock_service = Arc::new(MockService::new(dbg, "queue"));
    services.insert(mock_service.clone());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    let test_data = vec![
        (
            "/path/Point.Name1", 101,
            [
                ("SqlMetric", "101, 1102, 0, 0"),
                ("FnCount", "1"),
                ("FnGe", "---"),
            ]
        ),
        (
            "/path/Point.Name1", 201,
            [
                ("SqlMetric", "201, 1202, 0, 0"),
                ("FnCount", "1"),
                ("FnGe", "---"),
            ]
        ),
        (
            "/path/Point.Name1", 301,
            [
                ("SqlMetric", "301, 1302, 0, 0"),
                ("FnCount", "1"),
                ("FnGe", "---"),
            ]
        ),
        (
            "/path/Point.Name2", 202,
            [
                ("SqlMetric", "301, 1302, 202, 0"),
                ("FnCount", "---"),
                ("FnGe", "true"),
            ]
        ),
        (
            "/path/Point.Name3", 303,
            [
                ("SqlMetric", "301, 1302, 202, 303"),
                ("FnCount", "---"),
                ("FnGe", "false"),
            ]
        ),
        (
            "/path/Point.Name3", 304,
            [
                ("SqlMetric", "301, 1302, 202, 304"),
                ("FnCount", "---"),
                ("FnGe", "false"),
            ]
        ),
    ];
    mock_service.run().unwrap();
    let flow = FlowContext::new();
    for (name, value, target_value) in test_data {
        let point = value.to_point(0, name);
        // let inputName = &point.name();
        log::debug!("input point name: {:?}  value: {:?}", name, value);
        match task_nodes.get_eval_node(&name) {
            Some(eval_node) => {
                eval_node.borrow().add(&point);
                log::debug!("evalNode: {:?}", eval_node.borrow().name());
                log::trace!("evalNode outs: {:?}", eval_node.borrow().get_outs());
                for eval_node_var in eval_node.borrow().get_vars() {
                    log::trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", eval_node.borrow().name(), eval_node_var.borrow().id());
                    eval_node_var.borrow_mut().out();
                    log::debug!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", eval_node.borrow().name(), eval_node_var.borrow().id());
                };
                for eval_node_out in eval_node.borrow().get_outs() {
                    log::trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node.borrow().name());
                    let out = flow.ignore(eval_node_out.borrow_mut().out());
                    match out {
                        Ok(Some(out)) => {
                            let out_value = out.value().to_string();
                            log::debug!("TaskEvalNode.eval | evalNode '{}' out - '{}': {:?}", eval_node.borrow().name(), eval_node_out.borrow().id(), out);
                            if eval_node_out.borrow().kind() != FnKind::Var {
                                let out_name = out.name();
                                log::debug!("TaskEvalNode.eval | out.name: '{}'", out_name);
                                let target = match out_name {
                                    x if x.contains("SqlMetric") => target_value[0].1,
                                    x if x.contains("FnCount") => target_value[1].1,
                                    x if x.contains("FnGe") => target_value[2].1,
                                    _ => panic!("TaskEvalNode.eval | unexpected function {out_name}")
                                };
                                assert!(out_value == target, "\n   outValue: {} \ntargetValue: {}", out_value, target);
                            }
                        }
                        Ok(None) => log::warn!("TaskEvalNode.eval | evalNode '{}' out is None", eval_node.borrow().name()),
                        Err(err) => log::warn!("TaskEvalNode.eval | evalNode '{}' out is Error: {:#?}", eval_node.borrow().name(), err),
                    };
                };
            }
            None => panic!("input {:?} - not found in the current taskStuff", &name)
        };
    }
    mock_service.exit();
}
///
/// Тестирует TaskNodes.eval
#[test]
fn eval() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "eval";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::new(dbg, 0);
    let conf = serde_yaml::from_str(r#"
        service Task Task1:
            cycle: 1 us
            in queue api-link:
                max-length: 10000
            # recv-queue: Receiver.queue
            let Var2:
                input: point int '/path/Point.Name2'
            let Var3:
                input: point int '/path/Point.Name3'
            fn Debug:
                input fn SqlMetric:
                    initial: 0.123      # начальное значение
                    table: table_name
                    sql: "{input.value}, {input1.value}, {input2.value}, {input3.value}"
                    input let Var1:
                        input: point int '/path/Point.Name1'
                    input1 fn Add:
                            input1: Var1
                            input2: const int 1001
                    input2: Var2
                    input3: Var3
            fn Ge:
                input1: point int '/path/Point.Name2'
                input2: point int '/path/Point.Name3'
            fn Count:
                input: point int  '/path/Point.Name1'
    "#).unwrap();
    let conf = TaskConf::from_yaml(&self_name, &conf);
    log::trace!("conf: {:?}", conf);
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), None));
    let mock_service = Arc::new(MockService::new(dbg, "queue"));
    services.insert(mock_service.clone());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    let test_data = vec![
        (
            "/path/Point.Name1", 101,
            [
                ("SqlMetric", "101, 1102, 0, 0"),
                ("FnCount", "1"),
                ("FnGe", "---"),
            ]
        ),
        (
            "/path/Point.Name1", 201,
            [
                ("SqlMetric", "201, 1202, 0, 0"),
                ("FnCount", "1"),
                ("FnGe", "---"),
            ]
        ),
        (
            "/path/Point.Name1", 301,
            [
                ("SqlMetric", "301, 1302, 0, 0"),
                ("FnCount", "1"),
                ("FnGe", "---"),
            ]
        ),
        (
            "/path/Point.Name2", 202,
            [
                ("SqlMetric", "301, 1302, 202, 0"),
                ("FnCount", "---"),
                ("FnGe", "true"),
            ]
        ),
        (
            "/path/Point.Name3", 303,
            [
                ("SqlMetric", "301, 1302, 202, 303"),
                ("FnCount", "---"),
                ("FnGe", "false"),
            ]
        ),
        (
            "/path/Point.Name3", 304,
            [
                ("SqlMetric", "301, 1302, 202, 304"),
                ("FnCount", "---"),
                ("FnGe", "false"),
            ]
        ),
        (
            "/path/Point.Unknown", 1111,
            [
                ("SqlMetric", "301, 1302, 202, 304"),
                ("FnCount", "---"),
                ("FnGe", "false"),
            ]
        ),
    ];
    mock_service.run().unwrap();
    let flow = FlowContext::new();
    for (name, value, target_value) in test_data {
        let point = value.to_point(0, name);
        // let inputName = &point.name();
        log::debug!("input point name: {:?}  value: {:?}", name, value);
        task_nodes.eval(point);
        // Теперь заглядываем под капот только для снятия показаний
        match task_nodes.get_eval_node(&name) {
            Some(eval_node) => {
                for eval_node_out in eval_node.borrow().get_outs() {
                    let out = flow.ignore(eval_node_out.borrow_mut().out());
                    if let Ok(Some(out)) = out {
                        let out_value = out.value().to_string();
                        let out_name = out.name();
                        if eval_node_out.borrow().kind() != FnKind::Var {
                            let target = match out_name {
                                x if x.contains("SqlMetric") => target_value[0].1,
                                x if x.contains("FnCount") => target_value[1].1,
                                x if x.contains("FnGe") => target_value[2].1,
                                _ => panic!("TaskEvalNode.eval | unexpected function {out_name}")
                            };
                            assert!(out_value == target, "\n   outValue: {} \ntargetValue: {}", out_value, target);
                        }
                    }
                }
            }
            None => {
                if !["/path/Point.Unknown"].contains(&name) {
                    panic!("Входной сигнал '{name}' не найден в конфиге, но пришел на вход вычислений. Проверьте имя или добавьте в конфиг или уберите из входящих");
                }
            }
        }
    }
    mock_service.exit();
}
#[test]
fn test_state_retention() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "state_retention";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::new(dbg, 0);
    let conf = serde_yaml::from_str(r#"
        service Task Task1:
            cycle: 1 us
            in queue api-link:
                max-length: 10000
            # recv-queue: Receiver.queue
            fn Add 1:
                input1: point int '/path/Point.A'
                input2: point int '/path/Point.B'
            let VarA:
                input: point int '/path/Point.A'
            let VarB:
                input: point int '/path/Point.B'
            fn Add 2:
                input1: VarA
                input2: VarB
    "#).unwrap();
    let conf = TaskConf::from_yaml(&self_name, &conf);
    log::trace!("conf: {:?}", conf);
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), None));
    let mock_service = Arc::new(MockService::new(dbg, "queue"));
    services.insert(mock_service.clone());
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    let test_data = [
        // 1. Подаем на вход первую переменную. Вторая пока пустая (или по умолчанию).
        ("/path/Point.A", 10, 10),
        // 2. Подаем на вход вторую переменную. Результат равен сумме старой первой и данно
        ("/path/Point.B", 06, 16),
        ("/path/Point.B", 02, 12),
        ("/path/Point.A", 06, 08),
    ];
    let flow = FlowContext::new();
    for (name, val, target) in test_data {
        let point = val.to_point(0, name);
        task_nodes.eval(point);
        // Проверяем, что выход пересчитался с учетом A=10, B=None
        let node = task_nodes.get_eval_node(name).unwrap();
        for out in node.borrow().get_outs() {
            if let Ok(Some(result)) = flow.ignore(out.borrow_mut().out()) {
                let result = result.to_int().as_int().value;
                // Если результат использует старое значение A, значит инкапсуляция состояния работает.
                assert!(result == target, "{dbg} | input: {} \n result: {} \n target: {}", name, result, target);
            }
        }
    }
}
///
/// ### Тест "Отравленных данных" (Poisoned Data / Type Mismatch)
/// - Датчик сошел с ума и вместо числа прислал NaN, пустую строку или статус Invalid. 
/// - Движок должен проглотить это, вычислить FnResult::Err (или None),
/// пробросить эту ошибку по зависимым узлам графа вниз, но ни в коем случае не упасть.
#[test]
fn poisoned_data() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "poisoned_data";
    let services = Arc::new(Services::new(dbg, ServicesConf::new(dbg, ConfTree::new_root(serde_yaml::from_str("retain:").unwrap())), None));
    let mut task_nodes = TaskNodes::new(dbg, 0);
    let conf = serde_yaml::from_str(r#"
        service Task Task1:
            cycle: 1 us
            in queue api-link:
                max-length: 10000
            fn Add:
                input1: point int '/path/Sensor.A'
                input2: point int '/path/Sensor.B'
            let VarA:
                input fn PointId:
                    input point int every
    "#).unwrap();
    let conf = TaskConf::from_yaml(dbg, &conf);
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Подаем валидную A
    task_nodes.eval(10.to_point(0, "/path/Sensor.A"));
    // Подаем B с неверным типом данных (строку вместо int) или со статусом Invalid
    // Зависит от того, как у вас реализован to_point с ошибками. 
    // Допустим, мы шлем строку туда, где ждут Int.
    let bad_point = "garbage".to_point(0, "/path/Sensor.B"); 
    task_nodes.eval(bad_point);
    // Проверяем результат вычислений
    let node = task_nodes.get_eval_node("/path/Sensor.B").unwrap();
    let flow = FlowContext::new();
    for out in node.borrow().get_outs() {
        let result = flow.ignore(out.borrow_mut().out());
        // Движок должен честно сказать, что математика не сошлась, но остаться в живых
        assert!(matches!(result, FnResult::Ok(Some(_))), "Узел должен вернуть FnResult(Point {{Status::Invalid}}) при мусорных входных данных: \nresult: {:?} \ntarget: FnResult::Ok(_)", result);
        let result = result.unwrap().unwrap().status();
        let target = Status::Invalid;
        assert!(result == result, "Узел должен вернуть FnResult(Point {{Status::Invalid}}) при мусорных входных данных: \nresult: {:?} \ntarget: {:?}", result, target);
    }
}
///
/// ### Тест для point `every`
/// - `point any every` - Проверяем что реагирует на любое событие
/// - `point int every` - Проверяем что реагирует на любое событие указанного типа
#[test]
fn every_logic() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    let dbg = "every_logic";
    log::debug!("{dbg}");
    let self_name = Name::new("", dbg);
    let mut task_nodes = TaskNodes::new(dbg, 0);
    let conf_yaml = serde_yaml::from_str(r#"
        service Task TaskEveryTest:
            cycle: 1 us
            in queue api-link:
                max-length: 10000

            # реагирует ТОЛЬКО на '/path/Point.A'
            fn Acc SpecA:
                input: point int '/path/Point.A'

            # реагирует ТОЛЬКО на '/path/Point.B'
            fn Acc SpecB:
                input: point int '/path/Point.B'

            # реагирует на ЛЮБЫЕ события
            fn Acc Every:
                input: point any every
    "#).unwrap();
    let conf = TaskConf::from_yaml(&self_name, &conf_yaml);
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str("retain:").unwrap()),
    ), None));
    task_nodes.build_nodes(&Name::from(dbg), &conf, services).unwrap();
    // Сценарий проверки
    // 1. Отправляем /path/Point.A
    // Ожидаем: CountSpec = 1, CountEvery = 1
    task_nodes.eval(10.to_point(0, "/path/Point.A"));
    // Проверяем CountSpecA
    let spec_node = task_nodes.get_eval_node("/path/Point.A").unwrap();
    let spec_out = get_first_out_value(dbg, spec_node);
    assert_eq!(spec_out, Some(Value::Int(10)), "Специфичный узел должен был посчитать Point.A");
    // Проверяем CountSpecB
    let spec_node = task_nodes.get_eval_node("/path/Point.B").unwrap();
    let spec_out = get_first_out_value(dbg, spec_node);
    assert_eq!(spec_out, None, "Узел B НЕ должен был считать Point.A");
    // Проверяем CountEvery
    let every_node = task_nodes.get_eval_node("every").unwrap();
    let every_out = get_first_out_value(dbg, every_node);
    assert_eq!(every_out, Some(Value::Int(10)), "Узел every должен был посчитать Point.A");
    // 2. Отправляем /path/Point.B (которого нет в явном виде в конфиге)
    // Ожидаем: CountSpec = 1 (не изменился), CountEvery = 2 (среагировал)
    task_nodes.eval(20.to_point(0, "/path/Point.B"));

    let spec_node = task_nodes.get_eval_node("/path/Point.A").unwrap();
    let spec_out = get_first_out_value(dbg, spec_node);
    assert_eq!(spec_out, Some(Value::Int(20)), "Специфичный узел НЕ должен реагировать на Point.B");
    let spec_node = task_nodes.get_eval_node("/path/Point.B").unwrap();
    let spec_out = get_first_out_value(dbg, spec_node);
    assert_eq!(spec_out, Some(Value::Int(20)), "Специфичный узел должен реагировать на Point.B");
    let every_node = task_nodes.get_eval_node("every").unwrap();
    let every_out = get_first_out_value(dbg, every_node);
    assert_eq!(every_out, Some(Value::Int(30)), "Узел every ДОЛЖЕН был среагировать на Point.B");
}
///
/// Вспомогательная функция для извлечения результата
fn get_first_out_value(dbg: impl Display, node: Rc<RefCell<TaskEvalNode>>) -> Option<Value> {
    let node_name = node.borrow().name();
    let flow = FlowContext::new();
    for out in node.borrow().get_outs() {
        let point = flow.ignore(out.borrow_mut().out());
        if let Ok(Some(result)) = point {
            log::debug!("{dbg} | node {} take {:?} from '{}'", node_name, result.to_string().as_string().value, out.borrow().id());
            return Some(result.value());
        }
    }
    log::debug!("{dbg} | node {} take None", node_name);
    None
}
///
///
struct MockService {
    dbg: Dbg,
    name: Name,
    links: HashMap<String, Sender<Point>>,
    rx_recv: Owner<Receiver<Point>>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
//
impl MockService {
    fn new(parent: &str, link_name: &str) -> Self {
        let (send, recv) = channel::unbounded();
        let name = Name::new(parent, format!("MockService{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            links: HashMap::from([
                (link_name.to_string(), send),
            ]),
            rx_recv: Owner::new(recv),
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
//
//
impl Object for MockService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for MockService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockService")
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for MockService {
    //
    //
    fn get_link(&self, name: &str) -> Sender<Point> {
        match self.links.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.dbg, name),
        }
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let self_id = self.dbg.clone();
        let exit = self.exit.clone();
        let rx_recv = self.rx_recv.take().unwrap();
        let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
            loop {
                match rx_recv.recv() {
                    Ok(point) => {
                        log::debug!("{}.run | received: {:?}", self_id, point);
                    }
                    Err(err) => {
                        log::warn!("{}.run | error: {:?}", self_id, err);
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
