use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, RwLock}, thread};
use log::{info, warn, error, trace};
use sal_sync::services::{
    entity::{name::Name, object::Object, point::{point::Point, point_tx_id::PointTxId}},
    service::{link_name::LinkName, service::Service, service_handles::ServiceHandles},
    subscription::{subscription_criteria::SubscriptionCriteria, subscriptions::Subscriptions},
};
use crate::services::{safe_lock::SafeLock, services::Services};
///
/// - Receives points into the MPSC queue in the blocking mode
/// - If new point received, immediately sends it to the all subscribed consumers
/// - Keeps all consumers subscriptions in the single map:
pub struct MockMultiQueue {
    id: String,
    name: Name,
    subscriptions: Arc<Mutex<Subscriptions>>,
    rx_send: HashMap<String, Sender<Point>>,
    rx_recv: Vec<Receiver<Point>>,
    send_queues: Vec<String>,
    services: Arc<RwLock<Services>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl MockMultiQueue {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(parent: impl Into<String>, tx_queues: Vec<String>, rx_queue: impl Into<String>, services: Arc<RwLock<Services>>) -> Self {
        let name = Name::new(parent, format!("MockMultiQueue{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let (send, recv) = mpsc::channel();
        Self {
            id: name.join(),
            name: name.clone(),
            subscriptions: Arc::new(Mutex::new(Subscriptions::new(name))),
            rx_send: HashMap::from([(rx_queue.into(), send)]),
            rx_recv: vec![recv],
            send_queues: tx_queues,
            services,
            exit: Arc::new(AtomicBool::new(false)),
        }
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
unsafe impl Send for MockMultiQueue {}
unsafe impl Sync for MockMultiQueue {}
//
// 
impl Service for MockMultiQueue {
    //
    //
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        match self.rx_send.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    //
    fn subscribe(&mut self, receiver_id: &str, points: &[SubscriptionCriteria]) -> (Sender<Point>, Receiver<Point>) {
        let (send, recv) = mpsc::channel();
        let receiver_id = PointTxId::from_str(receiver_id);
        if points.is_empty() {
            self.subscriptions.slock(&self.id).add_broadcast(receiver_id, send.clone());
        } else {
            for subscription_criteria in points {
                self.subscriptions.slock(&self.id).add_multicast(receiver_id, &subscription_criteria.destination(), send.clone());
            }
        }
        (send, recv)
    }
    //
    //
    fn unsubscribe(&mut self, receiver_id: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        let receiver_id = PointTxId::from_str(receiver_id);
        for subscription_criteria in points {
            match self.subscriptions.slock(&self.id).remove(&receiver_id, &subscription_criteria.destination()) {
                Ok(_) => {}
                Err(err) => {
                    return Err(err)
                }
            }
        }
        Ok(())
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let recv = self.rx_recv.pop().unwrap();
        let subscriptions = self.subscriptions.clone();
        let mut static_subscriptions: HashMap<usize, Sender<Point>> = HashMap::new();
        for send_queue in &self.send_queues {
            let tx_send = self.services.rlock(&self_id).get_link(&LinkName::new(send_queue)).unwrap_or_else(|err| {
                panic!("{}.run | services.get_link error: {:#?}", self.id, err);
            });
            static_subscriptions.insert(PointTxId::from_str(send_queue), tx_send);
        }
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            loop {
                let subscriptions = subscriptions.lock().unwrap();
                match recv.recv() {
                    Ok(point) => {
                        let point_id = point.name();
                        trace!("{}.run | received: {:?}", self_id, point);
                        for (receiver_id, sender) in subscriptions.iter(&point_id).chain(&static_subscriptions) {
                            match sender.send(point.clone()) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("{}.run | subscriptions '{}', receiver '{}' - send error: {:?}", self_id, point_id, receiver_id, err);
                                }
                            };
                        }
                    }
                    Err(err) => {
                        warn!("{}.run | recv error: {:?}", self_id, err);
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break;
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
static COUNT: AtomicUsize = AtomicUsize::new(0);
