use coco::Stack;
use sal_core::error::Error;
use sal_sync::services::{
    entity::{Name, Object, Point},
    Service,
};
use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, thread::{self, JoinHandle}};

use crate::core_::{Mutex, RwLock};
///
/// 
pub struct MockMultiQueue {
    dbg: String,
    name: Name,
    send: Sender<Point>,
    recv: Mutex<Option<Receiver<Point>>>,
    received: Arc<RwLock<Vec<Point>>>,
    recv_limit: Option<usize>,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
}
impl MockMultiQueue {
    pub fn new(parent: &str, index: impl Into<String>, recv_limit: Option<usize>) -> Self {
        let name = Name::new(parent, format!("MockMultiQueue{}", index.into()));
        let (send, recv) = std::sync::mpsc::channel();
        Self {
            dbg: name.join(),
            name,
            send,
            recv: Mutex::new(Some(recv)),
            received: Arc::new(RwLock::new(vec![])),
            recv_limit,
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
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
        self.handle.push(handle);
        Ok(())
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
        self.exit.store(true, Ordering::SeqCst);
    }
}
