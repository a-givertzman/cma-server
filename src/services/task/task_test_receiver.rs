use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object, Point}, Service}, sync::{channel::{self, Receiver, RecvTimeoutError, Sender}, Handles, Owner}};
use std::{fmt::Debug, ops::{Bound, RangeBounds}, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread::{self}, time::Duration};
use crate::domain::RwLock;
///
/// 
pub struct TaskTestReceiver {
    name: Name,
    iterations: usize, 
    in_send: Sender<Point>,
    in_recv: Owner<Receiver<Point>>,
    received: Arc<RwLock<Vec<Point>>>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
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
    pub fn new(parent: &str, index: impl Into<String>, iterations: usize) -> Self {
        let (send, recv): (Sender<Point>, Receiver<Point>) = channel::unbounded();
        let name = Name::new(parent, format!("TaskTestReceiver{}", index.into()));
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            iterations,
            in_send: send,
            in_recv: Owner::new(recv),
            received: Arc::new(RwLock::new(vec![])),
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Returns vector of received Pont's
    #[allow(unused)]
    pub fn received(&self) -> Vec<Point> {
        self.received.read().clone()
    }
    ///
    /// Returns vector of received Pont's
    #[allow(unused)]
    pub fn received_len(&self) -> usize {
        self.received.read().len()
    }
    ///
    /// Returns and removes the subslice by the specified range from inner `received`
    #[allow(unused)]
    pub fn drain<R: RangeBounds<usize> + std::fmt::Debug>(&self, range: R) -> Vec<Point> {
        let mut lock = self.received.write();
        let len = lock.len();
        // Используем встроенный метод для получения индексов (стабилизирован в последних версиях)
        // Если диапазон некорректен, вернем пустой вектор
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        if start >= len {
            log::error!("{}.drain | Out of range or wrong Range: {:?}", self.dbg, range);
            return Vec::new();
        }
        lock.drain(range).collect()
    }
    ///
    /// Clearing vector of received Pont's
    #[allow(unused)]
    pub fn clear_received(&self) {
        *self.received.write() = vec![];
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
    fn get_link(&self, n_ame: &str) -> Sender<Point> {
        self.in_send.clone()
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
        let in_recv = self.in_recv.take().unwrap();
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
                        received.write().push(point.clone());
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
                            Point::Bytes(_) => {},
                        }
                    }
                    Err(err) => {
                        match err {
                            RecvTimeoutError::Timeout => {},
                            _ => log::error!("{}.run | Error receiving from queue: {:?}", dbg, err),
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
        self.exit.store(true, Ordering::Relaxed);
    }
    // pub fn getInputValues(&mut self) -> Receiver<PointType> {
    //     self.recv.pop().unwrap()
    // }
}
