#![allow(non_snake_case)]
use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc, Mutex}, thread};
use log::{warn, info};
use crate::{conf::point_config::name::Name, core_::{object::object::Object, point::point::Point}, services::service::{service::Service, service_handles::ServiceHandles}};
///
/// 
pub struct MockMultiQueue {
    id: String,
    name: Name,
    send: Sender<Point>,
    recv: Vec<Receiver<Point>>,
    received: Arc<Mutex<Vec<Point>>>,
    recvLimit: Option<usize>,
    exit: Arc<AtomicBool>,
}
impl MockMultiQueue {
    pub fn new(parent: &str, index: impl Into<String>, recvLimit: Option<usize>) -> Self {
        let name = Name::new(parent, format!("MockMultiQueue{}", index.into()));
        let (send, recv) = std::sync::mpsc::channel();
        Self {
            id: name.join(),
            name,
            send,
            recv: vec![recv],
            received: Arc::new(Mutex::new(vec![])),
            recvLimit,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn received(&self) -> Arc<Mutex<Vec<Point>>> {
        self.received.clone()
    }
}
//
// 
impl Object for MockMultiQueue {
    fn id(&self) -> &str {
        &self.id
    }
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
            .field("id", &self.id)
            .finish()
    }
}
//
// 
impl Service for MockMultiQueue {
    //
    //
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        assert!(name == "queue", "{}.run | link '{:?}' - not found", self.id, name);
        self.send.clone()
    }
    //
    // 
    fn run(&mut self) -> Result<ServiceHandles, String> {
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let recv = self.recv.pop().unwrap();
        let received = self.received.clone();
        let recvLimit = self.recvLimit.clone();
        let handle = thread::spawn(move || {
            match recvLimit {
                Some(recvLimit) => {
                    let mut receivedCount = 0;
                    'main: loop {
                        match recv.recv() {
                            Ok(point) => {
                                received.lock().unwrap().push(point);
                                receivedCount += 1;
                                if receivedCount >= recvLimit {
                                    break;
                                }
                            }
                            Err(err) => {
                                warn!("{}.run | recv error: {:?}", self_id, err);
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
                                received.lock().unwrap().push(point);
                            }
                            Err(err) => {
                                warn!("{}.run | recv error: {:?}", self_id, err);
                            }
                        }
                        if exit.load(Ordering::SeqCst) {
                            break 'main;
                        }        
                    }
                }
            }
        });
        info!("{}.run | Starting - ok", self.id);
        Ok(ServiceHandles::new(vec![(self.id.clone(), handle)]))
    }
    //
    // 
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
