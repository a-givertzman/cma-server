#![allow(non_snake_case)]
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, RwLock}, thread};
use log::{info, warn, error, trace};
use crate::{
    conf::point_config::name::Name, core_::{object::object::Object, point::{point_tx_id::PointTxId, point_type::PointType}}, services::{multi_queue::{subscription_criteria::SubscriptionCriteria, subscriptions::Subscriptions}, queue_name::QueueName, safe_lock::SafeLock, service::{service::Service, service_handles::ServiceHandles}, services::Services}
};
///
/// - Receives points into the MPSC queue in the blocking mode
/// - If new point received, immediately sends it to the all subscribed consumers
/// - Keeps all consumers subscriptions in the single map:
pub struct MockMultiQueue {
    id: String,
    name: Name,
    subscriptions: Arc<Mutex<Subscriptions>>,
    rxSend: HashMap<String, Sender<PointType>>,
    rxRecv: Vec<Receiver<PointType>>,
    sendQueues: Vec<String>,
    services: Arc<RwLock<Services>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl MockMultiQueue {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(parent: impl Into<String>, txQueues: Vec<String>, rxQueue: impl Into<String>, services: Arc<RwLock<Services>>) -> Self {
        let name = Name::new(parent, format!("MockMultiQueue{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        let (send, recv) = mpsc::channel();
        Self {
            id: name.join(),
            name: name.clone(),
            subscriptions: Arc::new(Mutex::new(Subscriptions::new(name))),
            rxSend: HashMap::from([(rxQueue.into(), send)]),
            rxRecv: vec![recv],
            sendQueues: txQueues,
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
impl Service for MockMultiQueue {
    //
    //
    fn get_link(&mut self, name: &str) -> Sender<PointType> {
        match self.rxSend.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    //
    fn subscribe(&mut self, receiverId: &str, points: &[SubscriptionCriteria]) -> (Sender<PointType>, Receiver<PointType>) {
        let (send, recv) = mpsc::channel();
        let receiverId = PointTxId::from_str(receiverId);
        if points.is_empty() {
            self.subscriptions.slock(&self.id).add_broadcast(receiverId, send.clone());
        } else {
            for subscription_criteria in points {
                self.subscriptions.slock(&self.id).add_multicast(receiverId, &subscription_criteria.destination(), send.clone());
            }
        }
        (send, recv)
    }
    //
    //
    fn unsubscribe(&mut self, receiverId: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        let receiverId = PointTxId::from_str(receiverId);
        for subscription_criteria in points {
            match self.subscriptions.slock(&self.id).remove(&receiverId, &subscription_criteria.destination()) {
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
    fn run(&mut self) -> Result<ServiceHandles, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let recv = self.rxRecv.pop().unwrap();
        let subscriptions = self.subscriptions.clone();
        let mut staticSubscriptions: HashMap<usize, Sender<PointType>> = HashMap::new();
        for sendQueue in &self.sendQueues {
            let txSend = self.services.rlock(&self_id).get_link(&QueueName::new(sendQueue)).unwrap_or_else(|err| {
                panic!("{}.run | services.get_link error: {:#?}", self.id, err);
            });
            staticSubscriptions.insert(PointTxId::from_str(sendQueue), txSend);
        }
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            loop {
                let subscriptions = subscriptions.lock().unwrap();
                match recv.recv() {
                    Ok(point) => {
                        let pointId = point.name();
                        trace!("{}.run | received: {:?}", self_id, point);
                        for (receiverId, sender) in subscriptions.iter(&pointId).chain(&staticSubscriptions) {
                            match sender.send(point.clone()) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("{}.run | subscriptions '{}', receiver '{}' - send error: {:?}", self_id, pointId, receiverId, err);
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
