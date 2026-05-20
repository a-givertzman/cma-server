use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{
    entity::{Name, Object, Point, PointConf, PointTxId}, Service, ServiceCycle, Services, SubscriptionCriteria
}, sync::{channel::{self, Receiver, RecvTimeoutError, Sender}, Handles, Owner}, thread_pool::Scheduler};
use std::{
    collections::HashMap, fmt::Debug, marker::PhantomData, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration
};
use concat_string::concat_string;
use crate::{
    domain::constants::constants::RECV_TIMEOUT, services::task::{task_conf::TaskConf, task_nodes::TaskNodes}, sync::SendWrapper,
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
    rx_recv: Owner<Receiver<Point>>,
    services: Arc<Services>,
    conf: TaskConf,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
//
impl Task {
    ///
    /// Creates new instance of [Task]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: TaskConf, services: Arc<Services>, scheduler: Scheduler) -> Task {
        let (send, recv) = channel::unbounded();
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Task {
            name: conf.name.clone(),
            in_send: HashMap::from([("in-send".to_owned(), send)]),
            rx_recv: Owner::new(recv),
            services,
            conf,
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    ///
    fn subscriptions_(&self, conf: &TaskConf, services: &Arc<Services>) -> Option<(String, Vec<SubscriptionCriteria>)> {
        if conf.subscribe.is_empty() {
            None
        } else {
            log::debug!("{}.subscriptions | requesting points...", self.dbg);
            let mut self_points = self.conf.points();
            let mut points = services.points(&self.dbg).then(
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
                log::error!("{}.subscriptions | Task does not supports multiple subscriptions for now: {:#?}.\n\tTry to use single subscription.", self.dbg, subscriptions);
                None
            } else {
                let subscriptions_first = subscriptions.clone().into_iter().next();
                match subscriptions_first {
                    Some((service_name, Some(points))) => {
                        Some((service_name, points))
                    }
                    Some((_, None)) => {
                        log::error!("{}.subscriptions | Task subscription configuration error / empty in: {:#?}", self.dbg, subscriptions);
                        None
                    }
                    None => {
                        log::error!("{}.subscriptions | Task subscription configuration error in: {:#?}", self.dbg, subscriptions);
                        None
                    }
                }
            }
        }
    }
    ///
    ///
    fn subscribe_(&self, subscriptions: &Option<(String, Vec<SubscriptionCriteria>)>, services: &Arc<Services>) -> Receiver<Point> {
        match subscriptions {
            Some((service_name, points)) => {
                let (_, rx_recv) = services.subscribe(
                    service_name,
                    &self.name.join(),
                    points,
                );
                rx_recv
            }
            None => self.rx_recv.take().unwrap(),
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
impl Service for Task {
    //
    //
    fn get_link(&self, name: &str) -> Sender<Point> {
        // match self.in_send.get(name) {
        match self.in_send.iter().next() {
            Some((_, send)) => send.clone(),
            None => {
                log::error!("{}.get_link | link '{}' - not found", self.dbg, name);
                panic!("{}.get_link | Error", self.dbg);
            }
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
        let task_nodes = {
            let mut task_nodes = TaskNodes::new(&dbg);
            task_nodes.build_nodes(&self_name, conf, services.clone())
                .map_err(|err| Error::new(&dbg, "run").pass(err))?;
            SendWrapper::wrap(task_nodes)
        };
        let handle = self.scheduler.spawn({
            let dbg = dbg.clone();
            move || {
            let mut cycle = ServiceCycle::new(&dbg, cycle_interval);
            let mut task_nodes = task_nodes.extract();
            log::trace!("{dbg}.run | task_nodes: {:#?}", task_nodes);
            'main: while !exit.load(Ordering::SeqCst) {
                log::trace!("{dbg}.run | Calculation step...");
                if cyclic {
                    cycle.start();
                    match rx_recv.recv_timeout(recv_timeout) {
                        Ok(point) => {
                            // log::debug!("{dbg}.run | point: {:?}", &point);
                            log::debug!("{dbg}.run | Event '{}': {:?}  {:?}  {:?}", point.name(), point.value(), point.status(), point.cot());
                            task_nodes.eval(point);
                            log::debug!("{dbg}.run | Calculation step - done ({:?})", cycle.elapsed());
                            cycle.wait();
                        }
                        Err(err) => match err {
                            RecvTimeoutError::Timeout => {},
                            _ => {
                                log::trace!("{dbg}.run | Error receiving from queue: {:?}", err);
                                break 'main;
                            }
                        }
                    };
                } else {
                    match rx_recv.recv() {
                        Ok(point) => {
                            log::debug!("{dbg}.run | point: {:?}", &point);
                            task_nodes.eval(point);
                            log::debug!("{dbg}.run | calculation step - done ({:?})", cycle.elapsed());
                        }
                        Err(err) => {
                            log::error!("{dbg}.run | Error receiving from queue: {:?}", err);
                            break 'main;
                        }
                    };
                }
            }
            if let Some((service_name, points)) = subscriptions {
                if let Err(err) = services.unsubscribe(&service_name,&self_name.join(), &points) {
                    log::error!("{dbg}.run | Unsubscribe error: {:#?}", err);
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        }});
        match handle {
            Ok(handle) => {
                log::info!("{dbg}.run | Starting - ok");
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => {
                let err = Error::new(&dbg, "run").pass_with("Start failed", err.to_string());
                log::warn!("{}", err);
                Err(err)
            }
        }
    }
    //
    //
    fn points(&self) -> Vec<PointConf> {
        self.conf.points()
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
        log::debug!("{}.run | Exit: {}", self.dbg, self.exit.load(Ordering::SeqCst));
    }
}
