#![allow(non_snake_case)]
use sal_sync::services::{entity::{Name, Object, {{Point, ToPoint}, PointTxId}}, service::{LinkName, Service}};
use std::{collections::HashMap, fmt::Debug, str::FromStr, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}, thread};
use testing::entities::test_value::Value;
use crate::{core_::RECV_TIMEOUT, services::{safe_lock::rwlock::SafeLock}};
///
/// 
pub struct MockRecvSendService {
    id: String,
    name: Name,
    rxSend: HashMap<String, Sender<Point>>,
    rx_recv: Owner<Receiver<Point>>,
    send_to: LinkName,
    services: Arc<Services>,
    test_data: Vec<Value>,
    sent: Arc<RwLock<Vec<Point>>>,
    received: Arc<RwLock<Vec<Point>>>,
    recvLimit: Option<usize>,
    exit: Arc<AtomicBool>,
}
//
// 
impl MockRecvSendService {
    pub fn new(parent: impl Into<String>, rxQueue: &str, send_to: &str, services: Arc<Services>, test_data: Vec<Value>, recvLimit: Option<usize>) -> Self {
        let name = Name::new(parent, format!("MockRecvSendService{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let (send, recv) = mpsc::channel::<Point>();
        Self {
            id: name.join(),
            name,
            rxSend: HashMap::from([(rxQueue.to_string(), send)]),
            rx_recv: Owner::new(recv),
            send_to: LinkName::from_str(send_to).unwrap(),
            services,
            test_data,
            sent: Arc::new(RwLock::new(vec![])),
            received: Arc::new(RwLock::new(vec![])),
            recvLimit,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
    pub fn sent(&self) -> Arc<RwLock<Vec<Point>>> {
        self.sent.clone()
    }
    ///
    /// 
    pub fn received(&self) -> Arc<RwLock<Vec<Point>>> {
        self.received.clone()
    }
}
//
// 
impl Object for MockRecvSendService {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for MockRecvSendService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockRecvSendService")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for MockRecvSendService {
    //
    //
    fn get_link(&self, name: &str) -> std::sync::mpsc::Sender<Point> {
        match self.rxSend.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let rx_recv = self.rx_recv.lock().unwrap().take().unwrap();
        let received = self.received.clone();
        let recvLimit = self.recvLimit.clone();
        let handle_recv = thread::Builder::new().name(format!("{}.run | Recv", self_id)).spawn(move || {
            log::info!("{}.run | Preparing thread Recv - ok", self_id);
            match recvLimit {
                Some(recvLimit) => {
                    let mut receivedCount = 0;
                    loop {
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {
                            Ok(point) => {
                                log::trace!("{}.run | received: {:?}", self_id, point);
                                received.write().unwrap().push(point);
                                receivedCount += 1;
                            }
                            Err(_) => {}
                        };
                        if receivedCount >= recvLimit {
                            break;
                        }
                        if exit.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                }
                None => {
                    loop {
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {
                            Ok(point) => {
                                log::trace!("{}.run | received: {:?}", self_id, point);
                                received.write().unwrap().push(point);
                            }
                            Err(_) => {}
                        };
                        if exit.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                }
            }
        });
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let txSend = self.services.get_link(&self.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.id, err);
        });
        let test_data = self.test_data.clone();
        let sent = self.sent.clone();
        let handle_send = thread::Builder::new().name(format!("{}.run | Send", self_id)).spawn(move || {
            log::info!("{}.run | Preparing thread Send - ok", self_id);
            let txId = PointTxId::from_str(&self_id);
            for value in test_data.iter() {
                let point = value.to_point(txId,&format!("{}/test", self_id));
                match txSend.send(point.clone()) {
                    Ok(_) => {
                        log::trace!("{}.run | send: {:?}", self_id, point);
                        sent.write().unwrap().push(point);
                    }
                    Err(err) => {
                        log::warn!("{}.run | send error: {:?}", self_id, err);
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
        match (handle_recv, handle_send) {
            (Ok(handle_recv), Ok(handle_send)) => Ok(ServiceHandles::new(vec![
                (format!("{}/read", self.id), handle_recv),
                (format!("{}/write", self.id), handle_send),
                ])),
            // TODO Exit 'write if read returns error'
            (Ok(_handle_recv), Err(err)) => Err(format!("{}.run | Error starting inner thread 'recv': {:#?}", self.id, err)),
            // TODO Exit 'read if write returns error'
            (Err(err), Ok(_handle_send)) => Err(format!("{}.run | Error starting inner thread 'send': {:#?}", self.id, err)),
            (Err(read_err), Err(write_err)) => Err(format!("{}.run | Error starting inner thread: \n\t  recv: {:#?}\n\t send: {:#?}", self.id, read_err, write_err)),
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
