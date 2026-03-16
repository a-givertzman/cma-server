use std::{
    fmt::Debug, hash::BuildHasherDefault,
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    thread::{self}, time::Duration,
};
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::{Error, ErrorLimit}};
use sal_sync::{
    collections::FxIndexMap, kernel::state::ChangeNotify,
    services::{
        conf::DiagKeywd, entity::{Cot, Name, Object, Point, PointConf, PointHlr, PointTxId, Status},
        Service, ServiceCycle,
        Services, SubscriptionCriteria,
    },
    sync::{channel::{RecvTimeoutError, Sender}, Handles}, thread_pool::Scheduler,
};
use crate::{
    conf::profinet_client_conf::profinet_client_conf::ProfinetClientConf,
    domain::{FxDashMap, RwLock, constants::constants::RECV_TIMEOUT},
    services::{
        diagnosis::diag_point::DiagPoint,
        profinet_client::{profinet_db::ProfinetDb, s7::s7_client::S7Client},
    },
};
///
/// 
type Diagnosis = Arc<FxDashMap<DiagKeywd, DiagPoint>>;
type ConnectionNotify = Arc<ChangeNotify<Status, Dbg>>;
///
/// Cyclically reads adressess from the PROFINET device and yields changed to the MultiQueue
/// Writes Point to the protocol (PROFINET device) specific address
pub struct ProfinetClient {
    txid: usize,
    name: Name,
    conf: ProfinetClientConf,
    services: Arc<Services>,
    diagnosis: Diagnosis,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl ProfinetClient {
    ///
    /// Creates new instance of the ProfinetClient
    pub fn new(conf: ProfinetClientConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let txid = PointTxId::from_str(&conf.name.join());
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        let diagnosis = Arc::new(conf.diagnosis.iter().map(|(keywd, conf)| {
            (keywd.to_owned(), DiagPoint::new(txid, conf.clone()))
        }).collect());
        Self {
            txid,
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            diagnosis,
            scheduler,
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Sends diagnosis point
    fn yield_diagnosis(
        dbg: &Dbg,
        diagnosis: &Diagnosis,
        kewd: &DiagKeywd,
        value: Status,
        tx: &Sender<Point>,
    ) {
        match diagnosis.get(kewd).map(|p| p.next(value)) {
            Some(point) => {
                if let Some(point) = point {
                    log::debug!("{}.yield_diagnosis | Send diagnostic '{}' ", dbg, kewd);
                    if let Err(err) = tx.send(point) {
                        log::warn!("{}.yield_status | Send diagnostic error: {}", dbg, err);
                    }
                }
            }
            None => log::debug!("{}.yield_diagnosis | Send diagnostic '{}' - not configured", dbg, kewd),
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
    fn read(&self, tx_send: Sender<Point>, connection_notify: ConnectionNotify) -> Result<(), Error> {
        log::info!("{}.read | starting...", self.dbg);
        let dbg = self.dbg.clone();
        let txid = self.txid;
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let diagnosis = self.diagnosis.clone();
        log::info!("{}.read | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let log_connected = ChangeNotify::builder(&dbg, Status::Invalid)
                .on(Status::Ok,  |message| log::info!("{message}"))
                .on(Status::Invalid, |message| log::warn!("{message}"))
                .build();
            let mut dbs = FxIndexMap::default();
            for (db_name, db_conf) in conf.dbs {
                log::info!("{}.read | configuring DB: {:?}...", dbg, db_name);
                let db = ProfinetDb::new(&dbg, txid, &db_conf);
                dbs.insert(db_name.clone(), db);
                log::info!("{}.read | configuring DB: {:?} - ok", dbg, db_name);
            }
            let mut cycle = ServiceCycle::new(&dbg, conf.cycle);
            let mut client = S7Client::new(dbg.clone(), conf.ip.clone());
            'main: while !exit.load(Ordering::Acquire) {
                match client.connect() {
                    Ok(_) => {
                        log_connected.add(Status::Ok, format!("{dbg}.read | Connection established"));
                        connection_notify.add(Status::Ok, dbg.clone());
                        'read: while !exit.load(Ordering::Acquire) {
                            cycle.start();
                            for (db_name, db) in &mut dbs {
                                log::trace!("{}.read | DB '{}' - reading...", dbg, db_name);
                                match db.read(&client, &tx_send) {
                                    Ok(_) => {
                                        db.errors.reset();
                                        log::trace!("{dbg}.read | DB '{db_name}' - reading - ok");
                                    }
                                    Err(err) => {
                                        _ = db.errors.add();
                                        if log::max_level() >= log::Level::Debug {
                                            log::warn!("{dbg}.read | DB '{db_name}' - reading - error: {:?}", err);
                                        }
                                        if !client.is_connected() {
                                            break 'read;
                                        }
                                    }
                                }
                                if exit.load(Ordering::Acquire) {
                                    break 'main;
                                }
                            }
                            if dbs.iter().all(|(_, db)| db.errors.errors() >= db.errors.limit()) {
                                break 'read;
                            }
                            cycle.wait();
                        }
                        log_connected.add(Status::Invalid, format!("{dbg}.read | Connection lost"));
                        connection_notify.add(Status::Invalid, dbg.clone());
                        Self::yield_status(&dbg, &mut dbs, &tx_send);
                        if let Err(err) = client.close() {
                            log::error!("{}.read | {:?}", dbg, err);
                        };
                    }
                    Err(err) => {
                        connection_notify.add(Status::Invalid, dbg.clone());
                        log_connected.add(Status::Invalid, format!("{dbg}.read | Disconnected: {:?}", err));
                        if log::max_level() >= log::Level::Trace {
                            log::warn!("{}.read | Connection error: {:?}", dbg, err);
                        }
                        thread::sleep(conf.reconnect_cycle);
                    }
                }
            }
            connection_notify.add(Status::Invalid, dbg.clone());
            Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
            // Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
            log::info!("{}.read | Exit", dbg);
            Ok(())
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.read | Started", self.dbg);
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => Err(Error::new(&self.dbg, "read").pass_with("Start failed", err)),
        }
    }
    ///
    /// Writes Point to the protocol (PROFINET device) specific address
    fn write(&self, tx_send: Sender<Point>, connection_notify: ConnectionNotify) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let txid = self.txid;
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let diagnosis = self.diagnosis.clone();
        log::info!("{}.write | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let log_connected = ChangeNotify::builder(&dbg, false)
                .on(true,  |message| log::info!("{}", message))
                .on(false, |message| log::warn!("{}", message))
                .build();
            let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
            let mut points: Vec<PointConf> = vec![];
            for (db_name, db_conf) in conf.dbs {
                log::info!("{}.write | configuring ProfinetDb: {:?}...", dbg, db_name);
                let db = ProfinetDb::new(&dbg, txid, &db_conf);
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
            'main: while !exit.load(Ordering::Acquire) {
                let mut errors_limit = ErrorLimit::new(3);
                thread::sleep(Duration::from_millis(100));    // Подождем по read подключит client
                match client.connect() {
                    Ok(_) => {
                        log_connected.add(true, format!("{}.write | Connection established", dbg));
                        connection_notify.add(Status::Ok, dbg.clone());
                        'write: while !exit.load(Ordering::Acquire) {
                            match rx_recv.recv_timeout(RECV_TIMEOUT) {
                                Ok(point) => {
                                    let point_name = point.name();
                                    let point_value = point.value();
                                    match point_name.split('/').nth(3) {
                                        Some(db_name) => {
                                            log::debug!("{}.write | ProfinetDb '{}' - writing point '{}'\t({:?})...", dbg, db_name, point_name, point_value);
                                            match dbs.get_mut(db_name) {
                                                Some(db) => {
                                                    match db.write(&client, point.clone()) {
                                                        Ok(_) => {
                                                            errors_limit.reset();
                                                            log::debug!("{}.write | ProfinetDb '{}' - writing point '{}'\t({:?}) - ok", dbg, db_name, point_name, point_value);
                                                            let reply = Self::reply_point(txid, point);
                                                            match tx_send.send(reply.clone()) {
                                                                Ok(_) => log::debug!("{}.write | ProfinetDb '{}' - sent reply: {:#?}", dbg, db_name, reply),
                                                                Err(err) => {
                                                                    log::error!("{}.write | Error sending to queue: {:?}", dbg, err);
                                                                    break 'main;
                                                                }
                                                            };
                                                        }
                                                        Err(err) => {
                                                            log::warn!("{}.write | ProfinetDb '{}' - write - error: {:?}", dbg, db_name, err);
                                                            if errors_limit.add().is_err() {
                                                                log::error!("{}.write | ProfinetDb '{}' - exceeded writing errors limit, trying to reconnect...", dbg, db_name);
                                                                if let Err(err) = tx_send.send(Point::String(PointHlr::new(
                                                                    txid,
                                                                    &point_name,
                                                                    format!("Write error: {}", err),
                                                                    Status::Ok,
                                                                    Cot::ActErr,
                                                                    chrono::offset::Utc::now(),
                                                                ))) {
                                                                    log::error!("{}.write | Error sending to queue: {:?}", dbg, err);
                                                                    break 'main;
                                                                };
                                                                if let Err(err) = client.close() {
                                                                    log::error!("{}.write | {:?}", dbg, err);
                                                                };
                                                                break 'write;
                                                            }
                                                        }
                                                    }
                                                }
                                                None => log::error!("{dbg}.write | ProfinetDb '{db_name}' - not found"),
                                            };
                                        }
                                        None => log::error!("{dbg}.write | ProfinetDb '{point_name}' - wrong name, expected like '/App/Ied/Signal'"),
                                    }
                                }
                                Err(err) => {
                                    match err {
                                        RecvTimeoutError::Timeout => {}
                                        _ => {
                                            log::error!("{}.write | Error receiving from queue: {:?}", dbg, err);
                                            break 'main;
                                        }
                                    }
                                }
                            }
                            if exit.load(Ordering::Acquire) {
                                break 'main;
                            }
                        }
                        connection_notify.add(Status::Invalid, dbg.clone());
                    }
                    Err(err) => {
                        connection_notify.add(Status::Invalid, dbg.clone());
                        log_connected.add(false, format!("{}.write | Disconnected: {:?}", dbg, err));
                        log::trace!("{}.write | Connection error: {:?}", dbg, err);
                        thread::sleep(conf.reconnect_cycle);
                    }
                }
            }
            connection_notify.add(Status::Invalid, dbg.clone());
            Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
            log::info!("{}.write | Exit", dbg);
            Ok(())
        });
        log::info!("{}.write | Started", self.dbg);
        match handle {
            Ok(handle) => {
                log::info!("{}.write | Started", self.dbg);
                self.handles.push(handle);
                Ok(())
            }
            Err(err) => Err(Error::new(&self.dbg, "read").pass_with("Start failed", err)),
        }
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
            Point::Bytes(point) => {
                Point::Bytes(PointHlr::new(
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
        let connection_notify: ConnectionNotify = {
            let tx_send1 = tx_send.clone();
            let tx_send2 = tx_send.clone();
            let diagnosis1 = self.diagnosis.clone();
            let diagnosis2 = self.diagnosis.clone();
            Arc::new(ChangeNotify::builder(&self.dbg, Status::Obsolete)
                .on(Status::Ok,  move |dbg| Self::yield_diagnosis(&dbg, &diagnosis1, &DiagKeywd::Connection, Status::Ok, &tx_send1))
                .on(Status::Invalid, move |dbg| Self::yield_diagnosis(&dbg, &diagnosis2, &DiagKeywd::Connection, Status::Invalid, &tx_send2))
                .build())
        };
        connection_notify.add(Status::Invalid, self.dbg.clone());
        Self::yield_diagnosis(&self.dbg, &self.diagnosis, &DiagKeywd::Status, Status::Ok, &tx_send);
        let handle_read = self.read(tx_send.clone(), connection_notify.clone());
        let handle_write = self.write(tx_send, connection_notify);
        log::info!("{}.run | started", self.dbg);
        let error = Error::new(&self.dbg, "run");
        match (handle_read, handle_write) {
            (Ok(_), Ok(_)) => {
                Ok(())
            }
            (Ok(_), Err(err)) => {
                self.exit();
                if let Err(err) = self.handles.wait() {
                    log::error!("{}.run | Error: {:?}", self.dbg, err);
                }
                Err(error.pass_with("Error starting inner thread 'read'", err.to_string()))
            }
            (Err(err), Ok(_)) => {
                self.exit();
                if let Err(err) = self.handles.wait() {
                    log::error!("{}.run | Error: {:?}", self.dbg, err);
                }
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
        self.exit.store(true, Ordering::Release);
    }
}
