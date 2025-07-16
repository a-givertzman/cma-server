use log::{warn, info};
use sal_sync::services::{
    entity::{name::Name, object::Object, point::point::Point},
    service::{service::Service, service_handles::ServiceHandles},
};
use std::{fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc, Mutex, RwLock}, thread};
///
/// 
pub struct MockMultiQueue {
    id: String,
    name: Name,
    send: Sender<Point>,
    recv: Mutex<Option<Receiver<Point>>>,
    received: Arc<RwLock<Vec<Point>>>,
    recv_limit: Option<usize>,
    exit: Arc<AtomicBool>,
}
impl MockMultiQueue {
    pub fn new(parent: &str, index: impl Into<String>, recv_limit: Option<usize>) -> Self {
        let name = Name::new(parent, format!("MockMultiQueue{}", index.into()));
        let (send, recv) = std::sync::mpsc::channel();
        Self {
            id: name.join(),
            name,
            send,
            recv: Mutex::new(Some(recv)),
            received: Arc::new(RwLock::new(vec![])),
            recv_limit,
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
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let recv = self.recv.lock().unwrap().take().unwrap();
        let received = self.received.clone();
        let recv_limit = self.recv_limit.clone();
        let handle = thread::spawn(move || {
            match recv_limit {
                Some(recv_limit) => {
                    let mut received_count = 0;
                    'main: loop {
                        match recv.recv() {
                            Ok(point) => {
                                received.write().unwrap().push(point);
                                received_count += 1;
                                if received_count >= recv_limit {
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
                                received.write().unwrap().push(point);
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
