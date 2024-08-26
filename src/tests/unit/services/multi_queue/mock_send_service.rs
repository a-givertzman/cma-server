#![allow(non_snake_case)]
use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, Mutex, RwLock}, thread, time::Duration};
use log::{info, warn, trace};
use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{link_name::LinkName, service::Service, service_handles::ServiceHandles}};
use testing::entities::test_value::Value;
use crate::services::{safe_lock::SafeLock, services::Services};
///
///
pub struct MockSendService {
    id: String,
    name: Name,
    send_to: LinkName,
    services: Arc<RwLock<Services>>,
    test_data: Vec<Value>,
    sent: Arc<Mutex<Vec<Point>>>,
    delay: Option<Duration>,
    exit: Arc<AtomicBool>,
}
//
// 
impl MockSendService {
    pub fn new(parent: impl Into<String>, send_to: &str, services: Arc<RwLock<Services>>, test_data: Vec<Value>, delay: Option<Duration>) -> Self {
        let name = Name::new(parent, format!("MockSendService{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        Self {
            id: name.join(),
            name,
            send_to: LinkName::new(send_to),
            services,
            test_data,
            sent: Arc::new(Mutex::new(vec![])),
            delay,
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
    pub fn sent(&self) -> Arc<Mutex<Vec<Point>>> {
        self.sent.clone()
    }
}
//
// 
impl Object for MockSendService {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for MockSendService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockSendService")
            .field("id", &self.id)
            .finish()
    }
}
//
// 
impl Service for MockSendService {
    //
    //
    fn get_link(&mut self, _name: &str) -> std::sync::mpsc::Sender<Point> {
        panic!("{}.get_link | Does not support get_link", self.id())
        // match self.rxSend.get(name) {
        //     Some(send) => send.clone(),
        //     None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        // }
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let txSend = self.services.rlock(&self_id).get_link(&self.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.id, err);
        });
        let test_data = self.test_data.clone();
        let sent = self.sent.clone();
        let delay = self.delay.clone();
        let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            for value in test_data {
                let point = value.to_point(0,&format!("{}/test", self_id));
                match txSend.send(point.clone()) {
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
                match delay {
                    Some(duration) => {
                        thread::sleep(duration);
                    }
                    None => {}
                }
            }
        });
        match handle {
            Ok(handle) => {
                info!("{}.run | Starting - ok", self.id);
                Ok(ServiceHandles::new(vec![(self.id.clone(), handle)]))
            }
            Err(err) => {
                let message = format!("{}.run | Start failed: {:#?}", self.id, err);
                warn!("{}", message);
                Err(message)
            }
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
pub static COUNT: AtomicUsize = AtomicUsize::new(0);
