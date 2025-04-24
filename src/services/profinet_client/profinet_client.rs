use std::{
    fmt::Debug, hash::BuildHasherDefault,
    sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Sender}, Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};
use coco::Stack;
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_core::error::Error;
use sal_sync::{
    collections::FxIndexMap, kernel::state::ChangeNotify,
    services::{
        conf::DiagKeywd, entity::{Cot, Name, Object, Point, PointConfig, PointHlr, PointTxId, Status},
        safe_lock::rwlock::SafeLock,
        service::{Service, ServiceCycle},
        services::Services, subscription::SubscriptionCriteria,
    },
};
use testing::stuff::wait::WaitTread;
use crate::{
    conf::profinet_client_config::profinet_client_config::ProfinetClientConfig,
    core_::{
        constants::constants::RECV_TIMEOUT, failure::errors_limit::ErrorLimit,
    },
    services::{
        diagnosis::diag_point::DiagPoint,
        profinet_client::{profinet_db::ProfinetDb, s7::s7_client::S7Client},
    },
};
///
/// Cyclically reads adressess from the PROFINET device and yields changed to the MultiQueue
/// Writes Point to the protocol (PROFINET device) specific address
pub struct ProfinetClient {
    tx_id: usize,
    id: String,
    name: Name,
    conf: ProfinetClientConfig,
    services: Arc<RwLock<Services>>,
    diagnosis: Arc<Mutex<FxIndexMap<DiagKeywd, DiagPoint>>>,
    handle: Stack<(String, JoinHandle<()>)>,
    exit: Arc<AtomicBool>,
}
//
//
impl ProfinetClient {
    ///
    /// Creates new instance of the ProfinetClient
    pub fn new(conf: ProfinetClientConfig, services: Arc<RwLock<Services>>) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        let diagnosis = Arc::new(Mutex::new(conf.diagnosis.iter().map(|(keywd, conf)| {
            (keywd.to_owned(), DiagPoint::new(tx_id, conf.clone()))
        }).collect()));
        Self {
            tx_id,
            id: format!("{}", conf.name),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            diagnosis,
            handle: Stack::new(),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Sends diagnosis point
    fn yield_diagnosis(
        self_id: &str,
        diagnosis: &Arc<Mutex<FxIndexMap<DiagKeywd, DiagPoint>>>,
        kewd: &DiagKeywd,
        value: Status,
        tx_send: &Sender<Point>,
    ) {
        match diagnosis.lock() {
            Ok(mut diagnosis) => {
                match diagnosis.get_mut(kewd) {
                    Some(point) => {
                        log::debug!("{}.yield_diagnosis | Sending diagnosis point '{}' ", self_id, kewd);
                        if let Some(point) = point.next(value) {
                            if let Err(err) = tx_send.send(point) {
                                log::warn!("{}.yield_status | Send error: {}", self_id, err);
                            }
                        }
                    }
                    None => log::debug!("{}.yield_diagnosis | Diagnosis point '{}' - not configured", self_id, kewd),
                }
            }
            Err(err) => log::error!("{}.yield_diagnosis | Diagnosis lock error: {:#?}", self_id, err),
        }
    }
    ///
    /// Sends all configured points from the current DB with the given status
    fn yield_status(self_id: &str, dbs: &mut FxIndexMap<String, ProfinetDb>, tx_send: &Sender<Point>) {
        for (db_name, db) in dbs {
            log::debug!("{}.yield_status | DB '{}' - sending Invalid status...", self_id, db_name);
            match db.yield_status(Status::Invalid, tx_send) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("{}.yield_status | send errors: \n\t{:?}", self_id, err);
                }
            };
        }
    }
    ///
    /// Reads data slice from the S7 device,
    fn read(&mut self, tx_send: Sender<Point>) -> Result<JoinHandle<()>, std::io::Error> {
        log::info!("{}.read | starting...", self.id);
        let self_id = self.id.clone();
        let tx_id = self.tx_id;
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let diagnosis = self.diagnosis.clone();
        match conf.cycle {
            Some(cycle_interval) => {
                if cycle_interval > Duration::ZERO {
                    log::info!("{}.read | Preparing thread...", self_id);
                    let handle = thread::Builder::new().name(format!("{}.read", self_id)).spawn(move || {
                        let mut is_connected = ChangeNotify::new(
                            &self_id,
                            false,
                            vec![
                                (true,  Box::new(|message| log::info!("{}", message))),
                                (false, Box::new(|message| log::warn!("{}", message))),
                            ],
                        );
                        let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
                        for (db_name, db_conf) in conf.dbs {
                            log::info!("{}.read | configuring DB: {:?}...", self_id, db_name);
                            let db = ProfinetDb::new(&self_id, tx_id, &db_conf);
                            dbs.insert(db_name.clone(), db);
                            log::info!("{}.read | configuring DB: {:?} - ok", self_id, db_name);
                        }
                        let mut cycle = ServiceCycle::new(&self_id, cycle_interval);
                        let mut client = S7Client::new(self_id.clone(), conf.ip.clone());
                        'main: while !exit.load(Ordering::SeqCst) {
                            let mut error_limit = ErrorLimit::new(3);
                            let mut status;
                            match client.connect() {
                                Ok(_) => {
                                    status = Status::Ok;
                                    is_connected.add(true, format!("{}.read | Connection established", self_id));
                                    Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Connection, Status::Ok, &tx_send);
                                    'read: while !exit.load(Ordering::SeqCst) {
                                        cycle.start();
                                        for (db_name, db) in &mut dbs {
                                            log::trace!("{}.read | DB '{}' - reading...", self_id, db_name);
                                            match db.read(&client, &tx_send) {
                                                Ok(_) => {
                                                    error_limit.reset();
                                                    log::trace!("{}.read | DB '{}' - reading - ok", self_id, db_name);
                                                }
                                                Err(err) => {
                                                    log::error!("{}.read | DB '{}' - reading - error: {:?}", self_id, db_name, err);
                                                    if error_limit.add().is_err() {
                                                        log::error!("{}.read | DB '{}' - exceeded reading errors limit, trying to reconnect...", self_id, db_name);
                                                        status = Status::Invalid;
                                                        Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
                                                        if let Err(err) = client.close() {
                                                            log::error!("{}.read | {:?}", self_id, err);
                                                        };
                                                        break 'read;
                                                    }
                                                }
                                            }
                                            if exit.load(Ordering::SeqCst) {
                                                break 'main;
                                            }
                                        }
                                        cycle.wait();
                                    }
                                    if status != Status::Ok {
                                        Self::yield_status(&self_id, &mut dbs, &tx_send);
                                    }
                                }
                                Err(err) => {
                                    is_connected.add(false, format!("{}.read | Connection lost: {:?}", self_id, err));
                                    log::trace!("{}.read | Connection error: {:?}", self_id, err);
                                }
                            }
                            thread::sleep(conf.reconnect_cycle);
                        }
                        log::info!("{}.read | Exit", self_id);
                    });
                    log::info!("{}.read | Started", self.id);
                    handle
                } else {
                    log::info!("{}.read | Disabled", self.id);
                    thread::Builder::new().name(format!("{}.read", self_id)).spawn(move || {})
                }
            }
            None => {
                log::info!("{}.read | Disabled", self.id);
                thread::Builder::new().name(format!("{}.read", self_id)).spawn(move || {})
            }
        }
    }
    ///
    /// Writes Point to the protocol (PROFINET device) specific address
    fn write(&mut self, tx_send: Sender<Point>) -> Result<JoinHandle<()>, std::io::Error> {
        let self_id = self.id.clone();
        let tx_id = self.tx_id;
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let diagnosis = self.diagnosis.clone();
        log::info!("{}.write | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.write", self_id.clone())).spawn(move || {
            let mut is_connected = ChangeNotify::new(
                &self_id,
                false,
                vec![
                    (true,  Box::new(|message| log::info!("{}", message))),
                    (false, Box::new(|message| log::warn!("{}", message))),
                ]
            );
            let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
            let mut points: Vec<PointConfig> = vec![];
            for (db_name, db_conf) in conf.dbs {
                log::info!("{}.write | configuring ProfinetDb: {:?}...", self_id, db_name);
                let db = ProfinetDb::new(&self_id, tx_id, &db_conf);
                dbs.insert(db_name.clone(), db);
                log::info!("{}.write | configuring ProfinetDb: {:?} - ok", self_id, db_name);
                points.extend(db_conf.points());
            }
            let points = points.iter().map(|point_conf| {
                SubscriptionCriteria::new(&point_conf.name, Cot::Act)
            }).collect::<Vec<SubscriptionCriteria>>();
            log::debug!("{}.write | Points subscribed on: ({})", self_id, points.len());
            for name in &points {
                println!("\t{:?}", name);
            }
            let (_, rx_recv) = services.wlock(&self_id).subscribe(&conf.subscribe, &self_id, &points);
            let mut client = S7Client::new(self_id.clone(), conf.ip.clone());
            'main: while !exit.load(Ordering::SeqCst) {
                let mut errors_limit = ErrorLimit::new(3);
                thread::sleep(conf.reconnect_cycle);
                match client.connect() {
                    Ok(_) => {
                        is_connected.add(true, format!("{}.write | Connection established", self_id));
                        Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Connection, Status::Ok, &tx_send);
                        'write: while !exit.load(Ordering::SeqCst) {
                            match rx_recv.recv_timeout(RECV_TIMEOUT) {
                                Ok(point) => {
                                    let point_name = point.name();
                                    let point_value = point.value();
                                    let db_name = point_name.split('/').nth(3).unwrap();
                                    log::debug!("{}.write | ProfinetDb '{}' - writing point '{}'\t({:?})...", self_id, db_name, point_name, point_value);
                                    // let dbName = point_name.split("/").skip(1).collect::<String>();
                                    match dbs.get_mut(db_name) {
                                        Some(db) => {
                                            match db.write(&client, point.clone()) {
                                                Ok(_) => {
                                                    errors_limit.reset();
                                                    log::debug!("{}.write | ProfinetDb '{}' - writing point '{}'\t({:?}) - ok", self_id, db_name, point_name, point_value);
                                                    let reply = Self::reply_point(tx_id, point);
                                                    match tx_send.send(reply.clone()) {
                                                        Ok(_) => log::debug!("{}.write | ProfinetDb '{}' - sent reply: {:#?}", self_id, db_name, reply),
                                                        Err(err) => log::error!("{}.write | Error sending to queue: {:?}", self_id, err),
                                                        // break 'main;
                                                    };
                                                }
                                                Err(err) => {
                                                    log::warn!("{}.write | ProfinetDb '{}' - write - error: {:?}", self_id, db_name, err);
                                                    if errors_limit.add().is_err() {
                                                        log::error!("{}.write | ProfinetDb '{}' - exceeded writing errors limit, trying to reconnect...", self_id, db_name);
                                                        Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
                                                        if let Err(err) = tx_send.send(Point::String(PointHlr::new(
                                                            tx_id,
                                                            &point_name,
                                                            format!("Write error: {}", err),
                                                            Status::Ok,
                                                            Cot::ActErr,
                                                            chrono::offset::Utc::now(),
                                                        ))) {
                                                            log::error!("{}.write | Error sending to queue: {:?}", self_id, err);
                                                            // break 'main;
                                                        };
                                                        if let Err(err) = client.close() {
                                                            log::error!("{}.write | {:?}", self_id, err);
                                                        };
                                                        break 'write;
                                                    }
                                                }
                                            }
                                        }
                                        None => {
                                            log::error!("{}.write | ProfinetDb '{}' - not found", self_id, db_name);
                                        }
                                    };
                                }
                                Err(err) => {
                                    match err {
                                        mpsc::RecvTimeoutError::Timeout => {}
                                        mpsc::RecvTimeoutError::Disconnected => {
                                            log::error!("{}.write | Error receiving from queue: {:?}", self_id, err);
                                            Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
                                            break 'main;
                                        }
                                    }
                                }
                            }
                            if exit.load(Ordering::SeqCst) {
                                break 'main;
                            }
                        }
                    }
                    Err(err) => {
                        is_connected.add(false, format!("{}.write | Connection lost: {:?}", self_id, err));
                        log::trace!("{}.write | Connection error: {:?}", self_id, err);
                    }
                }
            }
            log::info!("{}.write | Exit", self_id);
        });
        log::info!("{}.write | Started", self.id);
        handle
    }
    ///
    /// Creates confirmation reply point with the same value & Cot::ActCon
    fn reply_point(tx_id: usize, point: Point) -> Point {
        match point {
            Point::Bool(point) => {
                Point::Bool(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
            Point::Int(point) => {
                Point::Int(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
            Point::Real(point) => {
                Point::Real(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
            Point::Double(point) => {
                Point::Double(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
            Point::String(point) => {
                Point::String(PointHlr::new(
                    tx_id,
                    &point.name,
                    point.value,
                    Status::Ok,
                    Cot::ActCon,
                    chrono::offset::Utc::now(),
                ))
            },
        }
    }
}
//
//
impl Object for ProfinetClient {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for ProfinetClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProfinetClient")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for ProfinetClient {
    //
    //
    fn run(&mut self) -> Result<(), Error> {
        let tx_send = self.services.rlock(&self.id).get_link(&self.conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.id, err);
        });
        Self::yield_diagnosis(&self.id, &self.diagnosis.clone(), &DiagKeywd::Status, Status::Ok, &tx_send);
        Self::yield_diagnosis(&self.id, &self.diagnosis.clone(), &DiagKeywd::Connection, Status::Invalid, &tx_send);
        let handle_read = self.read(tx_send.clone());
        let handle_write = self.write(tx_send);
        log::info!("{}.run | started", self.id);
        let error = Error::new(&self.id, "run");
        match (handle_read, handle_write) {
            (Ok(handle_read), Ok(handle_write)) => {
                self.handle.push((format!("{}/read", self.id), handle_read));
                self.handle.push((format!("{}/write", self.id), handle_write));
                Ok(())
            }
            (Ok(handle_read), Err(err)) => {
                self.exit();
                handle_read.wait().unwrap();
                Err(error.pass_with("Error starting inner thread 'read'", err.to_string()))
            }
            (Err(err), Ok(handle_write)) => {
                self.exit();
                handle_write.wait().unwrap();
                Err(error.pass_with("Error starting inner thread 'write'", err.to_string()))
            }
            (Err(read_err), Err(write_err)) => {
                Err(error.pass_with(
                    "Error starting inner thread",
                    format!("\n\t  read: {:#?}\n\t write: {:#?}", read_err, write_err),
                ))
            }
        }
    }
    //
    //
    fn wait(&self) -> sal_sync::services::future::Future<()> {
        let dbg = self.id.clone();
        let (future, sink) = sal_sync::services::future::Future::new();
        let mut handles = vec![];
        while !self.handle.is_empty() {
            if let Some(h) = self.handle.pop() {
                handles.push(h);
            }
        }
        std::thread::spawn(move|| {
            for (id, h) in handles {
                if let Err(err) = h.join() {
                    log::warn!("{dbg}.wait | Wait for '{id}' error: {:?}", err);
                }
            }
            sink.add(());
        });
        future
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
    }
}
