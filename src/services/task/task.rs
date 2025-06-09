use coco::Stack;
use sal_core::{dbg::Dbg, error::Error};
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
    dbg: Dbg,
    name: Name,
    in_send: HashMap<String, Sender<Point>>,
    rx_recv: Mutex<Option<Receiver<Point>>>,
    services: Arc<RwLock<Services>>,
    conf: TaskConfig,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
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
            dbg: Dbg::new(conf.name.parent(), conf.name.me()),
            name: conf.name.clone(),
            // in_send: HashMap::from([(conf.rx.clone(), send)]),
            in_send: HashMap::from([("in-send".to_owned(), send)]),
            rx_recv: Mutex::new(Some(recv)),
            services,
            conf,
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    ///
    fn subscriptions_(&self, conf: &TaskConfig, services: &Arc<RwLock<Services>>) -> Option<(String, Vec<SubscriptionCriteria>)> {
        if conf.subscribe.is_empty() {
            None
        } else {
            log::debug!("{}.subscriptions | requesting points...", self.dbg);
            let mut self_points = self.conf.points();
            let mut points = services.rlock(&self.dbg).points(&self.dbg).then(
                |points| points,
                |err| {
                    log::error!("{}.subscriptions | Requesting Points error: {:?}", self.dbg, err);
                    vec![]
                },
            );
            points.append(&mut self_points);
            log::debug!("{}.subscriptions | rceived points: {:#?}", self.dbg, points.len());
            log::debug!(
                "{}.subscriptions | rceived points: {:#?}",
                self.dbg,
                points.iter().map(|p| concat_string!(p.id.to_string(), " | ", p.type_.to_string(), " | ", p.name)).collect::<Vec<String>>(),
            );
            log::debug!("{}.subscriptions | conf.subscribe: {:#?}", self.dbg, conf.subscribe);
            let subscriptions = conf.subscribe.with(&points);
            log::trace!("{}.subscriptions | subscriptions: {:#?}", self.dbg, subscriptions);
            if subscriptions.len() > 1 {
                panic!("{}.subscriptions | Error. Task does not supports multiple subscriptions for now: {:#?}.\n\tTry to use single subscription.", self.dbg, subscriptions);
            } else {
                let subscriptions_first = subscriptions.clone().into_iter().next();
                match subscriptions_first {
                    Some((service_name, Some(points))) => {
                        Some((service_name, points))
                    }
                    Some((_, None)) => {
                        log::warn!("{}.subscriptions | Error. Task subscription configuration error / empty in: {:#?}", self.dbg, subscriptions);
                        None
                    }
                    None => panic!("{}.subscriptions | Error. Task subscription configuration error in: {:#?}", self.dbg, subscriptions),
                }
            }
        }
    }
    ///
    ///
    fn subscribe_(&self, subscriptions: &Option<(String, Vec<SubscriptionCriteria>)>, services: &Arc<RwLock<Services>>) -> Receiver<Point> {
        match subscriptions {
            Some((service_name, points)) => {
                let (_, rx_recv) = services.wlock(&self.dbg).subscribe(
                    service_name,
                    &self.name.join(),
                    points,
                );
                rx_recv
            }
            None => {
                match self.rx_recv.lock() {
                    Ok(mut rx_recv) => rx_recv.take().unwrap(),
                    Err(err) => panic!("{}.subscribe | self.rx_recv - is not initialized, \n\t error: {:#?}", self.dbg, err),
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
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for Task {
    //
    //
    fn get_link(&self, name: &str) -> Sender<Point> {
        // match self.in_send.get(name) {
        match self.in_send.iter().next() {
            Some((_, send)) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.dbg, name),
        }
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        log::trace!("{}.run | Self tx_id: {}", self.dbg, PointTxId::from_str(&self.name.join()));
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let (cyclic, cycle_interval, recv_timeout) = match conf.cycle {
            Some(interval) => (interval > Duration::ZERO, interval, interval),
            None => (false, Duration::ZERO, RECV_TIMEOUT),
        };
        let subscriptions = self.subscriptions_(&conf, &services);
        let rx_recv = self.subscribe_(&subscriptions, &services);
        let handle = thread::Builder::new().name(format!("{} - main", dbg)).spawn(move || {
            let mut cycle = ServiceCycle::new(&dbg, cycle_interval);
            let mut task_nodes = TaskNodes::new(&dbg);
            task_nodes.build_nodes(&self_name, conf, services.clone());
            log::trace!("{}.run | taskNodes: {:#?}", dbg, task_nodes);
            'main: loop {
                log::trace!("{}.run | calculation step...", dbg);
                if cyclic {
                    cycle.start();
                    match rx_recv.recv_timeout(recv_timeout) {
                        Ok(point) => {
                            log::debug!("{}.run | point: {:?}", dbg, &point);
                            task_nodes.eval(point);
                            log::debug!("{}.run | calculation step - done ({:?})", dbg, cycle.elapsed());
                            cycle.wait();
                        }
                        Err(err) => {
                            match err {
                                RecvTimeoutError::Timeout => log::trace!("{}.run | Receive error: {:?}", dbg, err),
                                RecvTimeoutError::Disconnected => {
                                    log::error!("{}.run | Error receiving from queue: {:?}", dbg, err);
                                    break 'main;
                                }
                            }
                        }
                    };
                } else {
                    match rx_recv.recv() {
                        Ok(point) => {
                            log::debug!("{}.run | point: {:?}", dbg, &point);
                            task_nodes.eval(point);
                            log::debug!("{}.run | calculation step - done ({:?})", dbg, cycle.elapsed());
                        }
                        Err(err) => {
                            log::error!("{}.run | Error receiving from queue: {:?}", dbg, err);
                            break 'main;
                        }
                    };
                }
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            };
            if let Some((service_name, points)) = subscriptions {
                if let Err(err) = services.wlock(&dbg).unsubscribe(&service_name,&self_name.join(), &points) {
                    log::error!("{}.run | Unsubscribe error: {:#?}", dbg, err);
                }
            }
            log::info!("{}.run | Exit", dbg);
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handle.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
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
    fn wait(&self) -> Result<(), Error> {
        while !self.handle.is_empty() {
            if let Some(handle) = self.handle.pop() {
                if let Err(err) = handle.join() {
                    log::warn!("{}.wait | Error: {:?}", self.dbg, err);
                    return Err(Error::new(&self.dbg, "wait").pass(format!("{:?}", err)));
                }
            }
        }
        self.is_finished.store(true, Ordering::SeqCst);
        Ok(())
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.is_finished.load(Ordering::SeqCst)
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
        log::debug!("{}.run | Exit: {}", self.dbg, self.exit.load(Ordering::SeqCst));
    }
}
