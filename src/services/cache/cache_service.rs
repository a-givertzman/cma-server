//!
//! Service storing all received events in the memory,
//! - Subscribe on points by configured criteria
//! - Storing all received events on the disk if 'retain' option is true
//! - Storing received points into the HasMap by name as key
//! - Cyclically delyed stores accumulated changes to the disk if 'retain' option is true
//! Basic configuration parameters:
//! ```yaml
//! service CacheService Cache:
//!     retain: true    # true / false - enables storing cache on the disk
//!     suscribe:
//!         /App/MultiQueue: []
//! ```
use std::{
    env, fmt::Debug, fs, hash::BuildHasherDefault, io::Write, path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, RecvTimeoutError}, Arc},
    thread::{self, JoinHandle},
};
use chrono::Utc;
use coco::Stack;
use concat_string::concat_string;
use dashmap::DashMap;
use hashers::fx_hash::FxHasher;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    collections::FxIndexMap, services::{
        entity::{Cot, Name, Object, Point, PointConfig, PointConfigType, PointHlr, PointTxId, Status},
        Service,
        Services, SubscriptionCriteria, types::Bool,
    }
};
use serde::Serialize;
use serde_json::json;
use crate::{
    conf::cache_service_config::CacheServiceConfig,
    core_::{constants::constants::RECV_TIMEOUT, FxDashMap},
    services::cache::delay_store::DelyStore
};
///
/// CacheService service
/// - Subscribe on points by configured criteria
/// - Storing all received events on the disk if 'retain' option is true
pub struct CacheService {
    dbg: Dbg,
    name: Name,
    conf: CacheServiceConfig,
    services: Arc<Services>,
    cache: FxDashMap<String, Point>,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
}
//
//
impl CacheService {
    ///
    /// Creates new instance of the CacheService
    pub fn new(conf: CacheServiceConfig, services: Arc<Services>) -> Self {
        Self {
            dbg: Dbg::new(conf.name.parent(), conf.name.me()),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            cache: DashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns vector of the SubscriptionCriteria by config and list of configured Point's
    fn subscriptions(&self, conf: &CacheServiceConfig, points: &[PointConfig]) -> (String, Vec<SubscriptionCriteria>) {
        if conf.subscribe.is_empty() {
            panic!("{}.subscribe | Error. Subscription can`t be empty: {:#?}", self.dbg, conf.subscribe);
        } else {
            log::debug!("{}.subscribe | conf.subscribe: {:#?}", self.dbg, conf.subscribe);
            let subscriptions = conf.subscribe.with(points);
            log::trace!("{}.subscribe | subscriptions: {:#?}", self.dbg, subscriptions);
            if subscriptions.len() > 1 {
                panic!("{}.run | Error. Task does not supports multiple subscriptions for now: {:#?}.\n\tTry to use single subscription.", self.dbg, subscriptions);
            } else {
                match subscriptions.clone().into_iter().next() {
                    Some((service_name, Some(points))) => {
                        (service_name, points)
                    }
                    Some((_, None)) => panic!("{}.run | Error. Subscription configuration error in: {:#?}", self.dbg, subscriptions),
                    None => panic!("{}.run | Error. Subscription configuration error in: {:#?}", self.dbg, subscriptions),
                }
            }
        }
    }
    ///
    /// Creates directiry (all necessary folders in the 'path' if not exists)
    ///  - path is relative, will be joined with current working dir
    fn create_dir(dbg: &Dbg, path: &str) -> Result<PathBuf, String> {
        let current_dir = env::current_dir().unwrap();
        let path = current_dir.join(path);
        match path.exists() {
            true => Ok(path),
            false => {
                match fs::create_dir_all(&path) {
                    Ok(_) => Ok(path),
                    Err(err) => {
                        let message = format!("{}.create_dir | Error create path: '{:?}'\n\terror: {:?}", dbg, path, err);
                        log::error!("{}", message);
                        Err(message)
                    }
                }
            }
        }
    }
    ///
    /// Loads retained on the disk points to the self cache
    fn load(dbg: &Dbg, name: &Name, cache: &FxDashMap<String, Point>) {
        let path = Name::new("assets/cache/", name.join()).join().trim_start_matches('/').to_owned();
        let path = Path::new(&path).join("cache.json");
        match fs::OpenOptions::new().read(true).open(&path) {
            Ok(f) => {
                match serde_json::from_reader::<_, Vec<Point>>(f) {
                    Ok(v) => {
                        for point in v {
                            cache.insert(point.dest(), point);
                        }
                        log::info!("{}.load | Retained cache loaded from: '{:?}'", dbg, path);
                    }
                    Err(err) => {
                        log::error!("{}.load | Deserialize error: '{:?}'\n\tin file: {:?}", dbg, err, path);
                    }
                };
            }
            Err(err) => {
                log::error!("{}.load | Error open file: '{:?}'\n\terror: {:?}", dbg, path, err);
            }
        }
    }
    ///
    /// Writes array of the points to the json file:
    /// ```json
    /// [
    ///     {"type": "Bool","value": 1,"name": "/App/path/Point.name1","status": 2,"cot": "Inf","timestamp": "2024-04-08T08:52:32.656576549+00:00"},
    ///     {...,
    ///     ...
    /// ]
    /// ```
    fn write<S: Serialize>(dbg: &Dbg, name: &Name, points: Vec<S>) -> Result<(), String> {
        match Self::create_dir(dbg, Name::new("assets/cache/", name.join()).join().trim_start_matches('/')) {
            Ok(path) => {
                let path = path.join("cache.json");
                let mut message = String::new();
                let mut cache = String::new();
                cache.push('[');
                let content: String = points.into_iter().fold(String::new(), |mut points, point| {
                    points.push_str(concat_string!("\n", json!(point).to_string(), ",").as_str());
                    points
                }).trim_end_matches(',').to_owned();
                cache.push_str(content.as_str());
                cache.push_str("\n]");
                match fs::OpenOptions::new().truncate(true) .create(true).write(true).open(&path) {
                    Ok(mut f) => {
                        match f.write_all(cache.as_bytes()) {
                            Ok(_) => {
                                log::debug!("{}.write | Cache stored in: {:?}", dbg, path);
                            }
                            Err(err) => {
                                message = format!("{}.write | Error writing to file: '{:?}'\n\terror: {:?}", dbg, path, err);
                                log::error!("{}", message);
                            }
                        };
                        if message.is_empty() {Ok(())} else {Err(message)}
                    }
                    Err(err) => {
                        let message = format!("{}.write | Error open file: '{:?}'\n\terror: {:?}", dbg, path, err);
                        log::error!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(err) => {
                log::error!("{:#?}", err);
                Err(err)
            }
        }
    }
    ///
    /// Stores self.cache on the disk
    fn store(dbg: &Dbg, name: &Name, points: FxIndexMap<String, Point>, status: Status) -> Result<(), String> {
        let points: Vec<Point> = points.into_iter().map(|(_dest, point)| {
            match point.clone() {
                Point::Bool(mut point) => {
                    point.status = status;
                    Point::Bool(point)
                }
                Point::Int(mut point) => {
                    point.status = status;
                    Point::Int(point)
                }
                Point::Real(mut point) => {
                    point.status = status;
                    Point::Real(point)
                }
                Point::Double(mut point) => {
                    point.status = status;
                    Point::Double(point)
                }
                Point::String(mut point) => {
                    point.status = status;
                    Point::String(point)
                }
            }
        }).collect();
        Self::write(dbg, name, points)
    }
    ///
    /// Fills self cache with initial values for all configured points
    pub fn initial(
        dbg: &Dbg,
        tx_id: usize, 
        cache: &FxDashMap<String, Point>,
        points: &[PointConfig],
        initial_status: Status,
    ) {
        let timestamp = Utc::now();
        log::trace!("{}.initial | Initial cashe generated at {:?}", dbg, timestamp);
        for point_config in points {
            let point = match point_config.type_ {
                PointConfigType::Bool => Point::Bool(PointHlr::new(
                    tx_id,
                    &point_config.name,
                    Bool(false),
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointConfigType::Int => Point::Int(PointHlr::new(
                    tx_id,
                    &point_config.name,
                    0,
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointConfigType::Real => Point::Real(PointHlr::new(
                    tx_id,
                    &point_config.name,
                    0.0,
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointConfigType::Double => Point::Double(PointHlr::new(
                    tx_id,
                    &point_config.name,
                    0.0,
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointConfigType::String => Point::String(PointHlr::new(
                    tx_id,
                    &point_config.name,
                    String::new(),
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
                PointConfigType::Json => Point::String(PointHlr::new(
                    tx_id,
                    &point_config.name,
                    String::new(),
                    initial_status,
                    Cot::Inf,
                    timestamp,
                )),
            };
            cache.insert(SubscriptionCriteria::dest(&Cot::Inf, &point_config.name), point);
        }
    }
    ///
    /// Sorting cached values by key into IndexMap
    fn sorted(cache: &FxDashMap<String, Point>) -> FxIndexMap<String, Point> {
        let vec: Vec<(String, Point)> = cache.iter().map(|r| (r.key().to_string(), r.value().clone())).collect();
        let mut sorted = FxIndexMap::from_iter(vec);
        sorted.sort_by(|k1, _, k2, _| k1.cmp(&k2));
        sorted
    }
}
//
//
impl Object for CacheService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for CacheService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CacheService")
            .field("dbg", &self.dbg)
            .field("name", &self.name)
            .finish()
    }
}
//
//
impl Service for CacheService {
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let tx_id = PointTxId::from_str(&self_name.join());
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let cache = self.cache.clone();
        let point_configs = services.points(&self_name.join())
            .then(
                |points| points,
            |err| {
                log::error!("{}.run | Requesting Points error: {:?}", dbg, err);
                vec![]
            }
        );
        let (service_name, points) = self.subscriptions(&conf, &point_configs);
        log::debug!("{}.run | points: {:#?}", dbg, points.len());
        log::trace!("{}.run | points: {:#?}", dbg, points);
        let (_, rx_recv) = services.subscribe(
            &service_name,
            &self.name.join(),
            &points,
        );
        let mut dely_store = DelyStore::new(conf.retain_delay);
        log::info!("{}.run | Preparing thread...", dbg);
        let handle = thread::Builder::new().name(format!("{}.run", dbg)).spawn(move || {
            let initial_status = Status::Invalid;
            let retain_status = Status::Invalid;
            Self::initial(&dbg, tx_id, &cache, &point_configs, initial_status);
            Self::load(&dbg, &self_name, &cache);
            'main: loop {
                match rx_recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        cache.insert(point.dest(), point);
                        if dely_store.exceeded() && Self::store(&dbg, &self_name, Self::sorted(&cache), retain_status).is_ok() {
                            dely_store.set_stored();
                        }
                    }
                    Err(err) => {
                        match err {
                            RecvTimeoutError::Timeout => {
                                log::trace!("{}.run | Receive error: {:?}", dbg, err);
                            }
                            RecvTimeoutError::Disconnected => {
                                log::error!("{}.run | Error receiving from queue: {:?}", dbg, err);
                                break 'main;
                            }
                        }
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    if !dely_store.stored() {
                        _ = Self::store(&dbg, &self_name, Self::sorted(&cache), retain_status);
                    }
                    break;
                }
            }
            if let Err(err) = services.unsubscribe(&service_name, &self_name.join(), &points) {
                log::error!("{}.run | Unsubscribe error: {:#?}", dbg, err);
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
    fn gi(&self, receiver_name: &str, points: &[SubscriptionCriteria]) -> Receiver<Point> {
        let self_id = self.dbg.clone();
        log::info!("{}.gi | Gi requested from: {}", self_id, receiver_name);
        let (send, recv) = mpsc::channel();
        let cache = Arc::new(self.cache.clone());
        let points = points.to_owned();
        thread::spawn(move || {
            if points.is_empty() {
                for point in cache.iter().map(|r| r.value().clone()) {
                    match send.send(point.clone()) {
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("{}.gi | Send error: {:#?}", self_id, err);
                        }
                    }
                }
            } else {
                for point in points {
                    match cache.get(&point.destination()) {
                        Some(point) => {
                            match send.send(point.clone()) {
                                Ok(_) => {}
                                Err(err) => {
                                    log::error!("{}.gi | Send error: {:#?}", self_id, err);
                                }
                            }
                        }
                        None => {
                            log::error!("{}.gi | Error, requested point '{}' - not found", self_id, point.destination());
                        }
                    }
                }
            }
        });
        // self.handle.push(handle);
        recv
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
    }
}
