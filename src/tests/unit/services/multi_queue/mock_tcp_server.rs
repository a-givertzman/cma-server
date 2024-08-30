use log::{info, warn, debug, trace};
use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, Mutex}, thread};
use sal_sync::services::{
    entity::{name::Name, object::Object, point::{point::{Point, ToPoint}, point_tx_id::PointTxId}},
    service::{link_name::LinkName, service::Service, service_handles::ServiceHandles},
};
use testing::entities::test_value::Value;
use crate::{core_::constants::constants::RECV_TIMEOUT, services::{safe_lock::SafeLock, services::Services}};
///
///
pub struct MockTcpServer {
    id: String,
    name: Name,
    multi_queue: LinkName,
    services: Arc<Mutex<Services>>,
    test_data: Vec<Value>,
    sent: Arc<Mutex<Vec<Point>>>,
    received: Arc<Mutex<Vec<Point>>>,
    recv_limit: Option<usize>,
    exit: Arc<AtomicBool>,
}
//
// 
impl MockTcpServer {
    pub fn new(parent: impl Into<String>, multi_queue: &str, services: Arc<Mutex<Services>>, test_data: Vec<Value>, recv_limit: Option<usize>) -> Self {
        let name = Name::new(parent, format!("MockTcpServer{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        Self {
            id: name.join(),
            name,
            multi_queue: LinkName::new(multi_queue),
            services,
            test_data,
            sent: Arc::new(Mutex::new(vec![])),
            received: Arc::new(Mutex::new(vec![])),
            recv_limit,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// 
    pub fn id(&self) -> String {
        self.id.clone()
    }
    ///
    /// 
    // pub fn sent(&self) -> Arc<Mutex<Vec<PointType>>> {
    //     self.sent.clone()
    // }
    ///
    /// 
    pub fn received(&self) -> Arc<Mutex<Vec<Point>>> {
        self.received.clone()
    }
}
//
// 
impl Object for MockTcpServer {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for MockTcpServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockTcpServer")
            .field("id", &self.id)
            .finish()
    }
}
//
//
unsafe impl Send for MockTcpServer {}
unsafe impl Sync for MockTcpServer {}
//
// 
impl Service for MockTcpServer {
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let mq_service_name = self.multi_queue.service().unwrap();
        debug!("{}.run | Lock services...", self_id);
        let (_, rx_recv) = self.services.wlock(&self_id).subscribe(&mq_service_name, &self_id, &vec![]);
        let tx_send = self.services.rlock(&self_id).get_link(&self.multi_queue).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self_id, err);
        });
        debug!("{}.run | Lock services - ok", self_id);
        let received = self.received.clone();
        let recv_limit = self.recv_limit.clone();
        let handle_recv = thread::Builder::new().name(format!("{}.run | Recv", self_id)).spawn(move || {
            info!("{}.run | Preparing thread Recv - ok", self_id);
            match recv_limit {
                Some(recv_limit) => {
                    let mut received_count = 0;
                    loop {
                        match rx_recv.recv_timeout(RECV_TIMEOUT) {
                            Ok(point) => {
                                trace!("{}.run | received: {:?}", self_id, point);
                                received.lock().unwrap().push(point);
                                received_count += 1;
                            }
                            Err(_) => {}
                        };
                        if received_count >= recv_limit {
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
                                trace!("{}.run | received: {:?}", self_id, point);
                                received.lock().unwrap().push(point);
                            }
                            Err(_) => {}
                        };
                        if exit.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                }
            }
            info!("{}.run | Exit thread Recv", self_id);
        });
        let self_id = self.id.clone();
        let tx_id = PointTxId::from_str(&self_id);
        let exit = self.exit.clone();
        let test_data = self.test_data.clone();
        let sent = self.sent.clone();
        let handle_send = thread::Builder::new().name(format!("{}.run | Send", self_id)).spawn(move || {
            info!("{}.run | Preparing thread Send - ok", self_id);
            for value in test_data.iter() {
                let point = value.to_point(tx_id,&format!("{}/test", self_id));
                match tx_send.send(point.clone()) {
                    Ok(_) => {
                        trace!("{}.run | send: {:?}", self_id, point);
                        sent.lock().unwrap().push(point);
                    }
                    Err(err) => {
                        warn!("{}.run | send error: {:?}", self_id, err);
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
            info!("{}.run | Exit thread Send", self_id);
        });
        match (handle_recv, handle_send) {
            (Ok(handle_recv), Ok(handle_send)) => Ok(ServiceHandles::new(vec![
                (format!("{}/read", self.id), handle_recv),
                (format!("{}/write", self.id), handle_send),
                ])),
            // TODO Exit 'write if read returns error'
            (Ok(_handle_recv), Err(err)) => Err(format!("{}.run | Error starting inner thread 'send': {:#?}", self.id, err)),
            // TODO Exit 'read if write returns error'
            (Err(err), Ok(_handle_send)) => Err(format!("{}.run | Error starting inner thread 'recv': {:#?}", self.id, err)),
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
