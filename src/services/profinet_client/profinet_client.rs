use std::{
    fmt::Debug, hash::BuildHasherDefault,
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};
use coco::Stack;
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    collections::FxIndexMap, kernel::state::ChangeNotify,
    services::{
        conf::DiagKeywd, entity::{Cot, Name, Object, Point, PointConfig, PointHlr, PointTxId, Status},
        Service, ServiceCycle,
        Services, SubscriptionCriteria,
    },
    sync::channel::{RecvTimeoutError, Sender}, thread_pool::Scheduler,
};
use crate::{
    conf::profinet_client_config::profinet_client_config::ProfinetClientConfig,
    core_::{
        constants::constants::RECV_TIMEOUT, failure::ErrorLimit, Mutex,
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
    dbg: Dbg,
    name: Name,
    conf: ProfinetClientConfig,
    services: Arc<Services>,
    diagnosis: Arc<Mutex<FxIndexMap<DiagKeywd, DiagPoint>>>,
    handle: Stack<(String, JoinHandle<()>)>,
    is_finished: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
}
//
//
impl ProfinetClient {
    ///
    /// Creates new instance of the ProfinetClient
    pub fn new(conf: ProfinetClientConfig, services: Arc<Services>, schrduler: Scheduler) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        let diagnosis = Arc::new(Mutex::new(conf.diagnosis.iter().map(|(keywd, conf)| {
            (keywd.to_owned(), DiagPoint::new(tx_id, conf.clone()))
        }).collect()));
        Self {
            tx_id,
            dbg: Dbg::new(conf.name.parent(), conf.name.me()),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            diagnosis,
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Sends diagnosis point
    fn yield_diagnosis(
        dbg: &Dbg,
        diagnosis: &Arc<Mutex<FxIndexMap<DiagKeywd, DiagPoint>>>,
        kewd: &DiagKeywd,
        value: Status,
        tx_send: &Sender<Point>,
    ) {
        match diagnosis.lock().get_mut(kewd) {
            Some(point) => {
                log::debug!("{}.yield_diagnosis | Sending diagnosis point '{}' ", dbg, kewd);
                if let Some(point) = point.next(value) {
                    if let Err(err) = tx_send.send(point) {
                        log::warn!("{}.yield_status | Send error: {}", dbg, err);
                    }
                }
            }
            None => log::debug!("{}.yield_diagnosis | Diagnosis point '{}' - not configured", dbg, kewd),
        }
    }
    ///
    /// Sends all configured points from the current DB with the given status
    fn yield_status(dbg: &Dbg, dbs: &mut FxIndexMap<String, ProfinetDb>, tx_send: &Sender<Point>) {
        for (db_name, db) in dbs {
            log::debug!("{}.yield_status | DB '{}' - sending Invalid status...", dbg, db_name);
            match db.yield_status(Status::Invalid, tx_send) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("{}.yield_status | send errors: \n\t{:?}", dbg, err);
                }
            };
        }
    }
    ///
    /// Reads data slice from the S7 device,
    fn read(&self, tx_send: Sender<Point>) -> Result<JoinHandle<()>, std::io::Error> {
        log::info!("{}.read | starting...", self.dbg);
        let dbg = self.dbg.clone();
        let tx_id = self.tx_id;
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let diagnosis = self.diagnosis.clone();
        match conf.cycle {
            Some(cycle_interval) => {
                if cycle_interval > Duration::ZERO {
                    log::info!("{}.read | Preparing thread...", dbg);
                    let handle = thread::Builder::new().name(format!("{}.read", dbg)).spawn(move || {
                        let mut is_connected = ChangeNotify::new(
                            &dbg,
                            false,
                            vec![
                                (true,  Box::new(|message| log::info!("{}", message))),
                                (false, Box::new(|message| log::warn!("{}", message))),
                            ],
                        );
                        let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
                        for (db_name, db_conf) in conf.dbs {
                            log::info!("{}.read | configuring DB: {:?}...", dbg, db_name);
                            let db = ProfinetDb::new(&dbg, tx_id, &db_conf);
                            dbs.insert(db_name.clone(), db);
                            log::info!("{}.read | configuring DB: {:?} - ok", dbg, db_name);
                        }
                        let mut cycle = ServiceCycle::new(&dbg, cycle_interval);
                        let mut client = S7Client::new(dbg.clone(), conf.ip.clone());
                        'main: while !exit.load(Ordering::SeqCst) {
                            let mut error_limit = ErrorLimit::new(3);
                            let mut status;
                            match client.connect() {
                                Ok(_) => {
                                    status = Status::Ok;
                                    is_connected.add(true, format!("{}.read | Connection established", dbg));
                                    Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Connection, Status::Ok, &tx_send);
                                    'read: while !exit.load(Ordering::SeqCst) {
                                        cycle.start();
                                        for (db_name, db) in &mut dbs {
                                            log::trace!("{}.read | DB '{}' - reading...", dbg, db_name);
                                            match db.read(&client, &tx_send) {
                                                Ok(_) => {
                                                    error_limit.reset();
                                                    log::trace!("{}.read | DB '{}' - reading - ok", dbg, db_name);
                                                }
                                                Err(err) => {
                                                    log::error!("{}.read | DB '{}' - reading - error: {:?}", dbg, db_name, err);
                                                    if error_limit.add().is_err() {
                                                        log::error!("{}.read | DB '{}' - exceeded reading errors limit, trying to reconnect...", dbg, db_name);
                                                        status = Status::Invalid;
                                                        Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
                                                        if let Err(err) = client.close() {
                                                            log::error!("{}.read | {:?}", dbg, err);
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
                                        Self::yield_status(&dbg, &mut dbs, &tx_send);
                                    }
                                }
                                Err(err) => {
                                    is_connected.add(false, format!("{}.read | Connection lost: {:?}", dbg, err));
                                    log::trace!("{}.read | Connection error: {:?}", dbg, err);
                                }
                            }
                            thread::sleep(conf.reconnect_cycle);
                        }
                        log::info!("{}.read | Exit", dbg);
                    });
                    log::info!("{}.read | Started", self.dbg);
                    handle
                } else {
                    log::info!("{}.read | Disabled", self.dbg);
                    thread::Builder::new().name(format!("{}.read", dbg)).spawn(move || {})
                }
            }
            None => {
                log::info!("{}.read | Disabled", self.dbg);
                thread::Builder::new().name(format!("{}.read", dbg)).spawn(move || {})
            }
        }
    }
    ///
    /// Writes Point to the protocol (PROFINET device) specific address
    fn write(&self, tx_send: Sender<Point>) -> Result<JoinHandle<()>, std::io::Error> {
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let tx_id = self.tx_id;
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let diagnosis = self.diagnosis.clone();
        log::info!("{}.write | Preparing thread...", dbg);
        let handle = thread::Builder::new().name(format!("{}.write", dbg.clone())).spawn(move || {
            let mut is_connected = ChangeNotify::new(
                &dbg,
                false,
                vec![
                    (true,  Box::new(|message| log::info!("{}", message))),
                    (false, Box::new(|message| log::warn!("{}", message))),
                ]
            );
            let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
            let mut points: Vec<PointConfig> = vec![];
            for (db_name, db_conf) in conf.dbs {
                log::info!("{}.write | configuring ProfinetDb: {:?}...", dbg, db_name);
                let db = ProfinetDb::new(&dbg, tx_id, &db_conf);
                dbs.insert(db_name.clone(), db);
                log::info!("{}.write | configuring ProfinetDb: {:?} - ok", dbg, db_name);
                points.extend(db_conf.points());
            }
            let points = points.iter().map(|point_conf| {
                SubscriptionCriteria::new(&point_conf.name, Cot::Act)
            }).collect::<Vec<SubscriptionCriteria>>();
            log::debug!("{}.write | Points subscribed on: ({})", dbg, points.len());
            for name in &points {
                println!("\t{:?}", name);
            }
            let (_, rx_recv) = services.subscribe(&conf.subscribe, &self_name.join(), &points);
            let mut client = S7Client::new(dbg.clone(), conf.ip.clone());
            'main: while !exit.load(Ordering::SeqCst) {
                let mut errors_limit = ErrorLimit::new(3);
                thread::sleep(conf.reconnect_cycle);
                match client.connect() {
                    Ok(_) => {
                        is_connected.add(true, format!("{}.write | Connection established", dbg));
                        Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Connection, Status::Ok, &tx_send);
                        'write: while !exit.load(Ordering::SeqCst) {
                            match rx_recv.recv_timeout(RECV_TIMEOUT) {
                                Ok(point) => {
                                    let point_name = point.name();
                                    let point_value = point.value();
                                    let db_name = point_name.split('/').nth(3).unwrap();
                                    log::debug!("{}.write | ProfinetDb '{}' - writing point '{}'\t({:?})...", dbg, db_name, point_name, point_value);
                                    // let dbName = point_name.split("/").skip(1).collect::<String>();
                                    match dbs.get_mut(db_name) {
                                        Some(db) => {
                                            match db.write(&client, point.clone()) {
                                                Ok(_) => {
                                                    errors_limit.reset();
                                                    log::debug!("{}.write | ProfinetDb '{}' - writing point '{}'\t({:?}) - ok", dbg, db_name, point_name, point_value);
                                                    let reply = Self::reply_point(tx_id, point);
                                                    match tx_send.send(reply.clone()) {
                                                        Ok(_) => log::debug!("{}.write | ProfinetDb '{}' - sent reply: {:#?}", dbg, db_name, reply),
                                                        Err(err) => log::error!("{}.write | Error sending to queue: {:?}", dbg, err),
                                                        // break 'main;
                                                    };
                                                }
                                                Err(err) => {
                                                    log::warn!("{}.write | ProfinetDb '{}' - write - error: {:?}", dbg, db_name, err);
                                                    if errors_limit.add().is_err() {
                                                        log::error!("{}.write | ProfinetDb '{}' - exceeded writing errors limit, trying to reconnect...", dbg, db_name);
                                                        Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
                                                        if let Err(err) = tx_send.send(Point::String(PointHlr::new(
                                                            tx_id,
                                                            &point_name,
                                                            format!("Write error: {}", err),
                                                            Status::Ok,
                                                            Cot::ActErr,
                                                            chrono::offset::Utc::now(),
                                                        ))) {
                                                            log::error!("{}.write | Error sending to queue: {:?}", dbg, err);
                                                            // break 'main;
                                                        };
                                                        if let Err(err) = client.close() {
                                                            log::error!("{}.write | {:?}", dbg, err);
                                                        };
                                                        break 'write;
                                                    }
                                                }
                                            }
                                        }
                                        None => {
                                            log::error!("{}.write | ProfinetDb '{}' - not found", dbg, db_name);
                                        }
                                    };
                                }
                                Err(err) => {
                                    match err {
                                        RecvTimeoutError::Timeout => {}
                                        _ => {
                                            log::error!("{}.write | Error receiving from queue: {:?}", dbg, err);
                                            Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
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
                        is_connected.add(false, format!("{}.write | Connection lost: {:?}", dbg, err));
                        log::trace!("{}.write | Connection error: {:?}", dbg, err);
                    }
                }
            }
            log::info!("{}.write | Exit", dbg);
        });
        log::info!("{}.write | Started", self.dbg);
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
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for ProfinetClient {
    //
    //
    fn run(&self) -> Result<(), Error> {
        let tx_send = self.services.get_link(&self.conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.dbg, err);
        });
        Self::yield_diagnosis(&self.dbg, &self.diagnosis.clone(), &DiagKeywd::Status, Status::Ok, &tx_send);
        Self::yield_diagnosis(&self.dbg, &self.diagnosis.clone(), &DiagKeywd::Connection, Status::Invalid, &tx_send);
        let handle_read = self.read(tx_send.clone());
        let handle_write = self.write(tx_send);
        log::info!("{}.run | started", self.dbg);
        let error = Error::new(&self.dbg, "run");
        match (handle_read, handle_write) {
            (Ok(handle_read), Ok(handle_write)) => {
                self.handle.push((format!("{}/read", self.dbg), handle_read));
                self.handle.push((format!("{}/write", self.dbg), handle_write));
                Ok(())
            }
            (Ok(handle_read), Err(err)) => {
                self.exit();
                handle_read.join().unwrap();
                Err(error.pass_with("Error starting inner thread 'read'", err.to_string()))
            }
            (Err(err), Ok(handle_write)) => {
                self.exit();
                handle_write.join().unwrap();
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
    fn points(&self) -> Vec<PointConfig> {
        self.conf.points()
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        while !self.handle.is_empty() {
            if let Some((name, handle)) = self.handle.pop() {
                if let Err(err) = handle.join() {
                    log::warn!("{}.wait | Error join '{name}': {:?}", self.dbg, err);
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
