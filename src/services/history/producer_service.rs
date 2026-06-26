use std::{fmt::Debug, fs, io::Write, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use chrono::{DateTime, Utc};
use concat_string::concat_string;
use indexmap::IndexMap;
use rand::Rng;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{
    entity::{
        Cot, Name, Object,
        Point, PointConf, PointType, PointHlr, PointTxId,
        Status,
    }, types::Bool, Service, ServiceCycle, Services
}, sync::Handles, thread_pool::Scheduler};
use serde_json::json;
use testing::entities::test_value::Value;
use super::producer_service_conf::ProducerServiceConf;
///
/// Service for debuging / testing purposes
///  - prodices Point's into the configured service's queue
pub struct ProducerService {
    dbg: Dbg,
    name: Name,
    conf: ProducerServiceConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
// 
impl ProducerService {
    pub fn new(conf: ProducerServiceConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(conf.name.parent(), format!("{}(ProducerService)", conf.name.me()));
        Self {
            name: conf.name.clone(),
            conf,
            services,
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns map of the ParsePoint built from the provided PointConf's
    fn build_gen_points(parent: impl Into<String>, txid: usize, points: Vec<PointConf>) -> IndexMap<String, Box<impl ParsePoint<Value>>> {
        let parent = parent.into();
        let mut gen_points = IndexMap::new();
        for point_conf in points {
            match point_conf.type_ {
                PointType::Bool => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(&parent, txid, point_conf.name.clone(), &point_conf)));
                }
                PointType::Int => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(&parent, txid, point_conf.name.clone(), &point_conf)));
                }
                PointType::Real => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(&parent, txid, point_conf.name.clone(), &point_conf)));
                }
                PointType::Double => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(&parent, txid, point_conf.name.clone(), &point_conf)));
                }
                PointType::String => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(&parent, txid, point_conf.name.clone(), &point_conf)));
                }
                PointType::Bytes => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(&parent, txid, point_conf.name.clone(), &point_conf)));
                }
                PointType::Json => {
                    gen_points.insert(point_conf.name.clone(), Box::new(PointGen::new(&parent, txid, point_conf.name.clone(), &point_conf)));
                }
            }
        }
        gen_points
    }
    ///
    /// Writes Point into the log file ./logs/parent/points.log
    fn log(dbg: &Dbg, parent: &Name, point: &Point) {
        let path = concat_string!("./logs", parent.join(), "/points.log");
        match fs::OpenOptions::new().create(true).append(true).open(&path) {
            Ok(mut f) => {
                f.write_fmt(format_args!("{:?}\n", point)).unwrap();
            }
            Err(err) => {
                if log::max_level() >= log::LevelFilter::Trace {
                    log::warn!("{}.log | Error open file: '{}'\n\terror: {:?}", dbg, path, err)
                }
            }
        }
    }
}
//
//
impl Object for ProducerService {
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
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for ProducerService {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let txid = PointTxId::from_str(&self_name.join());
        let exit = self.exit.clone();
        let debug = self.conf.debug;
        let interval = self.conf.cycle.unwrap_or(Duration::ZERO);
        let delayed = !interval.is_zero();
        let mut cycle = ServiceCycle::new(&dbg, interval);
        let send = self.services.get_link(&self.conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", dbg, err);
        });
        let mut gen_points = Self::build_gen_points(self_name.join(), txid, self.conf.points());
        let handle = self.scheduler.spawn(move || {
            'main: loop {
                log::trace!("{}.run | Step...", dbg);
                for (_, gen_point) in &mut gen_points {
                    cycle.start();
                    if let Some(point) = gen_point.next(&Value::Bool(false), Utc::now()) {
                        match send.send(point.clone()) {
                            Ok(_) => {
                                // if debug {debug!("{}.run | sent point: {:?}", self_id, point);}
                                if debug {Self::log(&dbg, &self_name, &point);}
                            }
                            Err(err) => {
                                log::warn!("{}.run | Send error: {:?}", dbg, err);
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
            log::info!("{}.run | Exit", dbg);
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Started", self.dbg);
                self.handles.push(handle);
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
        self.exit.store(true, Ordering::Relaxed);
    }
}
///
/// Creates new Point's on call method 'next'
#[derive(Debug, Clone)]
pub struct PointGen {
    dbg: Dbg,
    pub txid: usize,
    _type: PointType,
    pub name: String,
    pub value: Value,
    pub status: Status,
    // pub history: PointConfHistory,
    // pub alarm: Option<u8>,
    pub timestamp: DateTime<Utc>,
    is_changed: bool,
}
//
//
impl PointGen {
    ///
    /// Creates new instance of the PointGen
    pub fn new(
        parent: impl Into<String>,
        txid: usize,
        name: String,
        config: &PointConf,
        // filter: Filter<T>,
    ) -> PointGen {
        PointGen {
            dbg: Dbg::new(parent, format!("PointGen({name})")),
            txid,
            _type: config.type_.clone(),
            name,
            value: Value::Bool(false),
            status: Status::Invalid,
            is_changed: false,
            // history: config.history.clone(),
            // alarm: config.alarm,
            timestamp: Utc::now(),
        }
    }
    ///
    /// Returns Point
    fn to_point(&self) -> Option<Point> {
        if self.is_changed {
            log::trace!("{}.to_point | generating point type '{:?}'...", self.dbg, self._type);
            match &self._type {
                PointType::Bool => {
                    Some(Point::Bool(PointHlr::new(
                        self.txid, 
                        &self.name, 
                        Bool(test_data_bool().as_bool()), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointType::Int => {
                    Some(Point::Int(PointHlr::new(
                        self.txid, 
                        &self.name, 
                        test_data_int().as_int(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointType::Real => {
                    Some(Point::Real(PointHlr::new(
                        self.txid, 
                        &self.name, 
                        test_data_real().as_real(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointType::Double => {
                    Some(Point::Double(PointHlr::new(
                        self.txid, 
                        &self.name, 
                        test_data_double().as_double(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointType::String => {
                    Some(Point::String(PointHlr::new(
                        self.txid, 
                        &self.name, 
                        test_data_double().as_double().to_string(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointType::Bytes => {
                    Some(Point::Bytes(PointHlr::new(
                        self.txid, 
                        &self.name, 
                        test_data_double().as_double().to_be_bytes().to_vec(), 
                        self.status, 
                        Cot::Inf,
                        self.timestamp,
                    )))
                }
                PointType::Json => {
                    Some(Point::String(PointHlr::new(
                        self.txid, 
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
    #[allow(unused)]
    fn next_status(&mut self, status: Status) -> Option<Point>;
    ///
    /// Returns true if value or status was updated since last call [addRaw()]
    #[allow(unused)]
    fn is_changed(&self) -> bool;
}


fn get_random_index(len: usize) -> usize {
    let mut rnd = rand::rng();
    rnd.random_range(0..len)
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