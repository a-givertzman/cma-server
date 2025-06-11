use std::{fmt::Debug, net::TcpStream, sync::{atomic::{AtomicBool, AtomicU32, Ordering}, Arc}, thread::{self, JoinHandle}, time::Duration};
use coco::Stack;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    collections::FxIndexMap, kernel::state::ExitNotify, services::{
        conf::DiagKeywd, entity::{Name, Object, Point, PointConfig, PointTxId, Status},
        Service,
        Services,
    }, sync::channel::Sender, thread_pool::Scheduler
};
use crate::{
    conf::slmp_client_config::slmp_client_config::SlmpClientConfig,
    core_::{constants::constants::RECV_TIMEOUT, Mutex},
    services::{
        diagnosis::diag_point::DiagPoint,
        slmp_client::{slmp_read::SlmpRead, slmp_write::SlmpWrite},
    },
    tcp::tcp_client_connect::TcpClientConnect,
     
};
///
/// - Connects to the SLMP device (FX5 Eth module)
/// - Cyclically reads adressess from the SLMP device and yields changed to the MultiQueue
/// - Writes Point to the protocol (SLMP device) specific address
pub struct SlmpClient {
    tx_id: usize,
    dbg: Dbg,
    name: Name,
    conf: SlmpClientConfig,
    services: Arc<Services>,
    diagnosis: Arc<Mutex<FxIndexMap<DiagKeywd, DiagPoint>>>,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
}
//
// 
impl SlmpClient {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: SlmpClientConfig, services: Arc<Services>, schrduler: Scheduler) -> Self {
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
        dest: &Sender<Point>,
    ) {
        match diagnosis.lock().get_mut(kewd) {
            Some(point) => {
                log::debug!("{}.yield_diagnosis | Sending diagnosis point '{}' ", dbg, kewd);
                if let Some(point) = point.next(value) {
                    if let Err(err) = dest.send(point) {
                        log::warn!("{}.yield_status | Send error: {}", dbg, err);
                    }
                }
            }
            None => log::debug!("{}.yield_diagnosis | Diagnosis point '{}' - not configured", dbg, kewd),
        }
    }
    ///
    /// Applies a write / read timeout for TcpStream
    fn set_stream_timout(dbg: &Dbg, stream: &TcpStream, read_timeout: Duration, write_timeout: Option<Duration>) {
        match stream.set_read_timeout(Some(read_timeout)) {
            Ok(_) => {
                log::info!("{}.set_stream_timout | Socket set read timeout {:?} - ok", dbg, read_timeout);
            }
            Err(err) => {
                log::warn!("{}.set_stream_timout | Socket set read timeout error {:?}", dbg, err);
            }
        }
        if let Some(timeout) = write_timeout {
            match stream.set_write_timeout(Some(timeout)) {
                Ok(_) => {
                    log::info!("{}.set_stream_timout | Socket set write timeout {:?} - ok", dbg, timeout);
                }
                Err(err) => {
                    log::warn!("{}.set_stream_timout | Socket set write timeout error {:?}", dbg, err);
                }
            }
        }
    }
}
//
//
impl Object for SlmpClient {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for SlmpClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SlmpClient")
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for SlmpClient {
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let tx_id = self.tx_id;
        let conf = self.conf.clone();
        let services = self.services.clone();
        let diagnosis = self.diagnosis.clone();
        let status = Arc::new(AtomicU32::new(Status::Ok.into()));
        let exit = Arc::new(ExitNotify::new(&dbg, Some(self.exit.clone()), None));
        let tx_send = self.services.get_link(&conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.dbg, err);
        });
        let mut tcp_client_connect = TcpClientConnect::new(
            dbg.clone(), 
            format!("{}:{}", conf.ip, conf.port),
            conf.reconnect_cycle,
            Some(self.exit.clone()),
        );
        log::info!("{}.run | Preparing thread...", dbg);
        let handle = thread::Builder::new().name(format!("{}.run", dbg.clone())).spawn(move || {
            log::info!("{}.run | Preparing thread - ok", dbg);
            let mut slmp_read = SlmpRead::new(
                &dbg,
                tx_id,
                // self.name.clone(),
                conf.clone(),
                tx_send.clone(),
                // diagnosis.clone(),
                status.clone(),
                exit.clone(),
            );
            let mut slmp_write = SlmpWrite::new(
                &dbg,
                tx_id,
                // self.name.clone(),
                conf.clone(),
                tx_send.clone(),
                // diagnosis.clone(),
                services.clone(),
                status,
                exit.clone(),
            );
            Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Status, Status::Ok, &tx_send);
            Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
                loop {
                log::info!("{}.run | Connecting...", dbg);
                exit.reset_pair();
                match tcp_client_connect.connect() {
                    Some(tcp_stream) =>  {
                        Self::set_stream_timout(
                            &dbg,
                            &tcp_stream,
                            conf.cycle.map_or(RECV_TIMEOUT, |cycle| cycle),
                            Some(RECV_TIMEOUT),
                        );
                        Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Connection, Status::Ok, &tx_send);
                        // info!("{}.run | Connecting...", self_id);
                        let h_r = slmp_read.run(tcp_stream.try_clone().unwrap());
                        let h_w = slmp_write.run(tcp_stream);
                        match (h_r, h_w) {
                            (Ok(h_r), Ok(h_w)) => {
                                h_r.join().unwrap();
                                h_w.join().unwrap();
                            },
                            (Ok(h_r), Err(_)) => {
                                Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
                                exit.exit_pair();
                                h_r.join().unwrap();
                            },
                            (Err(_), Ok(h_w)) => {
                                Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
                                exit.exit_pair();
                                h_w.join().unwrap();
                            }
                            (Err(_), Err(_)) => {
                                Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
                                exit.exit_pair();
                            }
                        }
                        log::info!("{}.run | All thrad exited...", dbg);
                    }
                    None => {
                        Self::yield_diagnosis(&dbg, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
                    }
                }
                if exit.get_parent() {
                    break;
                }
                log::info!("{}.run | Sleeping {:?}...", dbg, conf.reconnect_cycle);
                thread::sleep(conf.reconnect_cycle);
                log::warn!("{}.run | TcpClient connection failed - trying to reconnect...", dbg);
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
    fn points(&self) -> Vec<PointConfig> {
        self.conf.points()
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
