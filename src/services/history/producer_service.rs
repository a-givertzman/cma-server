use std::{fmt::Debug, fs, io::Write, sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock}, thread, time::Duration};
use chrono::{DateTime, Utc};
use concat_string::concat_string;
use indexmap::IndexMap;
use log::{info, trace, warn};
use rand::Rng;
use sal_sync::services::{
    entity::{
        cot::Cot, name::Name, object::Object,
        point::{
            point::Point, point_config::PointConfig, point_config_history::PointConfigHistory,
            point_config_type::PointConfigType, point_hlr::PointHlr, point_tx_id::PointTxId,
        },
        status::status::Status,
    },
    service::{service::Service, service_cycle::ServiceCycle, service_handles::ServiceHandles},
    types::bool::Bool,
};
use serde_json::json;
use testing::entities::test_value::Value;
use crate::services::{safe_lock::SafeLock, services::Services};
use super::producer_service_config::ProducerServiceConfig;
///
/// Service for debuging / testing purposes
///  - prodices Point's into the configured service's queue
pub struct ProducerService {
    id: String,
    name: Name,
    conf: ProducerServiceConfig,
    services: Arc<RwLock<Services>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl ProducerService {
    pub fn new(conf: ProducerServiceConfig, services: Arc<RwLock<Services>>) -> Self {
        Self {
            id: format!("{}(ProducerService)", conf.name),
            name: conf.name.clone(),
            conf,
            services,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns map of the ParsePoint built from the provided PointConfig's
    fn build_gen_points(parent_id: &str, tx_id: usize, points: Vec<PointConfig>) -> IndexMap<String, Box<impl ParsePoint<Value>>> {
        let mut gen_points = IndexMap::new();
        for point_conf in points {
            match point_conf.type_ {
                PointConfigType::Bool => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(parent_id, tx_id, point_conf.name.clone(), &point_conf)));
                }
                PointConfigType::Int => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(parent_id, tx_id, point_conf.name.clone(), &point_conf)));
                }
                PointConfigType::Real => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(parent_id, tx_id, point_conf.name.clone(), &point_conf)));
                }
                PointConfigType::Double => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(parent_id, tx_id, point_conf.name.clone(), &point_conf)));
                }
                PointConfigType::String => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(parent_id, tx_id, point_conf.name.clone(), &point_conf)));
                }
                PointConfigType::Json => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(parent_id, tx_id, point_conf.name.clone(), &point_conf)));
                }
            }
        }
        gen_points
    }
    ///
    /// Writes Point into the log file ./logs/parent/points.log
    fn log(self_id: &str, parent: &Name, point: &Point) {
        let path = concat_string!("./logs", parent.join(), "/points.log");
        match fs::OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                f.write_fmt(format_args!("{:?}\n", point)).unwrap();
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
impl Object for ProducerService {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for ProducerService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProducerService")
            .field("id", &self.id)
            .finish()
    }
}
//
//
unsafe impl Send for ProducerService {}
unsafe impl Sync for ProducerService {}
//
//
impl Service for ProducerService {
    //
    // 
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let self_name = self.name.clone();
        let tx_id = PointTxId::from_str(&self_id);
        let exit = self.exit.clone();
        let debug = self.conf.debug;
        let interval = self.conf.cycle.unwrap_or(Duration::ZERO);
        let delayed = !interval.is_zero();
        let mut cycle = ServiceCycle::new(&self.id, interval);
        let send = self.services.rlock(&self_id).get_link(&self.conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.id, err);
        });
        let mut gen_points = Self::build_gen_points(&self.id, tx_id, self.conf.points());
        let handle = thread::Builder::new().name(self_id.clone()).spawn(move || {
            'main: loop {
                trace!("{}.run | Step...", self_id);
                for (_, gen_point) in &mut gen_points {
                    cycle.start();
                    if let Some(point) = gen_point.next(&Value::Bool(false), Utc::now()) {
                        match send.send(point.clone()) {
                            Ok(_) => {
                                // if debug {debug!("{}.run | sent point: {:?}", self_id, point);}
                                if debug {Self::log(&self_id, &self_name, &point);}
                            }
                            Err(err) => {
                                warn!("{}.run | Send error: {:?}", self_id, err);
                            }
                        }
                    };
                    if delayed {
                        cycle.wait();
                    }
                    if exit.load(Ordering::SeqCst) {
                        break 'main;
                    }
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
    fn points(&self) -> Vec<PointConfig> {
        self.conf.points()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Relaxed);
    }
}
///
/// Creates new Point's on call method 'next'
#[derive(Debug, Clone)]
pub struct PointGen {
    id: String,
    pub tx_id: usize,
    _type: PointConfigType,
    pub name: String,
    pub value: Value,
    pub status: Status,
    pub history: PointConfigHistory,
    pub alarm: Option<u8>,
    pub timestamp: DateTime<Utc>,
    is_changed: bool,
}
//
//
impl PointGen {
    ///
    /// Creates new instance of the PointGen
    pub fn new(
        parent_id: &str,
        tx_id: usize,
        name: String,
        config: &PointConfig,
        // filter: Filter<T>,
    ) -> PointGen {
        PointGen {
            id: format!("{}/PointGen({})", parent_id, name),
            tx_id,
            _type: config.type_.clone(),
            name,
            value: Value::Bool(false),
            status: Status::Invalid,
            is_changed: false,
            history: config.history.clone(),
            alarm: config.alarm,
            timestamp: Utc::now(),
        }
    }
    ///
    /// Returns Point
    fn to_point(&self) -> Option<Point> {
        if self.is_changed {
            trace!("{}.to_point | generating point type '{:?}'...", self.id, self._type);
            match &self._type {
                PointConfigType::Bool => {
                    Some(Point::Bool(PointHlr::new(
                        self.tx_id, 
                        &self.name, 
                        Bool(test_data_bool().as_bool()), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointConfigType::Int => {
                    Some(Point::Int(PointHlr::new(
                        self.tx_id, 
                        &self.name, 
                        test_data_int().as_int(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointConfigType::Real => {
                    Some(Point::Real(PointHlr::new(
                        self.tx_id, 
                        &self.name, 
                        test_data_real().as_real(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointConfigType::Double => {
                    Some(Point::Double(PointHlr::new(
                        self.tx_id, 
                        &self.name, 
                        test_data_double().as_double(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointConfigType::String => {
                    Some(Point::String(PointHlr::new(
                        self.tx_id, 
                        &self.name, 
                        test_data_double().as_double().to_string(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointConfigType::Json => {
                    Some(Point::String(PointHlr::new(
                        self.tx_id, 
                        &self.name, 
                        json!(test_data_double().as_double()).to_string(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
            }
        } else {
            None
        }
    }
    ///
    /// Applyes new value
    fn add_value(&mut self, input: &Value, timestamp: DateTime<Utc>) {
        // if input != &self.value {
        // }
        self.value = input.clone();
        self.status = Status::Ok;
        self.timestamp = timestamp;
        self.is_changed = true;
    }    
}
//
//
impl ParsePoint<Value> for PointGen {
    //
    //
    fn next(&mut self, value: &Value, timestamp: DateTime<Utc>) -> Option<Point> {
        self.add_value(value, timestamp);
        match self.to_point() {
            Some(point) => {
                self.is_changed = false;
                Some(point)
            }
            None => None,
        }
    }
    //
    //
    fn next_status(&mut self, status: Status) -> Option<Point> {
        self.status = status;
        self.timestamp = Utc::now();
        self.to_point()
    }
    //
    //
    fn is_changed(&self) -> bool {
        self.is_changed
    }
}



pub trait ParsePoint<T> {
    ///
    /// Returns new point parsed from the data slice [bytes] with the given [timestamp] and Status::Ok
    fn next(&mut self, input: &T, timestamp: DateTime<Utc>) -> Option<Point>;
    ///
    /// Returns new point (prevously parsed) with the given [status]
    fn next_status(&mut self, status: Status) -> Option<Point>;
    ///
    /// Returns true if value or status was updated since last call [addRaw()]
    fn is_changed(&self) -> bool;
}


fn get_random_index(len: usize) -> usize {
    let mut rnd = rand::thread_rng();
    rnd.gen_range(0..len)
}


fn test_data_bool() -> Value {
    let data = [
        Value::Bool(true),
        Value::Bool(false),
        Value::Bool(false),
        Value::Bool(true),
        Value::Bool(true),
        Value::Bool(false),
        Value::Bool(true),
        Value::Bool(false),
        Value::Bool(true),
        Value::Bool(false),
        Value::Bool(true),
        Value::Bool(false),
    ];
    let index = get_random_index(data.len());
    data[index].clone()
}
fn test_data_int() -> Value {
    let data = [
        Value::Int(0),
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
        Value::Int(5),
        Value::Int(6),
        Value::Int(7),
        Value::Int(8),
        Value::Int(9),
    ];
    let index = get_random_index(data.len());
    data[index].clone()
}
fn test_data_real() -> Value {
    let data = [
        Value::Real(0.0),
        Value::Real(1.0),
        Value::Real(2.0),
        Value::Real(3.0),
        Value::Real(4.0),
        Value::Real(5.0),

    ];
    let index = get_random_index(data.len());
    data[index].clone()
}
fn test_data_double() -> Value {
    let data = [
        Value::Double(0.0),
        Value::Double(1.0),
        Value::Double(2.0),
        Value::Double(3.0),
        Value::Double(4.0),
        Value::Double(5.0),
    ];
    let index = get_random_index(data.len());
    data[index].clone()
}