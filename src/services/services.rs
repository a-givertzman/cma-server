use sal_sync::services::{
    entity::{name::Name, object::Object, point::{point::Point, point_config::PointConfig}}, 
    future::future::{Future, Sink}, retain::{retain_conf::RetainConf, retain_point_id::RetainPointId}, 
    service::{link_name::LinkName, service::Service, service_cycle::ServiceCycle, service_handles::ServiceHandles},
    subscription::subscription_criteria::SubscriptionCriteria
};
use std::{collections::HashMap, fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{Receiver, Sender}, Arc, RwLock}, thread, time::Duration};
use log::{debug, error, info, warn};
use concat_string::concat_string;
use crate::{core_::state::change_notify::ChangeNotify, services::safe_lock::SafeLock};
///
/// States of the Services behavior for logging
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NotifyState {
    Start,
    Info,
    Warn,
    RetainPointNotConfiguredWarn,
    Error,
    PointsRequestsIsEmpty,
    PointsRequestsAccessError,
}
///
/// Holds a map of the all services in app by there names
pub struct Services {
    id: String,
    name: Name,
    map: Arc<RwLock<HashMap<String, Arc<RwLock<dyn Service>>>>>,
    retain_conf: RetainConf,
    retain_point_id: Option<Arc<RwLock<RetainPointId>>>,
    points_requested: Arc<AtomicUsize>,
    points_request: Arc<RwLock<Vec< (String, Sink<Vec<PointConfig>>) >>>,
    exit: Arc<AtomicBool>,
}
//
//
impl Object for Services {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Services {
    pub const API_CLIENT: &'static str = "ApiClient";
    pub const MULTI_QUEUE: &'static str = "MultiQueue";
    pub const PROFINET_CLIENT: &'static str = "ProfinetClient";
    pub const TASK: &'static str = "Task";
    pub const TCP_CLIENT: &'static str = "TcpClient";
    pub const TCP_SERVER: &'static str = "TcpServer";
    pub const PRODUCER_SERVICE: &'static str = "ProducerService";
    pub const CACHE_SERVICE: &'static str = "CacheService";
    pub const SLMP_CLIENT: &'static str = "SlmpClient";
    ///
    /// Creates new instance of the Services
    pub fn new(parent: impl Into<String>, conf: RetainConf) -> Self {
        let name = Name::new(parent, "Services");
        let self_id = name.join();
        Self {
            id: self_id.clone(),
            name,
            map: Arc::new(RwLock::new(HashMap::new())),
            retain_point_id: match &conf.point {
                Some(_) => Some(Arc::new(RwLock::new(RetainPointId::new(&self_id, conf.clone())))),
                None => None,
            },
            retain_conf: conf,
            points_requested: Arc::new(AtomicUsize::new(0)),
            points_request: Arc::new(RwLock::new(vec![])),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Prepairing retained points id's
    fn prepare_point_ids(self_id: &str, notify: &mut ChangeNotify<NotifyState, String>, retain_point_id: &Option<Arc<RwLock<RetainPointId>>>, services: &Arc<RwLock<HashMap<String, Arc<RwLock<dyn Service>>>>>) {
        match retain_point_id {
            Some(retain_point_id) => {
                info!("{}.prepare_point_ids | Preparing Points id's...", self_id);
                match services.read() {
                    Ok(services) => {
                        for (service_id, service) in services.iter() {
                            let service_points = service.rlock(self_id).points();
                            match retain_point_id.write() {
                                Ok(mut retain_point_id) => {
                                    retain_point_id.insert(&service_id, service_points);
                                }
                                Err(err) => error!("{}.prepare_point_ids | Points id's write access error: {:#?}", self_id, err),
                            }
                        };
                        info!("{}.prepare_point_ids | Points is chashed: {}", self_id, retain_point_id.read().unwrap().is_cached());
                        let points = retain_point_id.write().unwrap()
                            .points()
                            .iter()
                            .map(|(owner, p)| {
                                p.iter().map(|p| {
                                    concat_string!(owner, " | ", p.id.to_string(), " | ", p.type_.to_string(), " | ", p.name, "\n")
                                }).collect()
                            }).collect::<Vec<String>>();
                        info!("{}.prepare_point_ids | Points: {:#?}", self_id, points);
                        info!("{}.prepare_point_ids | Preparing Points id's - ok", self_id);
                    }
                    Err(err) => error!("{}.prepare_point_ids | Services read access error: {:#?}", self_id, err),
                }
            }
            None => notify.add(NotifyState::RetainPointNotConfiguredWarn, format!("{}.run | Retain->Point - not configured", self_id)),
        }
    }
    ///
    /// Main loop of the Services
    pub fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let points_requested = self.points_requested.clone();
        let points_request = self.points_request.clone();
        let retain_point_id = self.retain_point_id.clone();
        let services = self.map.clone();
        let exit = self.exit.clone();
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            let mut notify = ChangeNotify::new(
                &self_id,
                NotifyState::Start,
                vec![
                    (NotifyState::Start,  Box::new(|message| log::info!("{}", message))),
                    (NotifyState::Info,   Box::new(|message| log::info!("{}", message))),
                    (NotifyState::Warn,   Box::new(|message| log::warn!("{}", message))),
                    (NotifyState::RetainPointNotConfiguredWarn,   Box::new(|message| log::warn!("{}", message))),
                    (NotifyState::Error,  Box::new(|message| log::error!("{}", message))),
                    (NotifyState::PointsRequestsAccessError,  Box::new(|message| log::error!("{}", message))),
                    (NotifyState::PointsRequestsIsEmpty,  Box::new(|message| log::error!("{}", message))),
                ],
            );
            Self::prepare_point_ids(&self_id, &mut notify, &retain_point_id, &services);
            let mut cycle = ServiceCycle::new(&self_id, Duration::from_millis(10));
            loop {
                cycle.start();
                if points_requested.load(Ordering::SeqCst) > 0 {
                    match points_request.write() {
                        Ok(mut requests) => {
                            match requests.pop() {
                                Some((requester_name, sink)) => {
                                    debug!("{}.run | Points requested from: '{}'", self_id, requester_name);
                                    points_requested.fetch_sub(1, Ordering::SeqCst);
                                    match &retain_point_id {
                                        Some(retain_point_id) => match retain_point_id.write() {
                                            Ok(mut retain_point_id) => {
                                                let points = retain_point_id.points()
                                                .into_iter().filter_map(|(owner, points)| {
                                                    if *owner != requester_name {
                                                        Some(points)
                                                    } else {
                                                        None
                                                    }
                                                }).flatten().collect();
                                                sink.add(points);
                                                debug!("{}.run | Points requested from: '{}' - Ok", self_id, requester_name);
                                            }
                                            Err(err) => {
                                                error!("{}.run | Points id's write access error, requester: '{}', error: {:#?}", self_id, requester_name, err);
                                                sink.add(vec![]);
                                            }
                                        },
                                        None => {
                                            notify.add(NotifyState::RetainPointNotConfiguredWarn, format!("{}.run | Retain->Point - not configured", self_id));
                                            sink.add(vec![]);
                                        }
                                    }
                                }
                                None => notify.add(NotifyState::PointsRequestsIsEmpty, format!("{}.run | Points requests is empty", self_id)),
                            }
                        }
                        Err(err) => {
                            notify.add(NotifyState::PointsRequestsAccessError, format!("{}.run | Points requests access error: {:#?}", self_id, err));
                        }
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
                cycle.wait();
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
            info!("{}.run | Exit", self_id);
        });
        thread::sleep(Duration::from_millis(50));
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
    ///
    /// Returns all holding services in the map<service id, service reference>
    pub fn all(&self) -> HashMap<String, Arc<RwLock<dyn Service>>> {
        let mut map = HashMap::new();
        match self.map.read() {
            Ok(services) => services.clone_into(&mut map),
            Err(err) => error!("{}.all | Services read access error: {:#?}", self.id, err),
        }
        map
    }
    ///
    /// Inserts a new service into the collection
    pub fn insert(&mut self, service: Arc<RwLock<dyn Service>>) {
        let name = service.rlock(&self.id).name().join();
        match self.map.write() {
            Ok(mut services) => {
                if services.contains_key(&name) {
                    panic!("{}.insert | Duplicated service name '{:?}'", self.id, name);
                }
                services.insert(name, service);
            }
            Err(err) => error!("{}.insert | Services write access error: {:#?}", self.id, err),
        }
    }
    ///
    /// Returns Service
    pub fn get(&self, name: &str) -> Option<Arc<RwLock<dyn Service>>> {
        match self.map.read() {
            Ok(services) => {
                match services.get(name) {
                    Some(srvc) => Some(srvc.clone()),
                    None => {
                        warn!("{}.get | service '{:?}' - not found", self.id, name);
                        None
                    },
                }
            }
            Err(err) => {
                error!("{}.get | Services read access error: {:#?}", self.id, err);
                None
            }
        }
    }
    ///
    /// Returns copy of the Sender - service's incoming queue by service link name (Service.link)
    pub fn get_link(&self, name: &LinkName) -> Result<Sender<Point>, String> {
        match name.split() {
            Ok((service, queue)) => {
                match self.get(&service) {
                    Some(srvc) => Ok(srvc.slock(&self.id).get_link(&queue)),
                    None => Err(format!("{}.get_link | service '{:?}' - not found", self.id, name)),
                }
            }
            Err(err) => Err(err),
        }
    }
    ///
    /// Returns Receiver
    /// - service - the name of the service to subscribe on
    pub fn subscribe(&mut self, service: &str, receiver_name: &str, points: &[SubscriptionCriteria]) -> (Sender<Point>, Receiver<Point>) {
        match self.get(service) {
            Some(srvc) => {
                let r = srvc.slock(&self.id).subscribe(receiver_name, points);
                r
            }
            None => panic!("{}.subscribe | service '{:?}' - not found", self.id, service),
        }
    }
    ///
    /// Returns ok if subscription extended sucessfully
    /// - service - the name of the service to extend subscribtion on
    pub fn extend_subscription(&mut self, service: &str, receiver_name: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        // panic!("{}.extend_subscription | Not implemented yet", self.id);
        match self.get(service) {
            Some(srvc) => {
                let r = srvc.slock(&self.id).extend_subscription(receiver_name, points);
                r
            }
            None => panic!("{}.extend_suscription | service '{:?}' - not found", self.id, service),
        }
    }
    ///
    /// Returns ok if subscription removed sucessfully
    /// - service - the name of the service to unsubscribe on
    pub fn unsubscribe(&mut self, service: &str, receiver_name: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        match self.get(service) {
            Some(srvc) => {
                let r = srvc.slock(&self.id).unsubscribe(receiver_name, points);
                r
            }
            None => panic!("{}.unsubscribe | service '{:?}' - not found", self.id, service),
        }
    }
    ///
    /// Returns list of point configurations over the all services
    ///  - requester_name - Service name !!!
    pub fn points(&self, requester_name: impl Into<String>) -> Future<Vec<PointConfig>> {
        let (future, sink) = Future::new();
        match self.points_request.write() {
            Ok(mut points_request) => {
                points_request.push((requester_name.into(), sink));
                self.points_requested.fetch_add(1, Ordering::SeqCst);
            }
            Err(err) => error!("{}.get | Services read access error: {:#?}", self.id, err),
        }
        future
        // let points = if !self.point_ids.read().unwrap().is_cached() {
        //     debug!("{}.points | requester_id: '{}'", self.id, requester_name);
        //     let mut points = vec![];
        //     for (service_id, service) in &self.map {
        //         if service_id != requester_name {
        //             let mut service_points = service.rlock(&self.id).points();
        //             points.append(&mut service_points);
        //         }
        //     };
        //     self.point_ids.write().unwrap().points(points)
        // } else {
        //     self.point_ids.write().unwrap().points(vec![])
        // };
        // debug!("{}.points | points: '{:#?}'", self.id, points.len());
        // trace!("{}.points | points: '{:#?}'", self.id, points);
        // points
    }
    ///
    /// Sends the General Interogation request to all services
    pub fn gi(&self, _service: &str, _points: &[SubscriptionCriteria]) -> Receiver<Point> {
        panic!("{}.gi | Not implemented yet", self.id);
    }
    ///
    /// Returns Retain configuration
    pub fn retain(&self) -> RetainConf {
        self.retain_conf.clone()
    }
    ///
    /// 
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
//
// 
impl Debug for Services {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Services")
            .field("id", &self.id)
            .finish()
    }
}
