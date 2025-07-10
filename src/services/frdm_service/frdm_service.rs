//!
//! # FRDM (Fiber Rope Defects Monitoring)
//! 
//! - Communication with Camera 
//! - Receives current rope position
//! - Scanning the rope for defects
//! - Calculates Rope Depreciation Rate
//! 
//! ## Basic configuration parameters:
//! 
//! ```yaml
//! service FrdmService FrdmService1:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
//! 
use std::{hash::BuildHasherDefault, net::{SocketAddr, UdpSocket}, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::{Error, ErrorLimit}};
use sal_sync::{
    collections::FxIndexMap, kernel::state::{ChangeNotify, Switch, SwitchCondition, SwitchState},
    services::{entity::{Name, Object, PointTxId}, Service, ServiceCycle, Services}, sync::Handles, thread_pool::Scheduler,
};
use crate::{
    conf::udp_client_config::udp_client_config::UdpClientConfig,
    core_::RwLock,
};
///
/// FRDM Service (Fiber Rope Defects Monitoring)
/// 
pub struct FrdmService {
    tx_id: usize,
    name: Name,
    conf: UdpClientConfig,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl FrdmService {
    /// Message starts with
    pub const SYN: u8 = 22;
    /// Start message ends with
    pub const EOT: u8 = 4;
    /// Header length in bytes
    pub const HEAD_LEN: usize = 7;
    //
    /// Crteates new instance of the FrdmService 
    pub fn new(conf: UdpClientConfig, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            tx_id,
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            scheduler,
            handles: Handles::new(&dbg),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
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
                                &[FrdmService::SYN, FrdmService::EOT] | &[FrdmService::SYN, FrdmService::EOT, ..] => {
                                    log::debug!("{}.handshake | {}: Start message ACK received", dbg, src_addr);
                                    return Ok((socket, src_addr, buf[2..].to_vec()))
                                    // switch_state.add(State::Read);
                                }
                                // Unexpected Data message received, but Start message expected
                                &[FrdmService::SYN, _addr, _type_, _c1,_c2,_c3, _c4, ..] => {
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
impl Object for FrdmService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for FrdmService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrdmService")
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
impl Service for FrdmService {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let tx_id = self.tx_id;
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let services = self.services.clone();
        log::debug!("{}.run | Preparing thread...", dbg);
        *SELF_ID.write() = dbg.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
                (NotifyState::Start,          Box::new(|message| log::info!("{}", message))),
                (NotifyState::Exit,           Box::new(|message| log::info!("{}", message))),
                (NotifyState::UdpBindError,   Box::new(|message| log::error!("{}", message))),
                (NotifyState::UdpRecvError,   Box::new(|message| log::error!("{}", message))),
            ]);

            let send = services
                .get_link(&conf.send_to)
                .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
            let mut reconnect = ServiceCycle::new(dbg, conf.reconnect);
            'main: loop {
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
                reconnect.wait();
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            }
            Ok(())
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
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
        self.exit.store(true, Ordering::SeqCst);
    }    
}
