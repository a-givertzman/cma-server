#[cfg(test)]

mod task_nodes {
    use log::{info, debug, trace, warn};
    use sal_sync::services::{entity::{name::Name, object::Object, point::point::{Point, ToPoint}}, service::{service::Service, service_handles::ServiceHandles}};
    use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, Once, RwLock}, thread};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::task_config::TaskConfig,
        services::{
            safe_lock::SafeLock, services::Services,
            task::{nested_function::{comp::fn_ge, fn_count, fn_kind::FnKind, fn_result::FnResult, sql_metric}, task_nodes::TaskNodes}
        },
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
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test");
        let path = "./src/tests/unit/services/task/task_nodes/task_nodes.yaml";
        let self_id = "test";
        let self_name = Name::new("", self_id);
        let mut task_nodes = TaskNodes::new(self_id);
        let conf = TaskConfig::read(&self_name, path);
        debug!("conf: {:?}", conf);
        let services = Arc::new(RwLock::new(Services::new(self_id, None)));
        let mock_service = Arc::new(Mutex::new(MockService::new(self_id, "queue")));
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
        mock_service.lock().unwrap().run().unwrap();
        for (name, value, target_value) in test_data {
            let point = value.to_point(0, name);
            // let inputName = &point.name();
            debug!("input point name: {:?}  value: {:?}", name, value);
            match task_nodes.get_eval_node(&name) {
                Some(eval_node) => {
                    eval_node.add(&point);
                    debug!("evalNode: {:?}", eval_node.name());
                    debug!("evalNode outs: {:?}", eval_node.get_outs());
                    for eval_node_var in eval_node.get_vars() {
                        trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", eval_node.name(), eval_node_var.borrow().id());
                        eval_node_var.borrow_mut().eval();
                        debug!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", eval_node.name(), eval_node_var.borrow().id());
                    };
                    for eval_node_out in eval_node.get_outs() {
                        trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node.name());
                        let out = eval_node_out.borrow_mut().out();
                        match out {
                            FnResult::Ok(out) => {
                                let out_value = out.value().to_string();
                                debug!("TaskEvalNode.eval | evalNode '{}' out - '{}': {:?}", eval_node.name(), eval_node_out.borrow().id(), out);
                                if eval_node_out.borrow().kind() != &FnKind::Var {
                                    let out_name = out.name();
                                    debug!("TaskEvalNode.eval | out.name: '{}'", out_name);
                                    let target = match target_value.get(out_name.as_str()) {
                                        Some(target) => target.to_string(),
                                        None => panic!("TaskEvalNode.eval | out.name '{}' - not foind in {:?}", out_name, target_value),
                                    };
                                    assert!(out_value == target, "\n   outValue: {} \ntargetValue: {}", out_value, target);
                                }
                            }
                            FnResult::None => warn!("TaskEvalNode.eval | evalNode '{}' out is None", eval_node.name()),
                            FnResult::Err(err) => warn!("TaskEvalNode.eval | evalNode '{}' out is Error: {:#?}", eval_node.name(), err),
                        };
                    };
                }
                None => panic!("input {:?} - not found in the current taskStuff", &name)
            };
        }
        mock_service.lock().unwrap().exit();
    }
    ///
    ///
    struct MockService {
        id: String,
        name: Name,
        links: HashMap<String, Sender<Point>>,
        rx_recv: Vec<Receiver<Point>>,
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
                rx_recv: vec![recv],
                exit: Arc::new(AtomicBool::new(false)),
            }
        }
    }
    //
    //
    impl Object for MockService {
        fn id(&self) -> &str {
            &self.id
        }
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
        fn run(&mut self) -> Result<ServiceHandles<()>, String> {
            info!("{}.run | Starting...", self.id);
            let self_id = self.id.clone();
            let exit = self.exit.clone();
            let rx_recv = self.rx_recv.pop().unwrap();
            let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
                loop {
                    match rx_recv.recv() {
                        Ok(point) => {
                            debug!("{}.run | received: {:?}", self_id, point);
                        }
                        Err(err) => {
                            warn!("{}.run | error: {:?}", self_id, err);
                        }
                    }
                    if exit.load(Ordering::SeqCst) {
                        break;
                    }
                }
            });
            match handle {
                Ok(handle) => {
                    info!("{}.run | Starting - ok", self.id);
                    Ok(ServiceHandles::new(vec![(self.id.clone(), handle)]))
                }
                Err(err) => {
                    let message = format!("{}.run | Start failed: {:#?}", self.id, err);
                    warn!("{}", message);
                    Err(message)
                }
            }
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
