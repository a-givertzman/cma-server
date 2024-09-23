#![allow(non_snake_case)]
use std::{collections::HashMap, fmt::Debug, str::FromStr, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, RwLock}, thread};
use log::{error, info, trace, warn};
use sal_sync::services::{
    entity::{name::Name, object::Object, point::{point::Point, point_tx_id::PointTxId}},
    service::{link_name::LinkName, service::Service, service_handles::ServiceHandles},
    subscription::{subscription_criteria::SubscriptionCriteria, subscriptions::Subscriptions}
};
use crate::services::{safe_lock::rwlock::SafeLock, services::Services};
///
/// - Receives points into the MPSC queue in the blocking mode
/// - If new point received, immediately sends it to the all subscribed consumers
/// - Keeps all consumers subscriptions in the single map:
pub struct MockMultiQueueMatch {
    id: String,
    name: Name,
    subscriptions: Arc<RwLock<Subscriptions>>,
    rxSend: HashMap<String, Sender<Point>>,
    rx_recv: Mutex<Option<Receiver<Point>>>,
    sendQueues: Vec<String>,
    services: Arc<RwLock<Services>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl MockMultiQueueMatch {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(parent: impl Into<String>, txQueues: Vec<String>, rxQueue: impl Into<String>, services: Arc<RwLock<Services>>) -> Self {
        let name = Name::new(parent, format!("MockMultiQueueMatch{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let (send, recv) = mpsc::channel();
        Self {
            id: name.join(),
            name: name.clone(),
            subscriptions: Arc::new(RwLock::new(Subscriptions::new(name))),
            rxSend: HashMap::from([(rxQueue.into(), send)]),
            rx_recv: Mutex::new(Some(recv)),
            sendQueues: txQueues,
            services,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
//
// 
impl Object for MockMultiQueueMatch {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for MockMultiQueueMatch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockMultiQueueMatch")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for MockMultiQueueMatch {
    //
    //
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        match self.rxSend.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    //
    fn subscribe(&mut self, receiverId: &str, points: &[SubscriptionCriteria]) -> (Sender<Point>, Receiver<Point>) {
        let (send, recv) = mpsc::channel();
        let receiverId = PointTxId::from_str(receiverId);
        if points.is_empty() {
            self.subscriptions.wlock(&self.id).add_broadcast(receiverId, send.clone());
        } else {
            for subscription_criteria in points {
                self.subscriptions.wlock(&self.id).add_multicast(receiverId, &subscription_criteria.destination(), send.clone());
            }
        }
        (send, recv)
    }
    //
    //
    fn unsubscribe(&mut self, receiverId: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        let receiverId = PointTxId::from_str(receiverId);
        for subscription_criteria in points {
            match self.subscriptions.wlock(&self.id).remove(&receiverId, &subscription_criteria.destination()) {
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
        let recv = self.rx_recv.lock().unwrap().take().unwrap();
        let subscriptions = self.subscriptions.clone();
        let mut staticSubscriptions: HashMap<usize, Sender<Point>> = HashMap::new();
        for sendQueue in &self.sendQueues {
            let txSend = self.services.rlock(&self_id).get_link(&LinkName::from_str(sendQueue).unwrap()).unwrap_or_else(|err| {
                panic!("{}.run | services.get_link error: {:#?}", self.id, err);
            });
            staticSubscriptions.insert(PointTxId::from_str(sendQueue), txSend);
        }
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            loop {
                let subscriptions = subscriptions.rlock(&self_id);
                match recv.recv() {
                    Ok(point) => {
                        let pointId = point.name();
                        trace!("{}.run | received: {:?}", self_id, point);
                        for (receiverId, sender) in subscriptions.iter(&pointId).chain(&staticSubscriptions) {
                            match receiverId != &point.tx_id() {
                                true => {
                                    match sender.send(point.clone()) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("{}.run | subscriptions '{}', receiver '{}' - send error: {:?}", self_id, pointId, receiverId, err);
                                        }
                                    };
                                }
                                false => {}
                            }
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
