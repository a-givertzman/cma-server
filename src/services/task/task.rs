use sal_sync::services::{
    entity::{name::Name, object::Object, point::{point::Point, point_config::PointConfig, point_tx_id::PointTxId}},
    service::{service::Service, service_cycle::ServiceCycle, service_handles::ServiceHandles},
    subscription::subscription_criteria::SubscriptionCriteria,
};
use std::{
    collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, Ordering}, 
    mpsc::{self, Receiver, RecvTimeoutError, Sender}, Arc, RwLock}, thread, time::Duration,
};
use log::{debug, error, info, trace, warn};
use concat_string::concat_string;
use crate::{
    core_::constants::constants::RECV_TIMEOUT,
    conf::task_config::TaskConfig, 
    services::{
        safe_lock::SafeLock, services::Services, task::task_nodes::TaskNodes,
    },
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
    rx_recv: Vec<Receiver<Point>>,
    services: Arc<RwLock<Services>>,
    conf: TaskConfig,
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
            in_send: HashMap::from([(conf.rx.clone(), send)]),
            rx_recv: vec![recv],
            services,
            conf,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    ///
    fn subscriptions(&mut self, conf: &TaskConfig, services: &Arc<RwLock<Services>>) -> Option<(String, Vec<SubscriptionCriteria>)> {
        if conf.subscribe.is_empty() {
            None
        } else {
            debug!("{}.subscriptions | requesting points...", self.id);
            let mut self_points = self.conf.points();
            let mut points = services.rlock(&self.id).points(&self.id).then(
                |points| points,
                |err| {
                    error!("{}.subscriptions | Requesting Points error: {:?}", self.id, err);
                    vec![]
                },
            );
            points.append(&mut self_points);
            debug!("{}.subscriptions | rceived points: {:#?}", self.id, points.len());
            debug!(
                "{}.subscriptions | rceived points: {:#?}",
                self.id,
                points.iter().map(|p| concat_string!(p.id.to_string(), " | ", p.type_.to_string(), " | ", p.name)).collect::<Vec<String>>(),
            );
            debug!("{}.subscriptions | conf.subscribe: {:#?}", self.id, conf.subscribe);
            let subscriptions = conf.subscribe.with(&points);
            trace!("{}.subscriptions | subscriptions: {:#?}", self.id, subscriptions);
            if subscriptions.len() > 1 {
                panic!("{}.subscriptions | Error. Task does not supports multiple subscriptions for now: {:#?}.\n\tTry to use single subscription.", self.id, subscriptions);
            } else {
                let subscriptions_first = subscriptions.clone().into_iter().next();
                match subscriptions_first {
                    Some((service_name, Some(points))) => {
                        Some((service_name, points))
                    }
                    Some((_, None)) => {
                        warn!("{}.subscriptions | Error. Task subscription configuration error / empty in: {:#?}", self.id, subscriptions);
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
                self.rx_recv.pop().unwrap()
            }
        }
    }
}
//
//
impl Object for Task {
    fn id(&self) -> &str {
        &self.id
    }
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
unsafe impl Send for Task {}
unsafe impl Sync for Task {}
//
//
impl Service for Task {
    //
    //
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        match self.in_send.get(name) {
            Some(send) => send.clone(),
            None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        }
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        trace!("{}.run | Self tx_id: {}", self.id, PointTxId::from_str(&self.id));
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
            trace!("{}.run | taskNodes: {:#?}", self_id, task_nodes);
            'main: loop {
                trace!("{}.run | calculation step...", self_id);
                if cyclic {
                    cycle.start();
                    match rx_recv.recv_timeout(recv_timeout) {
                        Ok(point) => {
                            debug!("{}.run | point: {:?}", self_id, &point);
                            task_nodes.eval(point);
                            debug!("{}.run | calculation step - done ({:?})", self_id, cycle.elapsed());
                            cycle.wait();
                        }
                        Err(err) => {
                            match err {
                                RecvTimeoutError::Timeout => trace!("{}.run | Receive error: {:?}", self_id, err),
                                RecvTimeoutError::Disconnected => {
                                    error!("{}.run | Error receiving from queue: {:?}", self_id, err);
                                    break 'main;
                                }
                            }
                        }
                    };
                } else {
                    match rx_recv.recv() {
                        Ok(point) => {
                            debug!("{}.run | point: {:?}", self_id, &point);
                            task_nodes.eval(point);
                            debug!("{}.run | calculation step - done ({:?})", self_id, cycle.elapsed());
                        }
                        Err(err) => {
                            error!("{}.run | Error receiving from queue: {:?}", self_id, err);
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
                    error!("{}.run | Unsubscribe error: {:#?}", self_id, err);
                }
            }
            info!("{}.run | Exit", self_id);
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
    fn points(&self) -> Vec<PointConfig> {
        self.conf.points()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
        debug!("{}.run | Exit: {}", self.id, self.exit.load(Ordering::SeqCst));
    }
}
