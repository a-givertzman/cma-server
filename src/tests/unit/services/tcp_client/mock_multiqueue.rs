use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{
    entity::{Name, Object, Point},
    Service,
}, sync::{channel::{self, Receiver, Sender}, Handles}};
use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{self}};
use crate::core_::{Mutex, RwLock};
///
/// 
pub struct MockMultiQueue {
    dbg: Dbg,
    name: Name,
    send: Sender<Point>,
    recv: Mutex<Option<Receiver<Point>>>,
    received: Arc<RwLock<Vec<Point>>>,
    recv_limit: Option<usize>,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
impl MockMultiQueue {
    pub fn new(parent: &str, index: impl Into<String>, recv_limit: Option<usize>) -> Self {
        let name = Name::new(parent, format!("MockMultiQueue{}", index.into()));
        let dbg = Dbg::new(name.parent(), name.me());
        let (send, recv) = channel::unbounded();
        Self {
            name,
            send,
            recv: Mutex::new(Some(recv)),
            received: Arc::new(RwLock::new(vec![])),
            recv_limit,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn received(&self) -> Arc<RwLock<Vec<Point>>> {
        self.received.clone()
    }
}
//
// 
impl Object for MockMultiQueue {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for MockMultiQueue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockMultiQueue")
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for MockMultiQueue {
    //
    //
    fn get_link(&self, name: &str) -> Sender<Point> {
        assert!(name == "queue", "{}.run | link '{:?}' - not found", self.dbg, name);
        self.send.clone()
    }
    //
    // 
    fn run(&self) -> Result<(), Error> {
        let self_id = self.dbg.clone();
        let exit = self.exit.clone();
        let recv = self.recv.lock().take().unwrap();
        let received = self.received.clone();
        let recv_limit = self.recv_limit.clone();
        let handle = thread::spawn(move || {
            match recv_limit {
                Some(recv_limit) => {
                    let mut received_count = 0;
                    'main: loop {
                        match recv.recv() {
                            Ok(point) => {
                                received.write().push(point);
                                received_count += 1;
                                if received_count >= recv_limit {
                                    break;
                                }
                            }
                            Err(err) => {
                                log::warn!("{}.run | recv error: {:?}", self_id, err);
                            }
                        }
                        if exit.load(Ordering::SeqCst) {
                            break 'main;
                        }        
                    }
                }
                None => {
                    'main: loop {
                        match recv.recv() {
                            Ok(point) => {
                                received.write().push(point);
                            }
                            Err(err) => {
                                log::warn!("{}.run | recv error: {:?}", self_id, err);
                            }
                        }
                        if exit.load(Ordering::SeqCst) {
                            break 'main;
                        }        
                    }
                }
            }
        });
        log::info!("{}.run | Starting - ok", self.dbg);
        self.handles.push(handle);
        Ok(())
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
