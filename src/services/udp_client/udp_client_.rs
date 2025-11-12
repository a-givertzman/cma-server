//!
//! # Communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol
//! 
//! Default port number 15180
//! 
//! ## Message structure
//! 
//!     |Field name:   | FUN | ADDR | TYPE | COUNT | DATA        |
//!     |---           | --- | ---- | ---- | ----- | ----        |
//!     |Data type:    | u8  | u8   | u8   | u32   | [T; COUNT]    | 
//!     |Example value:| 22  | 0    | 16   | 512  | [u16; 512] |
//!     
//!     - `FUN` Functional byte, 
//!         - `0x22` - Initialization message
//!         - `0x02` - Data message
//!         - `0x05` - Command message
//!         - `0x07` - Error message
//!     - `ADDR` = 0...255 - Index of the input channel (0 - first input channel)
//!     - `TYPE` - type of values in the array in `DATA` field
//!         - 8 - u8, 1 byte unsigned integer value
//!         - 9 - i8, 1 byte signed integer value
//!         - 16 - u16, 2 byte unsigned integer value
//!         - 17 - i16, 2 byte signed integer value
//!         - 32 - u32, 4 byte unsigned integer value
//!         - 33 - i32, 4 byte signed integer value
//!         - 132 - f32, 4 bytes float value
//!     - `COUNT` - length of the array in the `DATA` field, number of values of type specified in the `TYPE` field
//!     - `DATA` - array of values of type specified in the `TYPE` field
//! 
//! ## Error codes
//!     `0x01` - System error
//!     `0x02` - ADC Error
//!     `0x03` - DMA Error
//!     `0x04` - Network error
//!     `...` - To be extended if necessary
//! 
//! ## Basic configuration parameters:
//! 
//! ```yaml
//! service UdpClient Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```

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
use std::{hash::BuildHasherDefault, net::{SocketAddr, UdpSocket}, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::{Error, ErrorLimit}};
use sal_sync::{
    collections::FxIndexMap, kernel::state::{ChangeNotify, Switch, SwitchCondition, SwitchState},
    services::{entity::{Name, Object, PointTxId}, Service, ServiceCycle, Services}, sync::Handles, thread_pool::Scheduler,
};
use crate::{
    domain::RwLock, services::udp_client::{UdpClientConf, UdpClientConnect}
};
use super::udp_client::UdpClientUnit;
///
/// 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Units {
    Data,
    Unknown(String),
}
///
/// Communication with Vibro-analytics microcontroller (Sub MC) over udp simple protocol
pub struct UdpClient {
    tx_id: usize,
    dbg: Dbg,
    name: Name,
    conf: UdpClientConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
//
impl UdpClient {
    /// Message starts with
    pub const SYN: u8 = 0x22;
    /// Start message ends with
    pub const EOT: u8 = 0x04;
    pub const DAT: u8 = 0x02;
    pub const CMD: u8 = 0x05;
    pub const ERR: u8 = 0x07;
    /// Message header length in bytes
    pub const HEAD_LEN: usize = 7;
    //
    /// Crteates new instance of the [UdpClient] 
    pub fn new(conf: UdpClientConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            tx_id,
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    /// Returns UdpClint's Units
    pub fn build_units<'a>(dbg: &Dbg, tx_id: usize, conf: &UdpClientConf) -> FxIndexMap<Units, UdpClientUnit<'a>> {
        let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
        for (db_name, db_conf) in &conf.points {
            log::info!("{}.build_dbs | Configuring UdpClientDb: {:?}...", dbg, db_name);
            let db = UdpClientUnit::new(dbg, tx_id, &db_conf, conf.mtu);
            if db_name.ends_with("data") {
                dbs.insert(Units::Data, db);
            } else {
                dbs.insert(Units::Data, db);
                log::error!("{}.build_dbs | Unknown kind of DB '{}' in Configuring: {:#?} - ok", dbg, db_name, conf);
            }
            log::info!("{}.build_dbs | Configuring UdpClientDb: {:?} - ok", dbg, db_name);
        }
        dbs
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
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let tx_id = self.tx_id;
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let services = self.services.clone();
        log::info!("{}.run | Preparing thread...", dbg);
        *SELF_ID.write() = dbg.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
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
                                    log::info!("{}.run | State: {:?}", SELF_ID.read(), value);
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
                                    log::info!("{}.run | State: {:?}", SELF_ID.read(), value);
                                    value == State::Offline
                                }),
                                target: State::Offline,
                            },
                            SwitchCondition {
                                condition: Box::new(|value| {
                                    log::info!("{}.run | State: {:?}", SELF_ID.read(), value);
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
                                    log::info!("{}.run | State: {:?}", SELF_ID.read(), value);
                                    value == State::Offline
                                }),
                                target: State::Offline,
                            },
                            SwitchCondition {
                                condition: Box::new(|value| {
                                    log::info!("{}.run | State: {:?}", SELF_ID.read(), value);
                                    value == State::Start
                                }),
                                target: State::Start,
                            },
                        ],
                    },
                ],
            );
            let mut dbs = Self::build_units(dbg, tx_id, &conf);
            let send = services
                .get_link(&conf.send_to)
                .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
            let mut reconnect = ServiceCycle::new(dbg, conf.reconnect);
            let udp_connect = UdpClientConnect::new(parent, conf);
            'main: loop {
                reconnect.start();
                match Self::connect(dbg, &conf, exit.clone()) {
                    Ok((socket, remote_addr, bytes)) => {
                        let mut bytes = bytes;
                        'read: loop {
                            let mut error_limit = ErrorLimit::new(3);
                            match dbs.get_mut(&Units::Data) {
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
                                    log::error!("{}.run | UdpClientDb '{:?}' - Not found", dbg, Units::Data);
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
