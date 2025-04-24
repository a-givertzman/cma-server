#[cfg(test)]

mod task_nodes {
    use coco::Stack;
    use sal_core::error::Error;
    use sal_sync::services::{conf::{ConfTree, ServicesConf}, entity::{Name, Object, Point, ToPoint}, safe_lock::rwlock::SafeLock, service::Service, services::Services};
    use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, Once, RwLock}, thread::{self, JoinHandle}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::task_config::TaskConfig,
        services::task::{nested_function::{
            comp::fn_ge, fn_count, fn_kind::FnKind, fn_result::FnResult, sql_metric,
        }, task_nodes::TaskNodes},
    };
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
        let conf = TaskConfig::read(&self_name, path);
        log::debug!("conf: {:?}", conf);
        let services = Arc::new(RwLock::new(Services::new(self_id, ServicesConf::new(
            self_id, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
            "#).unwrap()),
        ))));
        let mock_service = Arc::new(RwLock::new(MockService::new(self_id, "queue")));
        services.wlock(self_id).insert(mock_service.clone());
        let sql_metric_count = sql_metric::COUNT.load(Ordering::SeqCst);
        let fn_count_count = fn_count::COUNT.load(Ordering::SeqCst);
        let fn_ge_count = fn_ge::COUNT.load(Ordering::SeqCst);
        task_nodes.build_nodes(&Name::from(self_id), conf, services);
        let test_data = vec![
            (
                "/path/Point.Name1", 101,
                HashMap::from([
                    (format!("/{}/SqlMetric{}", self_id, sql_metric_count), "101, 1102, 0, 0"),
                    (format!("/{}/FnCount{}.out", self_id, fn_count_count), "1"),
                ])
            ),
            (
                "/path/Point.Name1", 201,
                HashMap::from([
                    (format!("/{}/SqlMetric{}", self_id, sql_metric_count), "201, 1202, 0, 0"),
                    (format!("/{}/FnCount{}.out", self_id, fn_count_count), "1"),
                ])

            ),
            (
                "/path/Point.Name1", 301,
                HashMap::from([
                    (format!("/{}/SqlMetric{}", self_id, sql_metric_count), "301, 1302, 0, 0"),
                    (format!("/{}/FnCount{}.out", self_id, fn_count_count), "1"),
                ])

            ),
            (
                "/path/Point.Name2", 202,
                HashMap::from([
                    (format!("/{}/SqlMetric{}", self_id, sql_metric_count), "301, 1302, 202, 0"),
                    (format!("/{}/FnGe{}.out", self_id, fn_ge_count), "true"),
                ])

            ),
            (
                "/path/Point.Name3", 303,
                HashMap::from([
                    (format!("/{}/SqlMetric{}", self_id, sql_metric_count), "301, 1302, 202, 303"),
                    (format!("/{}/FnGe{}.out", self_id, fn_ge_count), "false"),
                ])

            ),
            (
                "/path/Point.Name3", 304,
                HashMap::from([
                    (format!("/{}/SqlMetric{}", self_id, sql_metric_count), "301, 1302, 202, 304"),
                    (format!("/{}/FnGe{}.out", self_id, fn_ge_count), "false"),
                ])

            ),
        ];
        mock_service.write().unwrap().run().unwrap();
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
                                    let target = match target_value.get(out_name.as_str()) {
                                        Some(target) => target.to_string(),
                                        None => panic!("TaskEvalNode.eval | out.name '{}' - not foind in {:?}", out_name, target_value),
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
        mock_service.read().unwrap().exit();
    }
    ///
    ///
    struct MockService {
        id: String,
        name: Name,
        links: HashMap<String, Sender<Point>>,
        rx_recv: Mutex<Option<Receiver<Point>>>,
        handle: Stack<JoinHandle<()>>,
        exit: Arc<AtomicBool>,
    }
    //
    //
    impl MockService {
        fn new(parent: &str, link_name: &str) -> Self {
            let (send, recv) = mpsc::channel();
            let name = Name::new(parent, format!("MockService{}", COUNT.fetch_add(1, Ordering::Relaxed)));
            Self {
                id: name.join(),
                name,
                links: HashMap::from([
                    (link_name.to_string(), send),
                ]),
                rx_recv: Mutex::new(Some(recv)),
                handle: Stack::new(),
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
                .field("id", &self.id)
                .finish()
        }
    }
    //
    //
    impl Service for MockService {
        //
        //
        fn get_link(&mut self, name: &str) -> Sender<Point> {
            match self.links.get(name) {
                Some(send) => send.clone(),
                None => panic!("{}.run | link '{:?}' - not found", self.id, name),
            }
        }
        //
        //
        fn run(&mut self) -> Result<(), Error> {
            log::info!("{}.run | Starting...", self.id);
            let self_id = self.id.clone();
            let exit = self.exit.clone();
            let rx_recv = self.rx_recv.lock().unwrap().take().unwrap();
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
                    log::info!("{}.run | Starting - ok", self.id);
                    self.handle.push(handle);
                    Ok(())
                }
                Err(err) => {
                    let err = Error::new(&self.id, "run").pass_with("Start failed", err.to_string());
                    log::warn!("{}", err);
                    Err(err)
                }
            }
        }
        //
        //
        fn wait(&self) -> sal_sync::services::future::Future<()> {
            let dbg = self.id.clone();
            let (future, sink) = sal_sync::services::future::Future::new();
            if let Some(handle) = self.handle.pop() {
                std::thread::spawn(move|| {
                    if let Err(err) = handle.join() {
                        log::warn!("{dbg}.wait | Error: {:?}", err);
                    }
                    sink.add(());
                });
            }
            future
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
}


// clear && cargo test -- --test-threads=1 --show-output
// clear && cargo test task_nodes_test -- --test-threads=1 --show-output
