use coco::Stack;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::services::{entity::{Name, Object, Point}, service::Service};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, RwLock}, thread::{self, JoinHandle}, time::Duration};
///
/// 
pub struct TaskTestReceiver {
    dbg: Dbg,
    name: Name,
    iterations: usize, 
    in_send: HashMap<String, Sender<Point>>,
    in_recv: Mutex<Option<Receiver<Point>>>,
    received: Arc<RwLock<Vec<Point>>>,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
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
            dbg: Dbg::new(name.parent(), name.me()),
            name,
            iterations,
            in_send: HashMap::from([(recv_queue.to_string(), send)]),
            in_recv: Mutex::new(Some(recv)),
            received: Arc::new(RwLock::new(vec![])),
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
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
            .field("id", &self.dbg)
            .finish()
    }
}
//
// 
impl Service for TaskTestReceiver {
    //
    //
    fn get_link(&self, name: &str) -> Sender<Point> {
        match self.in_send.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.dbg, name),
        }        
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        log::info!("{}.run | Starting...", dbg);
        let exit = self.exit.clone();
        let received = self.received.clone();
        let mut count = 0;
        // let mut error_count = 0;
        let in_recv = self.in_recv.lock().unwrap().take().unwrap();
        let iterations = self.iterations;
        let handle = thread::Builder::new().name(dbg.to_string()).spawn(move || {
            // log::info!("Task({}).run | prepared", name);
            'main: loop {
                if exit.load(Ordering::Relaxed) {
                    break 'main;
                }
                match in_recv.recv_timeout(Duration::from_millis(100)) {
                    Ok(point) => {
                        count += 1;
                        log::trace!("{}.run | received: {}/{}, (value: {:?})", dbg, count, iterations, point.value());
                        log::trace!("{}.run | received Point: {:#?}", dbg, point);
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
                            mpsc::RecvTimeoutError::Disconnected => log::error!("{}.run | Error receiving from queue: {:?}", dbg, err),
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
            log::info!("{}.run | received {} Point's", dbg, count);
            log::info!("{}.run | exit", dbg);
            // thread::sleep(Duration::from_secs_f32(2.1));
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handle.push(handle);
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
        while !self.handle.is_empty() {
            if let Some(handle) = self.handle.pop() {
                if let Err(err) = handle.join() {
                    log::warn!("{}.wait | Error: {:?}", self.dbg, err);
                    return Err(Error::new(&self.dbg, "wait").pass(format!("{:?}", err)));
                }
            }
        }
        self.is_finished.store(true, Ordering::SeqCst);
        Ok(())
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.is_finished.load(Ordering::SeqCst)
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
