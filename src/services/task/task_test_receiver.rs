use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_handles::ServiceHandles}};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, RwLock}, thread, time::Duration};
///
/// 
pub struct TaskTestReceiver {
    id: String,
    name: Name,
    iterations: usize, 
    in_send: HashMap<String, Sender<Point>>,
    in_recv: Mutex<Option<Receiver<Point>>>,
    received: Arc<RwLock<Vec<Point>>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl TaskTestReceiver {
    ///
    /// Creates new instance TaskTestReceiver
    /// - `index` - Index of instance (TaskTestReceiver1, TaskTestReceiver2,...etc)
    /// - `recv_queue` - name of the link used for receiving Point's
    /// - `iterations` - count down with each received Point, when zero TaskTestReceiver exits
    #[allow(unused)]
    pub fn new(parent: &str, index: impl Into<String>, recv_queue: &str, iterations: usize) -> Self {
        let (send, recv): (Sender<Point>, Receiver<Point>) = mpsc::channel();
        let name = Name::new(parent, format!("TaskTestReceiver{}", index.into()));
        Self {
            id: name.join(),
            name,
            iterations,
            in_send: HashMap::from([(recv_queue.to_string(), send)]),
            in_recv: Mutex::new(Some(recv)),
            received: Arc::new(RwLock::new(vec![])),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns vector of received Pont's
    #[allow(unused)]
    pub fn received(&self) -> Arc<RwLock<Vec<Point>>> {
        self.received.clone()
    }
    ///
    /// Clearing vector of received Pont's
    #[allow(unused)]
    pub fn clear_received(&self) {
        *self.received.write().unwrap() = vec![];
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
        log::info!("{}.run | Starting...", self_id);
        let exit = self.exit.clone();
        let received = self.received.clone();
        let mut count = 0;
        // let mut error_count = 0;
        let in_recv = self.in_recv.lock().unwrap().take().unwrap();
        let iterations = self.iterations;
        let handle = thread::Builder::new().name(self_id.clone()).spawn(move || {
            // log::info!("Task({}).run | prepared", name);
            'main: loop {
                if exit.load(Ordering::Relaxed) {
                    break 'main;
                }
                match in_recv.recv_timeout(Duration::from_millis(100)) {
                    Ok(point) => {
                        count += 1;
                        log::trace!("{}.run | received: {}/{}, (value: {:?})", self_id, count, iterations, point.value());
                        log::trace!("{}.run | received Point: {:#?}", self_id, point);
                        // debug!("{}.run | value: {}\treceived SQL: {:?}", value, sql);
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
                        match err {
                            mpsc::RecvTimeoutError::Timeout => {},
                            mpsc::RecvTimeoutError::Disconnected => log::error!("{}.run | Error receiving from queue: {:?}", self_id, err),
                        }
                        // error_count += 1;
                        // if errorCount > 10 {
                        //     log::warn!("{}.run | Error receiving count > 10, exit...", self_id);
                        //     break 'inner;
                        // }        
                    }
                };
                if exit.load(Ordering::Relaxed) {
                    break 'main;
                }
            };
            log::info!("{}.run | received {} Point's", self_id, count);
            log::info!("{}.run | exit", self_id);
            // thread::sleep(Duration::from_secs_f32(2.1));
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.id);
                Ok(ServiceHandles::new(vec![(self.id.clone(), handle)]))
            }
            Err(err) => {
                let message = format!("{}.run | Start failed: {:#?}", self.id, err);
                log::warn!("{}", message);
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
