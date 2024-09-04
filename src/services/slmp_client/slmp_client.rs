use std::{fmt::Debug, net::TcpStream, sync::{atomic::{AtomicBool, AtomicU32, Ordering}, mpsc::Sender, Arc, Mutex, RwLock}, thread, time::Duration};
use log::{debug, error, info, warn};
use sal_sync::{
    collections::map::IndexMapFxHasher,
    services::{
        entity::{name::Name, object::Object, point::{point::Point, point_config::PointConfig, point_tx_id::PointTxId}, status::status::Status},
        service::{service::Service, service_handles::ServiceHandles},
    },
};
use testing::stuff::wait::WaitTread;
use crate::{
    conf::{diag_keywd::DiagKeywd, slmp_client_config::slmp_client_config::SlmpClientConfig},
    core_::{
        constants::constants::RECV_TIMEOUT, state::exit_notify::ExitNotify,
    },
    services::{
        diagnosis::diag_point::DiagPoint, safe_lock::rwlock::SafeLock,
        services::Services, slmp_client::{slmp_read::SlmpRead, slmp_write::SlmpWrite},
    },
    tcp::tcp_client_connect::TcpClientConnect,
     
};
///
/// - Connects to the SLMP device (FX5 Eth module)
/// - Cyclically reads adressess from the SLMP device and yields changed to the MultiQueue
/// - Writes Point to the protocol (SLMP device) specific address
pub struct SlmpClient {
    tx_id: usize,
    id: String,
    name: Name,
    conf: SlmpClientConfig,
    services: Arc<RwLock<Services>>,
    diagnosis: Arc<Mutex<IndexMapFxHasher<DiagKeywd, DiagPoint>>>,
    exit: Arc<AtomicBool>,
}
//
// 
impl SlmpClient {
    ///
    /// Creates new instance of [ApiClient]
    /// - [parent] - the ID if the parent entity
    pub fn new(conf: SlmpClientConfig, services: Arc<RwLock<Services>>) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        let diagnosis = Arc::new(Mutex::new(conf.diagnosis.iter().map(|(keywd, conf)| {
            (keywd.to_owned(), DiagPoint::new(tx_id, conf.clone()))
        }).collect()));
        Self {
            tx_id,
            id: conf.name.join(),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            diagnosis,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Sends diagnosis point
    fn yield_diagnosis(
        self_id: &str,
        diagnosis: &Arc<Mutex<IndexMapFxHasher<DiagKeywd, DiagPoint>>>,
        kewd: &DiagKeywd,
        value: Status,
        dest: &Sender<Point>,
    ) {
        match diagnosis.lock() {
            Ok(mut diagnosis) => {
                match diagnosis.get_mut(kewd) {
                    Some(point) => {
                        debug!("{}.yield_diagnosis | Sending diagnosis point '{}' ", self_id, kewd);
                        if let Some(point) = point.next(value) {
                            if let Err(err) = dest.send(point) {
                                warn!("{}.yield_status | Send error: {}", self_id, err);
                            }
                        }
                    }
                    None => debug!("{}.yield_diagnosis | Diagnosis point '{}' - not configured", self_id, kewd),
                }
            }
            Err(err) => error!("{}.yield_diagnosis | Diagnosis lock error: {:#?}", self_id, err),
        }
    }
    ///
    /// Applies a write / read timeout for TcpStream
    fn set_stream_timout(self_id: &str, stream: &TcpStream, read_timeout: Duration, write_timeout: Option<Duration>) {
        match stream.set_read_timeout(Some(read_timeout)) {
            Ok(_) => {
                info!("{}.set_stream_timout | Socket set read timeout {:?} - ok", self_id, read_timeout);
            }
            Err(err) => {
                warn!("{}.set_stream_timout | Socket set read timeout error {:?}", self_id, err);
            }
        }
        if let Some(timeout) = write_timeout {
            match stream.set_write_timeout(Some(timeout)) {
                Ok(_) => {
                    info!("{}.set_stream_timout | Socket set write timeout {:?} - ok", self_id, timeout);
                }
                Err(err) => {
                    warn!("{}.set_stream_timout | Socket set write timeout error {:?}", self_id, err);
                }
            }
        }
    }
}
//
//
impl Object for SlmpClient {
    fn id(&self) -> &str {
        &self.id
    }
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
            .field("id", &self.id)
            .finish()
    }
}
//
//
unsafe impl Send for SlmpClient {}
unsafe impl Sync for SlmpClient {}
//
// 
impl Service for SlmpClient {
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let conf = self.conf.clone();
        let diagnosis = self.diagnosis.clone();
        let status = Arc::new(AtomicU32::new(Status::Ok.into()));
        let exit = Arc::new(ExitNotify::new(&self_id, Some(self.exit.clone()), None));
        let tx_send = self.services.rlock(&self_id).get_link(&conf.send_to).unwrap_or_else(|err| {
            panic!("{}.run | services.get_link error: {:#?}", self.id, err);
        });
        let mut tcp_client_connect = TcpClientConnect::new(
            self_id.clone(), 
            format!("{}:{}", conf.ip, conf.port),
            conf.reconnect_cycle,
            Some(self.exit.clone()),
        );
        let mut slmp_read = SlmpRead::new(
            &self_id,
            self.tx_id,
            // self.name.clone(),
            conf.clone(),
            tx_send.clone(),
            // diagnosis.clone(),
            status.clone(),
            exit.clone(),
        );
        let mut slmp_write = SlmpWrite::new(
            &self_id,
            self.tx_id,
            // self.name.clone(),
            conf.clone(),
            tx_send.clone(),
            // diagnosis.clone(),
            self.services.clone(),
            status,
            exit.clone(),
        );
        Self::yield_diagnosis(&self.id, &diagnosis, &DiagKeywd::Status, Status::Ok, &tx_send);
        Self::yield_diagnosis(&self.id, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            loop {
                info!("{}.run | Connecting...", self_id);
                exit.reset_pair();
                match tcp_client_connect.connect() {
                    Some(tcp_stream) =>  {
                        Self::set_stream_timout(
                            &self_id,
                            &tcp_stream,
                            conf.cycle.map_or(RECV_TIMEOUT, |cycle| cycle),
                            Some(RECV_TIMEOUT),
                        );
                        Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Connection, Status::Ok, &tx_send);
                        // info!("{}.run | Connecting...", self_id);
                        let h_r = slmp_read.run(tcp_stream.try_clone().unwrap());
                        let h_w = slmp_write.run(tcp_stream);
                        match (h_r, h_w) {
                            (Ok(h_r), Ok(h_w)) => {
                                h_r.wait().unwrap();
                                h_w.wait().unwrap();
                            },
                            (Ok(h_r), Err(_)) => {
                                Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
                                exit.exit_pair();
                                h_r.wait().unwrap();
                            },
                            (Err(_), Ok(h_w)) => {
                                Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
                                exit.exit_pair();
                                h_w.wait().unwrap();
                            }
                            (Err(_), Err(_)) => {
                                Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Status, Status::Invalid, &tx_send);
                                exit.exit_pair();
                            }
                        }
                        info!("{}.run | All thrad exited...", self_id);
                    }
                    None => {
                        Self::yield_diagnosis(&self_id, &diagnosis, &DiagKeywd::Connection, Status::Invalid, &tx_send);
                    }
                }
                if exit.get_parent() {
                    break;
                }
                info!("{}.run | Sleeping {:?}...", self_id, conf.reconnect_cycle);
                thread::sleep(conf.reconnect_cycle);
                warn!("{}.run | TcpClient connection failed - trying to reconnect...", self_id);
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
    fn points(&self) -> Vec<PointConfig> {
        self.conf.points()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
