use coco::Stack;
use sal_core::error::Error;
use sal_sync::services::{
    entity::{Name, Object, {Point, PointConfig, PointTxId}}, safe_lock::rwlock::SafeLock, service::{Service, ServiceCycle}, services::Services, subscription::SubscriptionCriteria
};
use std::{
    collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, RecvTimeoutError, Sender}, Arc, Mutex, RwLock}, thread::{self, JoinHandle}, time::Duration,
};
use concat_string::concat_string;
use crate::{
    core_::constants::constants::RECV_TIMEOUT,
    conf::task_config::TaskConfig, 
    services::task::task_nodes::TaskNodes,
};
///
/// Task implements entity, which provides cyclically (by event) executing calculations
///  - executed in the cycle mode (current impl)
///  - executed event mode (future impl..)
///  - has some number of functions / variables / metrics or additional entities
pub struct Task {
    id: String,
    name: Name,
    in_send: HashMap<String, Sender<Point>>,
    rx_recv: Mutex<Option<Receiver<Point>>>,
    services: Arc<RwLock<Services>>,
    conf: TaskConfig,
    handle: Stack<JoinHandle<()>>,
    exit: Arc<AtomicBool>,
}
//
//
impl Task {
    ///
    /// Creates new instance of [Task]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: TaskConfig, services: Arc<RwLock<Services>>) -> Task {
        let (send, recv) = mpsc::channel();
        Task {
            id: conf.name.join(),
            name: conf.name.clone(),
            // in_send: HashMap::from([(conf.rx.clone(), send)]),
            in_send: HashMap::from([("in-send".to_owned(), send)]),
            rx_recv: Mutex::new(Some(recv)),
            services,
            conf,
            handle: Stack::new(),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    ///
    fn subscriptions(&mut self, conf: &TaskConfig, services: &Arc<RwLock<Services>>) -> Option<(String, Vec<SubscriptionCriteria>)> {
        if conf.subscribe.is_empty() {
            None
        } else {
            log::debug!("{}.subscriptions | requesting points...", self.id);
            let mut self_points = self.conf.points();
            let mut points = services.rlock(&self.id).points(&self.id).then(
                |points| points,
                |err| {
                    log::error!("{}.subscriptions | Requesting Points error: {:?}", self.id, err);
                    vec![]
                },
            );
            points.append(&mut self_points);
            log::debug!("{}.subscriptions | rceived points: {:#?}", self.id, points.len());
            log::debug!(
                "{}.subscriptions | rceived points: {:#?}",
                self.id,
                points.iter().map(|p| concat_string!(p.id.to_string(), " | ", p.type_.to_string(), " | ", p.name)).collect::<Vec<String>>(),
            );
            log::debug!("{}.subscriptions | conf.subscribe: {:#?}", self.id, conf.subscribe);
            let subscriptions = conf.subscribe.with(&points);
            log::trace!("{}.subscriptions | subscriptions: {:#?}", self.id, subscriptions);
            if subscriptions.len() > 1 {
                panic!("{}.subscriptions | Error. Task does not supports multiple subscriptions for now: {:#?}.\n\tTry to use single subscription.", self.id, subscriptions);
            } else {
                let subscriptions_first = subscriptions.clone().into_iter().next();
                match subscriptions_first {
                    Some((service_name, Some(points))) => {
                        Some((service_name, points))
                    }
                    Some((_, None)) => {
                        log::warn!("{}.subscriptions | Error. Task subscription configuration error / empty in: {:#?}", self.id, subscriptions);
                        None
                    }
                    None => panic!("{}.subscriptions | Error. Task subscription configuration error in: {:#?}", self.id, subscriptions),
                }
            }
        }
    }
    ///
    ///
    fn subscribe(&mut self, subscriptions: &Option<(String, Vec<SubscriptionCriteria>)>, services: &Arc<RwLock<Services>>) -> Receiver<Point> {
        match subscriptions {
            Some((service_name, points)) => {
                let (_, rx_recv) = services.wlock(&self.id).subscribe(
                    service_name,
                    &self.name.join(),
                    points,
                );
                rx_recv
            }
            None => {
                match self.rx_recv.lock() {
                    Ok(mut rx_recv) => rx_recv.take().unwrap(),
                    Err(err) => panic!("{}.subscribe | self.rx_recv - is not initialized, \n\t error: {:#?}", self.id, err),
                }
            }
        }
    }
}
//
//
impl Object for Task {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for Task {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Task")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for Task {
    //
    //
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        // match self.in_send.get(name) {
        match self.in_send.iter().next() {
            Some((_, send)) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    //
    fn run(&mut self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.id);
        log::trace!("{}.run | Self tx_id: {}", self.id, PointTxId::from_str(&self.id));
        let self_id = self.id.clone();
        let self_name = self.name.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let (cyclic, cycle_interval, recv_timeout) = match conf.cycle {
            Some(interval) => (interval > Duration::ZERO, interval, interval),
            None => (false, Duration::ZERO, RECV_TIMEOUT),
        };
        let subscriptions = self.subscriptions(&conf, &services);
        let rx_recv = self.subscribe(&subscriptions, &services);
        let handle = thread::Builder::new().name(format!("{} - main", self_id)).spawn(move || {
            let mut cycle = ServiceCycle::new(&self_id, cycle_interval);
            let mut task_nodes = TaskNodes::new(&self_id);
            task_nodes.build_nodes(&self_name, conf, services.clone());
            log::trace!("{}.run | taskNodes: {:#?}", self_id, task_nodes);
            'main: loop {
                log::trace!("{}.run | calculation step...", self_id);
                if cyclic {
                    cycle.start();
                    match rx_recv.recv_timeout(recv_timeout) {
                        Ok(point) => {
                            log::debug!("{}.run | point: {:?}", self_id, &point);
                            task_nodes.eval(point);
                            log::debug!("{}.run | calculation step - done ({:?})", self_id, cycle.elapsed());
                            cycle.wait();
                        }
                        Err(err) => {
                            match err {
                                RecvTimeoutError::Timeout => log::trace!("{}.run | Receive error: {:?}", self_id, err),
                                RecvTimeoutError::Disconnected => {
                                    log::error!("{}.run | Error receiving from queue: {:?}", self_id, err);
                                    break 'main;
                                }
                            }
                        }
                    };
                } else {
                    match rx_recv.recv() {
                        Ok(point) => {
                            log::debug!("{}.run | point: {:?}", self_id, &point);
                            task_nodes.eval(point);
                            log::debug!("{}.run | calculation step - done ({:?})", self_id, cycle.elapsed());
                        }
                        Err(err) => {
                            log::error!("{}.run | Error receiving from queue: {:?}", self_id, err);
                            break 'main;
                        }
                    };
                }
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            };
            if let Some((service_name, points)) = subscriptions {
                if let Err(err) = services.wlock(&self_id).unsubscribe(&service_name,&self_name.join(), &points) {
                    log::error!("{}.run | Unsubscribe error: {:#?}", self_id, err);
                }
            }
            log::info!("{}.run | Exit", self_id);
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.id);
                self.handle.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&self.id, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn points(&self) -> Vec<PointConfig> {
        self.conf.points()
    }
    //
    //
    fn wait(&self) -> sal_sync::services::future::Future<()> {
        let dbg = self.id.clone();
        let (future, sink) = sal_sync::services::future::Future::new();
        if let Some(handle) = self.handle.pop() {
            std::thread::spawn(move|| {
                if let Err(err) = handle.join() {
                    log::warn!("{dbg}.wait | Error: {:?}", err);
                }
                sink.add(());
            });
        }
        future
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
        log::debug!("{}.run | Exit: {}", self.id, self.exit.load(Ordering::SeqCst));
    }
}
