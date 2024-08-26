use std::{collections::HashMap, fmt::Debug, fs, io::Write, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}, Arc, Mutex, RwLock}, thread};
use log::{debug, error, info, trace, warn};
use crate::{
    conf::{multi_queue_config::MultiQueueConfig, point_config::name::Name}, 
    core_::{constants::constants::RECV_TIMEOUT, object::object::Object, point::{point_tx_id::PointTxId, point::Point}}, 
    services::{
        multi_queue::subscription_criteria::SubscriptionCriteria, queue_name::QueueName, safe_lock::SafeLock, service::{service::Service, service_handles::ServiceHandles}, services::Services
    },
};
use concat_string::concat_string;
use super::subscriptions::Subscriptions;
///
/// - Receives points into the MPSC queue in the blocking mode
/// - If new point received, immediately sends it to the all subscribed consumers
/// - Keeps all consumers subscriptions in the single map:
pub struct MultiQueue {
    id: String,
    name: Name,
    subscriptions: Arc<Mutex<Subscriptions>>,
    subscriptions_changed: Arc<AtomicBool>,
    rx_send: HashMap<String, Sender<Point>>,
    rx_recv: Vec<Receiver<Point>>,
    send_queues: Vec<QueueName>,
    services: Arc<RwLock<Services>>,
    receiver_dictionary: HashMap<usize, String>,
    exit: Arc<AtomicBool>,
}
//
//
impl MultiQueue {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: MultiQueueConfig, services: Arc<RwLock<Services>>) -> Self {
        let self_id = format!("{}", conf.name);
        let (send, recv) = mpsc::channel();
        let send_queues = conf.send_to;
        Self {
            id: self_id.clone(),
            name: conf.name.clone(),
            subscriptions: Arc::new(Mutex::new(Subscriptions::new(self_id))),
            subscriptions_changed: Arc::new(AtomicBool::new(false)),
            rx_send: HashMap::from([(conf.rx, send)]),
            rx_recv: vec![recv],
            send_queues,
            services,
            receiver_dictionary: HashMap::new(),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Writes Subscription's to the log file 
    fn log(&self, name: &str, receiver_name: &str, rceiver_hash: usize, points: &[SubscriptionCriteria]) {
        let path = concat_string!("./logs", self.name.join(), name);
        let destinations: Vec<String> = points.iter().map(|cr| {cr.destination()}).collect();
        match fs::OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                f.write_fmt(format_args!("\n\n\t{} ({})", receiver_name, rceiver_hash)).unwrap();
                match serde_json::to_writer_pretty(f, &destinations) {
                    Ok(_) => {}
                    Err(err) => {
                        if log::max_level() >= log::LevelFilter::Trace {
                            warn!("{}.log | Error writing to file: '{}'\n\terror: {:?}", self.id, path, err)
                        }
                    }
                }
            }
            Err(err) => {
                if log::max_level() >= log::LevelFilter::Trace {
                    warn!("{}.log | Error open file: '{}'\n\terror: {:?}", self.id, path, err)
                }
            }
        }
    }
    ///
    /// Writes Point's to the log file 
    fn log_point(self_id: &str, parent: &Name, point_id: &str, point: &Point) {
        let path = concat_string!("./logs", parent.join(), "/points.log");
        match fs::OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                f.write_fmt(format_args!("'{}': {:?}\n",point_id, point)).unwrap();
            }
            Err(err) => {
                if log::max_level() >= log::LevelFilter::Trace {
                    warn!("{}.log | Error open file: '{}'\n\terror: {:?}", self_id, path, err)
                }
            }
        }
    }
}
//
//
impl Object for MultiQueue {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for MultiQueue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MultiQueue")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for MultiQueue {
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
    fn subscribe(&mut self, receiver_name: &str, points: &[SubscriptionCriteria]) -> (Sender<Point>, Receiver<Point>) {
        let (send, recv) = mpsc::channel();
        let receiver_hash = PointTxId::from_str(receiver_name);
        self.receiver_dictionary.insert(receiver_hash, receiver_name.to_string());
        if points.is_empty() {
            self.subscriptions.slock(&self.id).add_broadcast(receiver_hash, send.clone());
            self.log("/broadcast.log", receiver_name, receiver_hash, points);
            debug!("{}.subscribe | Broadcast subscription registered, receiver: \n\t{} ({})", self.id, receiver_name, receiver_hash);
        } else {
            for subscription_criteria in points {
                self.subscriptions.slock(&self.id).add_multicast(receiver_hash, &subscription_criteria.destination(), send.clone());
            }
            self.log("/multicast.log", receiver_name, receiver_hash, points);
            debug!("{}.subscribe | Multicast subscription registered, receiver: \n\t{} ({}) \n\tpoints: {:#?}", self.id, receiver_name, receiver_hash, points.len());
            trace!("{}.subscribe | Multicast subscription registered, receiver: \n\t{} ({}) \n\tpoints: {:#?}", self.id, receiver_name, receiver_hash, points);
        }
        self.subscriptions_changed.store(true, Ordering::SeqCst);
        (send, recv)
    }
    //
    //
    fn extend_subscription(&mut self, receiver_name: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        let receiver_hash = PointTxId::from_str(receiver_name);
        if points.is_empty() {
            let message = format!("{}.extend_subscription | Broadcast subscription can't be extended, receiver: {} ({})", self.id, receiver_name, receiver_hash);
            warn!("{}", message);
            Err(message)
        } else {
            let mut message = String::new();
            for subscription_criteria in points {
                trace!("{}.extend_subscription | Multicast subscription extending for receiver: {} ({})...", self.id, receiver_name, receiver_hash);
                if let Err(err) = self.subscriptions.slock(&self.id).extend_multicast(receiver_hash, &subscription_criteria.destination()) {
                    message = concat_string!(message, err, "\n");
                };
            }
            self.log("/multicast.log", receiver_name, receiver_hash, points);
            if message.is_empty() {
                debug!("{}.extend_subscription | Multicast subscription extended, receiver: {} ({})", self.id, receiver_name, receiver_hash);
                self.subscriptions_changed.store(true, Ordering::SeqCst);
                Ok(())
            } else {
                debug!("{}.extend_subscription | Multicast subscription extended, receiver: {} ({}) \n\t with errors: {:?}", self.id, receiver_name, receiver_hash, message);
                self.subscriptions_changed.store(true, Ordering::SeqCst);
                Err(message)
            }
        }
    }
    //
    //
    fn unsubscribe(&mut self, receiver_name: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        let mut changed = false;
        let receiver_hash = PointTxId::from_str(receiver_name);
        if points.is_empty() {
            match self.subscriptions.slock(&self.id).remove_all(&receiver_hash) {
                Ok(_) => {
                    self.receiver_dictionary.remove(&receiver_hash);
                    changed |= true;
                    debug!("{}.unsubscribe | Broadcast subscription removed, receiver: {} ({})", self.id, receiver_name, receiver_hash);
                }
                Err(err) => {
                    return Err(err)
                }
            }
        } else {
            for subscription_criteria in points {
                match self.subscriptions.slock(&self.id).remove(&receiver_hash, &subscription_criteria.destination()) {
                    Ok(_) => {
                        self.receiver_dictionary.remove(&receiver_hash);
                        changed |= true;
                        debug!("{}.unsubscribe | Multicat subscription '{}' removed, receiver: {} ({})", self.id, subscription_criteria.destination(), receiver_name, receiver_hash);
                    }
                    Err(err) => {
                        return Err(err)
                    }
                }
            }
        }
        if changed {
            self.subscriptions_changed.store(true, Ordering::SeqCst);
        }
        Ok(())
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let self_name = self.name.clone();
        let exit = self.exit.clone();
        let recv = self.rx_recv.pop().unwrap();
        let subscriptions_ref = self.subscriptions.clone();
        let subscriptions_changed = self.subscriptions_changed.clone();
        // let receiver_dictionary = self.receiver_dictionary.clone();
        for receiver_name in &self.send_queues {
            let send = self.services.rlock(&self_id).get_link(receiver_name).unwrap_or_else(|err| {
                panic!("{}.run | services.get_link error: {:#?}", self_id, err);
            });
            let receiver_hash = PointTxId::from_str(&receiver_name.name());
            self.subscriptions.slock(&self_id).add_broadcast(receiver_hash, send.clone());
            debug!("{}.run | Broadcast subscription registered, receiver: \n\t{} ({})", self.id, receiver_name, receiver_hash);
        }
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            let mut subscriptions = subscriptions_ref.slock(&self_id).clone();
            loop {
                if subscriptions_changed.load(Ordering::Relaxed) {
                    subscriptions_changed.store(false, Ordering::SeqCst);
                    debug!("{}.run | Subscriptions changes detected", self_id);
                    subscriptions = subscriptions_ref.slock(&self_id).clone();
                }
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        let point_id = SubscriptionCriteria::new(&point.name(), point.cot()).destination();
                        trace!("{}.run | received: \n\t{:?}", self_id, point);
                        Self::log_point(&self_id, &self_name, &point_id, &point);
                        for (receiver_hash, sender) in subscriptions.iter(&point_id) {
                            if receiver_hash != &point.tx_id() {
                                match sender.send(point.clone()) {
                                    Ok(_) => {
                                        trace!("{}.run | sent to '{}' point: {:?}", self_id, receiver_hash, point);
                                    }
                                    Err(err) => {
                                        error!("{}.run | subscriptions '{}', receiver '{}' - send error: {:?}", self_id, point_id, receiver_hash, err);
                                    }
                                };
                            }
                        }
                    }
                    Err(err) => {
                        trace!("{}.run | recv timeout: {:?}", self_id, err);
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    subscriptions_ref.slock(&self_id).exit();
                    break;
                }
            }
            info!("{}.run | Exit", self_id);
        });
        match handle {
            Ok(handle) => {
                info!("{}.run | Started", self.id);
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
