//!
//! # Communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol
//! 
//! ## Basic configuration parameters:
//! 
//! ```yaml
//! service UdpClient Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
//! 
//! ## UDP protocol description
//! 
//! Default port number 15180
//! 
//! Message in the UDP has fallowing fiels
//! 
//! |Field name:   | SYN | ADDR | TYPE | COUNT | DATA        |
//! |---           | --- | ---- | ---- | ----- | ----        |
//! |Data type:    | u8  | u8   | u8   | u32   | u8[1024]    | 
//! |Example value:| 22  | 0    | 16   | 1024  | [u16; 1024] |
//! - `SYN` = 22 - message starts with
//! - `ADDR` = 0...255 - an address of the input channel (0 - first input channel)
//! - `TYPE` - type of values in the array in `DATA` field
//!     - 8 - 1 byte integer value
//!     - 16 - 2 byte float value
//!     - 32 - u16[1024] an array of 2 byte values of length 512
//! - `COUNT` - length of the array in the `DATA` field
//! - `DATA` - array of values of type specified in the `TYPE` field
//! 
use std::{hash::BuildHasherDefault, net::UdpSocket, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, Mutex, RwLock}, thread, time::Duration};
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use log::{info, warn};
use sal_sync::{collections::map::IndexMapFxHasher, services::{entity::{name::Name, object::Object, point::{point::Point, point_tx_id::PointTxId}}, service::{link_name::LinkName, service::Service, service_handles::ServiceHandles}}};
use crate::{
    conf::udp_client_config::udp_client_config::UdpClientConfig, core_::{failure::errors_limit::ErrorLimit, state::change_notify::ChangeNotify}, services::{safe_lock::SafeLock, services::Services}
};
use super::udp_client_db::UdpClientDb;
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Dbs {
    Data,
    Unknown(String),
}
///
/// Do something ...
pub struct UdpClient {
    tx_id: usize,
    id: String,
    name: Name,
    conf: UdpClientConfig,
    services: Arc<RwLock<Services>>,
    exit: Arc<AtomicBool>,
}
//
//
impl UdpClient {
    /// Message starts with
    pub const SYN: u8 = 22;
    /// Start message ends with
    pub const EOT: u8 = 4;
    /// Header length in bytes
    pub const HEAD_LEN: usize = 7;
    //
    /// Crteates new instance of the UdpClient 
    pub fn new(conf: UdpClientConfig, services: Arc<RwLock<Services>>) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        Self {
            tx_id,
            id: conf.name.join(),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    ///
    pub fn build_dbs(self_id: &str, tx_id: usize, conf: &UdpClientConfig) -> IndexMapFxHasher<Dbs, UdpClientDb> {
        let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
        for (db_name, db_conf) in &conf.dbs {
            log::info!("{}.build_dbs | Configuring UdpClientDb: {:?}...", self_id, db_name);
            let db = UdpClientDb::new(self_id, tx_id, &db_conf);
            if db_name.ends_with("data") {
                dbs.insert(Dbs::Data, db);
            } else {
                dbs.insert(Dbs::Data, db);
                log::error!("{}.build_dbs | Unknown kind of DB '{}' in Configuring: {:#?} - ok", self_id, db_name, conf);
            }
            log::info!("{}.build_dbs | Configuring UdpClientDb: {:?} - ok", self_id, db_name);
        }
        dbs
    }
}
//
//
impl Object for UdpClient {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for UdpClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UdpClient")
            .field("id", &self.id)
            .finish()
    }
}
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum State {
    Start,
    Exit,
    UdpBindError,
    UdpRecvError,
}
//
//
unsafe impl Send for UdpClient {}
unsafe impl Sync for UdpClient {}
//
// 
impl Service for UdpClient {
    //
    // 
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let tx_id = self.tx_id;
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let services = self.services.clone();
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
            let self_id = &self_id;
            let mut notify: ChangeNotify<_, String> = ChangeNotify::new(self_id, State::Start, vec![
                (State::Start,          Box::new(|message| log::info!("{}", message))),
                (State::Exit,           Box::new(|message| log::info!("{}", message))),
                (State::UdpBindError,   Box::new(|message| log::error!("{}", message))),
                (State::UdpRecvError,   Box::new(|message| log::error!("{}", message))),
            ]);
            let mut dbs = Self::build_dbs(self_id, tx_id, &conf);
            let mtu = 4096;
            let mut buf = vec![0; mtu];
            let mut count: usize;
            let send = services.rlock(self_id)
                .get_link(&conf.send_to)
                .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", self_id, conf.send_to, err));
            'main: loop {
                match UdpSocket::bind(&conf.local_addr) {
                    Ok(socket) => {
                        match socket.send_to(&[Self::SYN, Self::EOT], &conf.remote_addr) {
                            Ok(_) => {
                                log::debug!("{}.run | Start message sent to'{}'", self_id, conf.remote_addr);
                                let mut error_limit = ErrorLimit::new(3);
                                loop {
                                    match socket.recv_from(&mut buf) {
                                        Ok((_, src_addr)) => {
                                            match buf.as_slice() {
                                                // Empty message received
                                                &[] => {
                                                    log::debug!("{}.run | {}: Empty message received", self_id, src_addr);
                                                }
                                                // Start ACK received
                                                &[UdpClient::SYN, UdpClient::EOT, ..] => {
                                                    log::debug!("{}.run | {}: Start message ACK received", self_id, src_addr);
                                                }
                                                // Data message received
                                                &[UdpClient::SYN, addr, type_, c1,c2,c3, c4, ..] => {
                                                    count = u32::from_be_bytes([c1, c2, c3, c4]) as usize;
                                                    match dbs.get_mut(&Dbs::Data) {
                                                        Some(db_data) => {
                                                            match db_data.read(&socket, &send) {
                                                                Ok(_) => {
                                                                    error_limit.reset();
                                                                    log::trace!("{}.read | UdpClientDb '{}' - reading - ok", self_id, db_data.name);
                                                                }
                                                                Err(err) => {
                                                                    warn!("{}.read | UdpClientDb '{}' - reading - error: {:?}", self_id, db_data.name, err);
                                                                    if error_limit.add().is_err() {
                                                                        log::error!("{}.read | UdpClientDb '{}' - exceeded reading errors limit, trying to reconnect...", self_id, db_data.name);
                                                                        break 'main;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        None => {},
                                                    }
                                                }
                                                _ => {
                                                    log::warn!("{}.run | {}: Unknown message format: {:#?}...", self_id, src_addr, &buf[..=10]);
                                                }
                                            }
                                        }
                                        Err(err) => notify.add(State::UdpRecvError, format!("{}.run | UdpSocket recv error: {:#?}", self_id, err)),
                                    }
                                    if exit.load(Ordering::SeqCst) {
                                        break 'main;
                                    }
                                }
                            }
                            Err(err) => {
                                log::debug!("{}.run | Start message to '{}' error {:#?}", self_id, conf.remote_addr, err);
                            }
                        }
                    }
                    Err(err) => notify.add(State::UdpBindError, format!("{}.run | UdpSocket::bind error: {:#?}", self_id, err)),
                }
                thread::sleep(conf.reconnect);
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            }
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
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }    
}
