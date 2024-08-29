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
    env, fmt::Debug, fs, hash::{BuildHasher, BuildHasherDefault}, io::Write, path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, RecvTimeoutError}, Arc, RwLock},
    thread,
};
use chrono::Utc;
use concat_string::concat_string;
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use log::{debug, error, info, trace, warn};
use sal_sync::{collections::map::IndexMapFxHasher, services::{entity::{
    cot::Cot, name::Name, object::Object,
    point::{point::Point, point_config::PointConfig, point_config_type::PointConfigType, point_hlr::PointHlr, point_tx_id::PointTxId},
    status::status::Status,
}, service::{service::Service, service_handles::ServiceHandles}, subscription::subscription_criteria::SubscriptionCriteria, types::bool::Bool}};
use serde::Serialize;
use serde_json::json;
use crate::{
    conf::cache_service_config::CacheServiceConfig,
    core_::constants::constants::RECV_TIMEOUT,
    services::{
        cache::delay_store::DelyStore,
        safe_lock::SafeLock,
        services::Services,
    }
};
///
/// CacheService service
/// - Subscribe on points by configured criteria
/// - Storing all received events on the disk if 'retain' option is true
pub struct CacheService {
    id: String,
    name: Name,
    conf: CacheServiceConfig,
    services: Arc<RwLock<Services>>,
    cache: Arc<RwLock<IndexMap<String, Point, BuildHasherDefault<FxHasher>>>>,
    exit: Arc<AtomicBool>,
}
//
//
impl CacheService {
    ///
    /// Creates new instance of the CacheService
    pub fn new(conf: CacheServiceConfig, services: Arc<RwLock<Services>>) -> Self {
        Self {
            id: conf.name.join(),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            cache: Arc::new(RwLock::new(IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default()))),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns vector of the SubscriptionCriteria by config and list of configured Point's
    fn subscriptions(&mut self, conf: &CacheServiceConfig, points: &[PointConfig]) -> (String, Vec<SubscriptionCriteria>) {
        if conf.subscribe.is_empty() {
            panic!("{}.subscribe | Error. Subscription can`t be empty: {:#?}", self.id, conf.subscribe);
        } else {
            debug!("{}.subscribe | conf.subscribe: {:#?}", self.id, conf.subscribe);
            let subscriptions = conf.subscribe.with(points);
            trace!("{}.subscribe | subscriptions: {:#?}", self.id, subscriptions);
            if subscriptions.len() > 1 {
                panic!("{}.run | Error. Task does not supports multiple subscriptions for now: {:#?}.\n\tTry to use single subscription.", self.id, subscriptions);
            } else {
                match subscriptions.clone().into_iter().next() {
                    Some((service_name, Some(points))) => {
                        (service_name, points)
                    }
                    Some((_, None)) => panic!("{}.run | Error. Subscription configuration error in: {:#?}", self.id, subscriptions),
                    None => panic!("{}.run | Error. Subscription configuration error in: {:#?}", self.id, subscriptions),
                }
            }
        }
    }
    ///
    /// Creates directiry (all necessary folders in the 'path' if not exists)
    ///  - path is relative, will be joined with current working dir
    fn create_dir(self_id: &str, path: &str) -> Result<PathBuf, String> {
        let current_dir = env::current_dir().unwrap();
        let path = current_dir.join(path);
        match path.exists() {
            true => Ok(path),
            false => {
                match fs::create_dir_all(&path) {
                    Ok(_) => Ok(path),
                    Err(err) => {
                        let message = format!("{}.create_dir | Error create path: '{:?}'\n\terror: {:?}", self_id, path, err);
                        error!("{}", message);
                        Err(message)
                    }
                }
            }
        }
    }
    ///
    /// Loads retained on the disk points to the self cache
    fn load(self_id: &str, name: &Name, cache: &Arc<RwLock<IndexMap<String, Point, BuildHasherDefault<FxHasher>>>>) {
        match cache.write() {
            Ok(mut cache) => {
                let path = Name::new("assets/cache/", name.join()).join().trim_start_matches('/').to_owned();
                let path = Path::new(&path).join("cache.json");
                match fs::OpenOptions::new().read(true).open(&path) {
                    Ok(f) => {
                        match serde_json::from_reader::<_, Vec<Point>>(f) {
                            Ok(v) => {
                                for point in v {
                                    cache.insert(point.dest(), point);
                                }
                                info!("{}.load | Retained cache loaded from: '{:?}'", self_id, path);
                            }
                            Err(err) => {
                                error!("{}.load | Deserialize error: '{:?}'\n\tin file: {:?}", self_id, err, path);
                            }
                        };
                    }
                    Err(err) => {
                        error!("{}.load | Error open file: '{:?}'\n\terror: {:?}", self_id, path, err);
                    }
                }
            }
            Err(err) => {
                error!("{}.load | Error write access cache: {:?}", self_id, err);
            }
        };
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
    fn write<S: Serialize>(self_id: &str, name: &Name, points: Vec<S>) -> Result<(), String> {
        match Self::create_dir(self_id, Name::new("assets/cache/", name.join()).join().trim_start_matches('/')) {
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
                                debug!("{}.write | Cache stored in: {:?}", self_id, path);
                            }
                            Err(err) => {
                                message = format!("{}.write | Error writing to file: '{:?}'\n\terror: {:?}", self_id, path, err);
                                error!("{}", message);
                            }
                        };
                        if message.is_empty() {Ok(())} else {Err(message)}
                    }
                    Err(err) => {
                        let message = format!("{}.write | Error open file: '{:?}'\n\terror: {:?}", self_id, path, err);
                        error!("{}", message);
                        Err(message)
                    }
                }
            }
            Err(err) => {
                error!("{:#?}", err);
                Err(err)
            }
        }
    }
    ///
    /// Stores self.cache on the disk
    fn store<T: BuildHasher>(self_id: &str, name: &Name, points: &IndexMap<String, Point, T>, status: Status) -> Result<(), String> {
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
        Self::write(self_id, name, points)
    }
    ///
    /// Fills self cache with initial values for all configured points
    pub fn initial(
        self_id: &str, 
        tx_id: usize, 
        cache: &Arc<RwLock<IndexMapFxHasher<String, Point>>>, 
        points: &[PointConfig],
        initial_status: Status,
    ) {
        match cache.write() {
            Ok(mut cache) => {
                let timestamp = Utc::now();
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
            Err(err) => {
                error!("{}.initial | Error write access cache: {:?}", self_id, err);
            }
        }
    }
}
//
//
impl Object for CacheService {
    fn id(&self) -> &str {
        &self.id
    }
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
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}
//
//
impl Service for CacheService {
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let self_name = self.name.clone();
        let tx_id = PointTxId::from_str(&self_name.join());
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let cache = self.cache.clone();
        let point_configs = services.rlock(&self_id).points(&self_name.join())
            .then(
                |points| points,
            |err| {
                error!("{}.run | Requesting Points error: {:?}", self_id, err);
                vec![]
            }
        );
        let (service_name, points) = self.subscriptions(&conf, &point_configs);
        debug!("{}.run | points: {:#?}", self_id, points.len());
        trace!("{}.run | points: {:#?}", self_id, points);
        let (_, rx_recv) = services.wlock(&self_id).subscribe(
            &service_name,
            &self.name.join(),
            &points,
        );
        let mut dely_store = DelyStore::new(conf.retain_delay);
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
            let initial_status = Status::Invalid;
            let retain_status = Status::Invalid;
            Self::initial(&self_id, tx_id, &cache, &point_configs, initial_status);
            Self::load(&self_id, &self_name, &cache);
            'main: loop {
                match rx_recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        match cache.write() {
                            Ok(mut cache) => {
                                cache.insert(point.dest(), point);
                                if dely_store.exceeded() && Self::store(&self_id, &self_name, &cache, retain_status).is_ok() {
                                    dely_store.set_stored();
                                }
                            }
                            Err(err) => {
                                error!("{}.run | Error write access cache: {:?}", self_id, err);
                            }
                        }
                    }
                    Err(err) => {
                        match err {
                            RecvTimeoutError::Timeout => {
                                trace!("{}.run | Receive error: {:?}", self_id, err);
                            }
                            RecvTimeoutError::Disconnected => {
                                error!("{}.run | Error receiving from queue: {:?}", self_id, err);
                                break 'main;
                            }
                        }
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    if !dely_store.stored() {
                        _ = Self::store(&self_id, &self_name, &cache.read().unwrap(), retain_status);
                    }
                    break;
                }
            }
            if let Err(err) = services.wlock(&self_id).unsubscribe(&service_name, &self_name.join(), &points) {
                error!("{}.run | Unsubscribe error: {:#?}", self_id, err);
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
    fn gi(&self, receiver_name: &str, points: &[SubscriptionCriteria]) -> Receiver<Point> {
        let self_id = self.id.clone();
        info!("{}.gi | Gi requested from: {}", self_id, receiver_name);
        let (send, recv) = mpsc::channel();
        let self_cache = self.cache.clone();
        let points = points.to_owned();
        thread::spawn(move || {
            if points.is_empty() {
                match self_cache.read() {
                    Ok(cache) => {
                        for point in cache.values() {
                            match send.send(point.clone()) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("{}.gi | Send error: {:#?}", self_id, err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        error!("{}.gi | Error read access cache: {:#?}", self_id, err);
                    }
                }
            } else {
                match self_cache.read() {
                    Ok(cache) => {
                        for point in points {
                            match cache.get(&point.destination()) {
                                Some(point) => {
                                    match send.send(point.clone()) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("{}.gi | Send error: {:#?}", self_id, err);
                                        }
                                    }
                                }
                                None => {
                                    error!("{}.gi | Error, requested point '{}' - not found", self_id, point.destination());
                                }
                            }
                        }

                    }
                    Err(err) => {
                        error!("{}.gi | Error read access cache: {:#?}", self_id, err);
                    }
                }
            }
        });
        recv
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
