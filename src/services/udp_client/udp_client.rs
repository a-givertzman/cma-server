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
use std::{hash::BuildHasherDefault, net::{SocketAddr, UdpSocket}, sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock}, thread::{self, JoinHandle}, time::Duration};
use coco::Stack;
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    collections::FxIndexMap, kernel::state::{ChangeNotify, Switch, SwitchCondition, SwitchState},
    services::{entity::{Name, Object, PointTxId}, safe_lock::rwlock::SafeLock, service::{Service, ServiceCycle}, services::Services},
};
use crate::{
    conf::udp_client_config::udp_client_config::UdpClientConfig,
    core_::failure::errors_limit::ErrorLimit,
};
use super::udp_client_db::UdpClientDb;
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dbs {
    Data,
    Unknown(String),
}
///
/// Do something ...
pub struct UdpClient {
    tx_id: usize,
    dbg: Dbg,
    name: Name,
    conf: UdpClientConfig,
    services: Arc<RwLock<Services>>,
    handle: Stack<JoinHandle<()>>,
    is_finished: Arc<AtomicBool>,
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
            dbg: Dbg::new(conf.name.parent(), conf.name.me()),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            handle: Stack::new(),
            is_finished: Arc::new(AtomicBool::new(false)),
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns UdpClint's DB blokcs
    pub fn build_dbs(dbg: &Dbg, tx_id: usize, conf: &UdpClientConfig) -> FxIndexMap<Dbs, UdpClientDb> {
        let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
        for (db_name, db_conf) in &conf.dbs {
            log::info!("{}.build_dbs | Configuring UdpClientDb: {:?}...", dbg, db_name);
            let db = UdpClientDb::new(dbg, tx_id, &db_conf, conf.mtu);
            if db_name.ends_with("data") {
                dbs.insert(Dbs::Data, db);
            } else {
                dbs.insert(Dbs::Data, db);
                log::error!("{}.build_dbs | Unknown kind of DB '{}' in Configuring: {:#?} - ok", dbg, db_name, conf);
            }
            log::info!("{}.build_dbs | Configuring UdpClientDb: {:?} - ok", dbg, db_name);
        }
        dbs
    }
    ///
    /// Returns the socket ready to receive data messages
    /// - Connected to the remote address
    /// - Hanshaked - Start message sent and acknowledged
    fn handshake(dbg: &Dbg, socket: UdpSocket, conf: &UdpClientConfig, exit: Arc<AtomicBool>) -> Result<(UdpSocket, SocketAddr, Vec<u8>), String> {
        let mut buf = vec![0; conf.mtu];
        match socket.send_to(&[Self::SYN, Self::EOT], &conf.remote_addr) {
            Ok(_) => {
                log::debug!("{}.handshake | Start message sent to'{}'", dbg, conf.remote_addr);
                let mut error_limit = ErrorLimit::new(4);
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((_, src_addr)) => {
                            error_limit.reset();
                            match buf.as_slice() {
                                // Empty message received
                                &[] => {
                                    log::warn!("{}.handshake | {}: Empty message received", dbg, src_addr);
                                }
                                // Start ACK received
                                &[UdpClient::SYN, UdpClient::EOT] | &[UdpClient::SYN, UdpClient::EOT, ..] => {
                                    log::debug!("{}.handshake | {}: Start message ACK received", dbg, src_addr);
                                    return Ok((socket, src_addr, buf[2..].to_vec()))
                                    // switch_state.add(State::Read);
                                }
                                // Unexpected Data message received, but Start message expected
                                &[UdpClient::SYN, _addr, _type_, _c1,_c2,_c3, _c4, ..] => {
                                    log::warn!("{}.handshake | {}: Start message expected, but Data message received: {:#?}...", dbg, src_addr, &buf[..=10]);
                                }
                                // Unknown message received
                                _ => {
                                    log::warn!("{}.handshake | {}: Unknown message format: {:#?}...", dbg, src_addr, &buf[..=10]);
                                }
                            }
                        }
                        Err(err) => {
                            // notify.add(State::UdpRecvError, format!("{}.handshake | UdpSocket recv error: {:#?}", self_id, err)),
                            match err.kind() {
                                std::io::ErrorKind::WouldBlock => {
                                    let message = &format!("{}.handshake | Socket read timeout", dbg);
                                    log::debug!("{}", message);
                                },
                                std::io::ErrorKind::TimedOut => {
                                    let message = &format!("{}.handshake | Socket read timeout", dbg);
                                    log::debug!("{}", message);
                                }
                                _ => {
                                    let message = format!("{}.handshake | Read start message error: {:#?}", dbg, err);
                                    log::warn!("{}", message);
                                },
                            }
                            if error_limit.add().is_err() {
                                // switch_state.add(State::Offline);
                                return Err(format!("{}.handshake | Socket read errors limit exceeded, trying to reconnect...", dbg))
                            }
                        }
                    }
                    if exit.load(Ordering::SeqCst) {
                        return Err(format!("{}.handshake | Breaked by `exit` ", dbg))
                    }
                }
            }
            Err(err) => {
                // switch_state.add(State::Offline);
                Err(format!("{}.handshake | Start message to '{}' error {:#?}", dbg, conf.remote_addr, err))
            }
        }
    }
    ///
    /// Returns the socket ready to receive data messages
    /// - Connected to the remote address
    /// - Hanshaked - Start message sent and acknowledged
    fn connect(dbg: &Dbg, conf: &UdpClientConfig, exit: Arc<AtomicBool>) -> Result<(UdpSocket, SocketAddr, Vec<u8>), String> {
        match UdpSocket::bind(&conf.local_addr) {
            Ok(socket) => {
                loop {
                    match socket.connect(&conf.remote_addr) {
                        Ok(_) => {
                            if let Err(err) = socket.set_read_timeout(Some(Duration::from_millis(100))) {
                                log::error!("{}.connect | Socket Set timeout error: {:?}", dbg, err);
                            }
                            return Self::handshake(dbg, socket.try_clone().unwrap(), conf, exit)
                        }
                        Err(err) => {
                            log::error!("{}.connect | Connect error: {:?}", dbg, err);
                            // switch_state.add(State::Offline);
                            // Err(format!("{}.connect | Connect error: {:?}", self_id, err))
                        }
                    }
                    if exit.load(Ordering::SeqCst) {
                        return Err(format!("{}.connect | Breaked by `exit` ", dbg))
                    }
                }
            }
            Err(err) => {
                // notify.add(NotifyState::UdpBindError, format!("{}.connect | UdpSocket::bind error: {:#?}", self_id, err)),
                Err(format!("{}.connect | UdpSocket::bind error: {:#?}", dbg, err))
            }
        }        
    }
}
//
//
impl Object for UdpClient {
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
            .field("id", &self.dbg)
            .finish()
    }
}
///
/// Used for logging
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NotifyState {
    Start,
    Exit,
    UdpBindError,
    UdpRecvError,
}
///
/// Used for protocol states
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum State {
    Offline,
    Start,
    Read,
}
//
//
static SELF_ID: std::sync::LazyLock<RwLock<Dbg>> = std::sync::LazyLock::new(|| RwLock::new(Dbg::own("")));
//
// 
impl Service for UdpClient {
    //
    // 
    fn run(&mut self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let tx_id = self.tx_id;
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let services = self.services.clone();
        log::info!("{}.run | Preparing thread...", dbg);
        *SELF_ID.write().unwrap() = dbg.clone();
        let handle = thread::Builder::new().name(format!("{}.run", dbg)).spawn(move || {
            let dbg = &dbg;
            let mut notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
                (NotifyState::Start,          Box::new(|message| log::info!("{}", message))),
                (NotifyState::Exit,           Box::new(|message| log::info!("{}", message))),
                (NotifyState::UdpBindError,   Box::new(|message| log::error!("{}", message))),
                (NotifyState::UdpRecvError,   Box::new(|message| log::error!("{}", message))),
            ]);
            let mut switch_state: SwitchState<State, State> = SwitchState::new(
                State::Start,
                vec![
                    Switch{
                        state: State::Offline,
                        conditions: vec![
                            SwitchCondition {
                                condition: Box::new(|value| {
                                    log::info!("{}.run | State: {:?}", SELF_ID.read().unwrap(), value);
                                    value == State::Start
                                }),
                                target: State::Start,
                            },
                        ],
                    },
                    Switch{
                        state: State::Start,
                        conditions: vec![
                            SwitchCondition {
                                condition: Box::new(|value| {
                                    log::info!("{}.run | State: {:?}", SELF_ID.read().unwrap(), value);
                                    value == State::Offline
                                }),
                                target: State::Offline,
                            },
                            SwitchCondition {
                                condition: Box::new(|value| {
                                    log::info!("{}.run | State: {:?}", SELF_ID.read().unwrap(), value);
                                    value == State::Read
                                }),
                                target: State::Read,
                            },
                        ],
                    },
                    Switch{
                        state: State::Read,
                        conditions: vec![
                            SwitchCondition {
                                condition: Box::new(|value| {
                                    log::info!("{}.run | State: {:?}", SELF_ID.read().unwrap(), value);
                                    value == State::Offline
                                }),
                                target: State::Offline,
                            },
                            SwitchCondition {
                                condition: Box::new(|value| {
                                    log::info!("{}.run | State: {:?}", SELF_ID.read().unwrap(), value);
                                    value == State::Start
                                }),
                                target: State::Start,
                            },
                        ],
                    },
                ],
            );
            let mut dbs = Self::build_dbs(dbg, tx_id, &conf);
            let send = services.rlock(dbg)
                .get_link(&conf.send_to)
                .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
            let mut reconnect = ServiceCycle::new(dbg, conf.reconnect);
            'main: loop {
                reconnect.start();
                match Self::connect(dbg, &conf, exit.clone()) {
                    Ok((socket, remote_addr, bytes)) => {
                        let mut bytes = bytes;
                        'read: loop {
                            let mut error_limit = ErrorLimit::new(3);
                            match dbs.get_mut(&Dbs::Data) {
                                Some(db_data) => {
                                    match db_data.read(&socket, bytes, &send) {
                                        Ok(_) => {
                                            error_limit.reset();
                                            log::trace!("{}.run | UdpClientDb '{}' - reading from '{}' - ok", dbg, db_data.name, remote_addr);
                                        }
                                        Err(err) => {
                                            log::warn!("{}.run | UdpClientDb '{}' - reading from '{}' - error: {:?}", dbg, db_data.name, remote_addr, err);
                                            if error_limit.add().is_err() {
                                                log::error!("{}.run | UdpClientDb '{}' - exceeded reading errors limit, trying to reconnect...", dbg, db_data.name);
                                                switch_state.add(State::Start);
                                                break 'read;
                                            }
                                        }
                                    }
                                }
                                None => {
                                    log::error!("{}.run | UdpClientDb '{:?}' - Not found", dbg, Dbs::Data);
                                },
                            }
                            bytes = vec![];
                            if exit.load(Ordering::SeqCst) {
                                break 'main;
                            }
                        }                    }
                    Err(err) => {
                        log::error!("{}.run | Error: {:?}", dbg, err);
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
                reconnect.wait();
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            }
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
