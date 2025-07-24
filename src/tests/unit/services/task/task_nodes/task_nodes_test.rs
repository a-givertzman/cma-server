#[cfg(test)]

use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::{Name, Object, Point, ToPoint}, Service, Services}, sync::{channel::{self, Receiver, Sender}, Handles, Owner}};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, Once}, thread::{self}};
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::services::task::{fn_kind::FnKind, fn_result::FnResult, TaskConf, TaskNodes};
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
/// clear && cargo test -- --test-threads=1 --show-output
/// clear && cargo test task_nodes_test -- --test-threads=1 --show-output
///
#[test]
fn test() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    println!("test");
    let path = "./src/tests/unit/services/task/task_nodes/task_nodes.yaml";
    let self_id = "test";
    let self_name = Name::new("", self_id);
    let mut task_nodes = TaskNodes::new(self_id);
    let conf = TaskConf::read(&self_name, path);
    log::debug!("conf: {:?}", conf);
    let services = Arc::new(Services::new(self_id, ServicesConf::new(
        self_id, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), None));
    let mock_service = Arc::new(MockService::new(self_id, "queue"));
    services.insert(mock_service.clone());
    task_nodes.build_nodes(&Name::from(self_id), conf, services);
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
    for (name, value, target_value) in test_data {
        let point = value.to_point(0, name);
        // let inputName = &point.name();
        log::debug!("input point name: {:?}  value: {:?}", name, value);
        match task_nodes.get_eval_node(&name) {
            Some(eval_node) => {
                eval_node.add(&point);
                log::debug!("evalNode: {:?}", eval_node.name());
                log::debug!("evalNode outs: {:?}", eval_node.get_outs());
                for eval_node_var in eval_node.get_vars() {
                    log::trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", eval_node.name(), eval_node_var.borrow().id());
                    eval_node_var.borrow_mut().eval();
                    log::debug!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", eval_node.name(), eval_node_var.borrow().id());
                };
                for eval_node_out in eval_node.get_outs() {
                    log::trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node.name());
                    let out = eval_node_out.borrow_mut().out();
                    match out {
                        FnResult::Ok(out) => {
                            let out_value = out.value().to_string();
                            log::debug!("TaskEvalNode.eval | evalNode '{}' out - '{}': {:?}", eval_node.name(), eval_node_out.borrow().id(), out);
                            if eval_node_out.borrow().kind() != &FnKind::Var {
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
                        FnResult::None => log::warn!("TaskEvalNode.eval | evalNode '{}' out is None", eval_node.name()),
                        FnResult::Err(err) => log::warn!("TaskEvalNode.eval | evalNode '{}' out is Error: {:#?}", eval_node.name(), err),
                    };
                };
            }
            None => panic!("input {:?} - not found in the current taskStuff", &name)
        };
    }
    mock_service.exit();
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
