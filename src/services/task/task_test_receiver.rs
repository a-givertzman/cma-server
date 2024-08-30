use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_handles::ServiceHandles}};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, RwLock}, thread};
use log::{info, warn, trace, debug};
///
/// 
pub struct TaskTestReceiver {
    id: String,
    name: Name,
    iterations: usize, 
    in_send: HashMap<String, Sender<Point>>,
    in_recv: Vec<Receiver<Point>>,
    received: Arc<RwLock<Vec<Point>>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl TaskTestReceiver {
    ///
    /// 
    pub fn new(parent: &str, index: impl Into<String>, recv_queue: &str, iterations: usize) -> Self {
        let (send, recv): (Sender<Point>, Receiver<Point>) = mpsc::channel();
        let name = Name::new(parent, format!("TaskTestReceiver{}", index.into()));
        Self {
            id: name.join(),
            name,
            iterations,
            in_send: HashMap::from([(recv_queue.to_string(), send)]),
            in_recv: vec![recv],
            received: Arc::new(RwLock::new(vec![])),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
    pub fn received(&self) -> Arc<RwLock<Vec<Point>>> {
        self.received.clone()
    }
}
//
// 
impl Object for TaskTestReceiver {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
unsafe impl Send for TaskTestReceiver {}
unsafe impl Sync for TaskTestReceiver {}
//
// 
impl Debug for TaskTestReceiver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskTestReceiver")
            .field("id", &self.id)
            .finish()
    }
}
//
// 
impl Service for TaskTestReceiver {
    //
    //
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        match self.in_send.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }        
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        let self_id = self.id.clone();
        info!("{}.run | Starting...", self_id);
        let exit = self.exit.clone();
        let received = self.received.clone();
        let mut count = 0;
        // let mut error_count = 0;
        let in_recv = self.in_recv.pop().unwrap();
        let iterations = self.iterations;
        let handle = thread::Builder::new().name(self_id.clone()).spawn(move || {
            // info!("Task({}).run | prepared", name);
            'main: loop {
                if exit.load(Ordering::Relaxed) {
                    break 'main;
                }
                match in_recv.recv() {
                    Ok(point) => {
                        debug!("{}.run | received: {}/{}, (value: {:?})", self_id, count, iterations, point.value());
                        trace!("{}.run | received Point: {:#?}", self_id, point);
                        // debug!("{}.run | value: {}\treceived SQL: {:?}", value, sql);
                        count += 1;
                        received.write().unwrap().push(point.clone());
                        if count >= iterations {
                            break 'main;
                        }
                        match point {
                            Point::Bool(_) => {},
                            Point::Int(_) => {},
                            Point::Real(_) => {},
                            Point::Double(_) => {},
                            Point::String(p) => {
                                if p.name.to_lowercase().ends_with("exit") || p.value == "exit" {
                                    break 'main;
                                }
                            },
                        }
                    }
                    Err(err) => {
                        warn!("{}.run | Error receiving from queue: {:?}", self_id, err);
                        // error_count += 1;
                        // if errorCount > 10 {
                        //     warn!("{}.run | Error receiving count > 10, exit...", self_id);
                        //     break 'inner;
                        // }        
                    }
                };
                if exit.load(Ordering::Relaxed) {
                    break 'main;
                }
            };
            info!("{}.run | received {} Point's", self_id, count);
            info!("{}.run | exit", self_id);
            // thread::sleep(Duration::from_secs_f32(2.1));
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
        self.exit.store(true, Ordering::Relaxed);
    }
    // pub fn getInputValues(&mut self) -> Receiver<PointType> {
    //     self.recv.pop().unwrap()
    // }
}
