use std::{collections::HashMap, fs, hash::BuildHasherDefault, path::Path, sync::{mpsc::{Receiver, Sender}, Arc, Mutex}};
use hashers::fx_hash::FxHasher;
use log::{debug, error, trace};
use crate::{
    core_::point::point_type::PointType, 
    conf::point_config::point_config::PointConfig, 
    services::{
        multi_queue::subscription_criteria::SubscriptionCriteria, queue_name::QueueName, service::service::Service
    }
};

///
/// Holds a map of the all services in app by there names
pub struct Services {
    id: String,
    map: HashMap<String, Arc<Mutex<dyn Service + Send>>>,
}
///
/// 
impl Services {
    pub const API_CLIENT: &'static str = "ApiClient";
    pub const MULTI_QUEUE: &'static str = "MultiQueue";
    pub const PROFINET_CLIENT: &'static str = "ProfinetClient";
    pub const TASK: &'static str = "Task";
    pub const TCP_CLIENT: &'static str = "TcpClient";
    pub const TCP_SERVER: &'static str = "TcpServer";
    ///
    /// Creates new instance of the Services
    pub fn new(parent: impl Into<String>) -> Self {
        Self {
            id: parent.into(),
            map: HashMap::new(),
        }
    }
    ///
    /// 
    pub fn all(&self) -> HashMap<String, Arc<Mutex<dyn Service + Send>>> {
        self.map.clone()
    }
    ///
    /// 
    pub fn insert(&mut self, id:&str, service: Arc<Mutex<dyn Service + Send>>) {
        if self.map.contains_key(id) {
            panic!("{}.insert | Duplicated service name '{:?}'", self.id, id);
        }
        self.map.insert(id.to_string(), service);
    }
    ///
    /// Returns Service
    pub fn get(&self, name: &str) -> Arc<Mutex<dyn Service>> {
        match self.map.get(name) {
            Some(srvc) => srvc.clone(),
            None => panic!("{}.get | service '{:?}' - not found", self.id, name),
        }
    }
    ///
    /// Returns copy of the Sender - service's incoming queue
    pub fn get_link(&self, name: &str) -> Sender<PointType> {
        let name = QueueName::new(name);
        match self.map.get(name.service()) {
            Some(srvc) => srvc.lock().unwrap().get_link(name.queue()),
            None => panic!("{}.get | service '{:?}' - not found", self.id, name),
        }
    }
    ///
    /// Returns Receiver
    /// - service - the name of the service to subscribe on
    pub fn subscribe(&mut self, service: &str, receiver_id: &str, points: &[SubscriptionCriteria]) -> (Sender<PointType>, Receiver<PointType>) {
        match self.map.get(service) {
            Some(srvc) => {
                debug!("{}.subscribe | Lock service '{:?}'...", self.id, service);
                let r = srvc.lock().unwrap().subscribe(receiver_id, points);
                debug!("{}.subscribe | Lock service '{:?}' - ok", self.id, service);
                r
            },
            None => panic!("{}.get | service '{:?}' - not found", self.id, service),
        }
    }
    ///
    /// Returns ok if subscription extended sucessfully
    /// - service - the name of the service to extend subscribtion on
    pub fn extend_subscription(&mut self, service: &str, receiver_id: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        // panic!("{}.extend_subscription | Not implemented yet", self.id);
        match self.map.get(service) {
            Some(srvc) => {
                debug!("{}.extend_subscription | Lock service '{:?}'...", self.id, service);
                let r = srvc.lock().unwrap().extend_subscription(receiver_id, points);
                debug!("{}.extend_subscription | Lock service '{:?}' - ok", self.id, service);
                r
            },
            None => panic!("{}.get | service '{:?}' - not found", self.id, service),
        }
    }
    ///
    /// Returns ok if subscription removed sucessfully
    /// - service - the name of the service to unsubscribe on
    fn unsubscribe(&mut self, service: &str, receiver_id: &str, points: &[SubscriptionCriteria]) -> Result<(), String> {
        match self.map.get(service) {
            Some(srvc) => {
                debug!("{}.unsubscribe | Lock service '{:?}'...", self.id, service);
                let r = srvc.lock().unwrap().unsubscribe(receiver_id, points);
                debug!("{}.unsubscribe | Lock service '{:?}' - ok", self.id, service);
                r
            },
            None => panic!("{}.get | service '{:?}' - not found", self.id, service),
        }
    }
    ///
    /// Returns list of point configurations over the all services
    pub fn points(&self) -> Vec<PointConfig> {
        let mut points = vec![];
        for service in self.map.values() {
        debug!("{}.points | service: '{:?}'", self.id, service.lock().unwrap().id());
        let mut service_points = service.lock().unwrap().points();
            points.append(&mut service_points);
        };
        trace!("{}.points | points: '{:#?}'", self.id, points);
        points
    }
}

///
/// Stores unique Point ID in the json file
struct RetainPointId {
    id: String,
    services: Arc<Mutex<Services>>,
    path: String,
    cache: HashMap<usize, PointConfig, BuildHasherDefault<FxHasher>>,
}
///
/// 
impl RetainPointId {
    ///
    /// Creates new instance of the RetainPointId
    ///  - parent - the name of the parent object
    ///  - services - Services thread safe mutable reference
    ///  - path - path to the file, where point id's will be stored
    pub fn new(parent: &str, services: Arc<Mutex<Services>>, path: &str) -> Self {
        Self {
            id: format!("{}/RetainPointId", parent),
            services,
            path: path.to_owned(),
            cache: HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
        }
    }
    ///
    /// 
    pub fn points(&self) -> Vec<PointConfig> {
        let json_value = self.read(self.path.clone());
        match json_value {
            Ok(retained) => {
                self.points().into_iter().map(|point| {
                    let id = match retained.get(&point.name) {
                        Some(id) => *id,
                        None => {
                            retained.values().max().unwrap_or(&0).to_owned()
                        },
                    };
                    PointConfig {
                        id,
                        name: point.name,
                        _type: point._type,
                        history: point.history,
                        alarm: point.alarm,
                        address: point.address,
                        filters: point.filters,
                        comment: point.comment,
                    }
                }).collect()
            },
            Err(err) => panic!("{}.points | Error reading retain file {}: \n\t{:#?}", self.id, self.path, err),
        }
    }
    ///
    /// Reads file contains json map:
    /// ```json
    /// {
    ///     "/path/Point.name1": 0,
    ///     "/path/Point.name2": 1,
    ///     ...
    /// }
    /// ```
    fn read<P: AsRef<Path> + std::fmt::Display>(&self, path: P) -> Result<HashMap<String, usize, BuildHasherDefault<FxHasher>>, String> {
        match fs::read_to_string(path) {
            Ok(json_string) => {
                match serde_json::from_str(&json_string) {
                    Ok(config) => {
                        let cinfig: serde_json::Map<String, serde_json::Value> = config;
                        let result = config.into_iter().filter_map(|(key, value)| {
                            match value.as_u64() {
                                Some(value) => {
                                    Some((key, value as usize))
                                },
                                None => {
                                    error!("{}.read | Error parsing usize value in pair: {}: {:?}", self.id, key, value);
                                    None
                                },
                            }
                        }).collect();
                        Ok(result)
                    },
                    Err(err) => {
                        Err(format!("{}.read | Error in config: {:?}\n\terror: {:?}", self.id, json_string, err))
                    },
                }
            },
            Err(err) => {
                Err(format!("{}.read | File {} reading error: {:?}", self.id, path, err))
            },
        }
    }
}