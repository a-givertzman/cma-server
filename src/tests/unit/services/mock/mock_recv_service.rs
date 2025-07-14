use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}, thread::{self}};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object, Point}, Service}, sync::{channel::{self, Receiver, Sender}, Handles, Owner}};
use crate::core_::{constants::constants::RECV_TIMEOUT, RwLock};
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
///
/// 
pub struct MockRecvService {
    dbg: Dbg,
    name: Name,
    in_queue: HashMap<String, Sender<Point>>,
    recv: Owner<Receiver<Point>>,
    received: Arc<RwLock<Vec<Point>>>,
    recv_limit: Option<usize>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
// 
impl MockRecvService {
    ///
    /// - `in_queue` - The name if link to send to
    /// - `recv_limit` - Service will exit after received specified number of events
    pub fn new(parent: impl Into<String>, in_queue: &str, recv_limit: Option<usize>) -> Self {
        let name = Name::new(parent, format!("MockRecvService{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let (send, recv) = channel::unbounded();
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            in_queue: HashMap::from([(in_queue.to_string(), send)]),
            recv: Owner::new(recv),
            received: Arc::new(RwLock::new(vec![])),
            recv_limit,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
    // pub fn id(&self) -> String {
    //     self.id.clone()
    // }
    ///
    /// 
    pub fn received(&self) -> Arc<RwLock<Vec<Point>>> {
        self.received.clone()
    }
}
//
// 
impl Object for MockRecvService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for MockRecvService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockRecvService")
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for MockRecvService {
    //
    //
    fn get_link(&self, name: &str) -> Sender<Point> {
        match self.in_queue.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.dbg, name),
        }
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let exit = self.exit.clone();
        let recv = self.recv.take().unwrap();
        let received = self.received.clone();
        let recv_limit = self.recv_limit.clone();
        let handle = thread::Builder::new().name(format!("{}.run", dbg)).spawn(move || {
            log::info!("{}.run | Preparing thread - ok", dbg);
            match recv_limit {
                Some(recv_limit) => {
                    let mut received_count = 0;
                    loop {
                        match recv.recv_timeout(RECV_TIMEOUT) {
                            Ok(point) => {
                                log::trace!("{}.run | received: {:?}", dbg, point);
                                received.write().push(point);
                                received_count += 1;
                            }
                            Err(_) => {}
                        };
                        if received_count >= recv_limit {
                            break;
                        }
                        if exit.load(Ordering::Acquire) {
                            break;
                        }
                    }
                }
                None => {
                    loop {
                        match recv.recv_timeout(RECV_TIMEOUT) {
                            Ok(point) => {
                                log::trace!("{}.run | received: {:?}", dbg, point);
                                received.write().push(point);
                            }
                            Err(_) => {}
                        };
                        if exit.load(Ordering::Acquire) {
                            break;
                        }
                    }
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
        self.exit.store(true, Ordering::Release);
    }
}
