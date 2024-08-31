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
use std::{net::UdpSocket, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, Mutex, RwLock}, thread, time::Duration};
use log::{info, warn};
use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_handles::ServiceHandles}};
use crate::{
    conf::udp_client_config::udp_client_config::UdpClientConfig, core_::state::change_notify::ChangeNotify, services::services::Services
};
///
/// Do something ...
pub struct UdpClient {
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
    pub fn new(parent: impl Into<String>, conf: UdpClientConfig, services: Arc<RwLock<Services>>) -> Self {
        Self {
            id: conf.name.join(),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            exit: Arc::new(AtomicBool::new(false)),
        }
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
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id)).spawn(move || {
            let self_id = &self_id;
            let mut notify: ChangeNotify<_, String> = ChangeNotify::new(self_id, State::Start, vec![
                (State::Start,          Box::new(|message| log::info!("{}", message))),
                (State::Exit,           Box::new(|message| log::info!("{}", message))),
                (State::UdpBindError,   Box::new(|message| log::error!("{}", message))),
                (State::UdpRecvError,   Box::new(|message| log::error!("{}", message))),
            ]);
            let mtu = 4096;
            let mut buf = vec![0; mtu];
            let mut count: usize;
            'main: loop {
                match UdpSocket::bind(&conf.local_addr) {
                    Ok(socket) => {
                        match socket.send_to(&[Self::SYN, Self::EOT], &conf.remote_addr) {
                            Ok(_) => {
                                log::debug!("{}.run | Start message sent to'{}'", self_id, conf.remote_addr);
                                loop {
                                    match socket.recv_from(&mut buf) {
                                        Ok((_, src_addr)) => {
                                            match buf.as_slice() {
                                                // Empty message
                                                &[] => {
                                                    log::debug!("{}.run | {}: Empty message", self_id, src_addr);
                                                }
                                                // Start ACK
                                                &[UdpClient::SYN, UdpClient::EOT, ..] => {
                                                    log::debug!("{}.run | {}: Start message", self_id, src_addr);
                                                }
                                                // Data message
                                                &[UdpClient::SYN, addr, type_, c1,c2,c3, c4, ..] => {
                                                    count = u32::from_be_bytes([c1, c2, c3, c4]) as usize;
                                                    log::debug!("{}.run | {}: addr: {} type: {} count: {}", self_id, src_addr, addr, type_, count);
                                                    match &buf[Self::HEAD_LEN..(Self::HEAD_LEN + count)].try_into() {
                                                        Ok(data) => {
                                                            let data: &Vec<u8> = data;
                                                            log::debug!("{}.run | {}: data: {:?}", self_id, src_addr, data);
                                                        }
                                                        Err(err) => {
                                                            log::error!("{}.run | {}: Error message length: {}, expected {}, \n\t error: {:#?}", self_id, src_addr, buf.len(), Self::HEAD_LEN + count, err);
                                                        }
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
