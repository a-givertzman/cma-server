//!
//! Service implements kind of bihavior
//! Basic configuration parameters:
//! ```yaml
//! service MockUdpServer Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
use std::{net::UdpSocket, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, RwLock}, thread};
use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_handles::ServiceHandles}};
use crate::{
    // conf::tcp_server_config::MockUdpServerConfig,
    core_::state::change_notify::ChangeNotify, services::{services::Services, udp_client::udp_client::UdpClient} 
};
///
/// 
#[derive(Clone)]
pub struct MockUdpServerConfig {
    pub name: Name,
    pub local_addr: String,
}
///
/// Do something ...
pub struct MockUdpServer {
    id: String,
    conf: MockUdpServerConfig,
    services: Arc<RwLock<Services>>,
    exit: Arc<AtomicBool>,
}
//
//
impl MockUdpServer {
    //
    /// Crteates new instance of the MockUdpServer 
    pub fn new(parent: impl Into<String>, conf: MockUdpServerConfig, services: Arc<RwLock<Services>>) -> Self {
        Self {
            id: format!("{}/MockUdpServer({})", parent.into(), conf.name),
            conf: conf.clone(),
            services,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
}
//
//
impl Object for MockUdpServer {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        todo!()
    }
}
//
// 
impl std::fmt::Debug for MockUdpServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockUdpServer")
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
    UdpRecvdError,
}
//
// 
impl Service for MockUdpServer {
    //
    // 
    fn get_link(&mut self, name: &str) -> Sender<Point> {
        panic!("{}.get_link | Does not support get_link", self.id())
        // match self.rxSend.get(name) {
        //     Some(send) => send.clone(),
        //     None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        // }
    }
    //
    //
    fn run(&mut self) -> Result<ServiceHandles<()>, String> {
        log::info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        log::info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            let self_id = &self_id;
            let mut notify: ChangeNotify<_, String> = ChangeNotify::new(self_id, State::Start, vec![
                (State::Start,          Box::new(|message| log::info!("{}", message))),
                (State::Exit,           Box::new(|message| log::info!("{}", message))),
                (State::UdpBindError,   Box::new(|message| log::info!("{}", message))),
            ]);
            let local_addr =  conf.local_addr;
            loop {
                match UdpSocket::bind(&local_addr) {
                    Ok(socket) => {
                        let mtu = 4096;
                        let mut buf = vec![0; mtu];
                        let mut count: u32;
                        loop {
                            match socket.recv_from(&mut buf) {
                                Ok((len, src_addr)) => {
                                    // Start of communication
                                    match buf.as_slice() {
                                        &[] => {
                                            log::debug!("{}.run | {}: Empty message", self_id, src_addr);
                                        }
                                        &[UdpClient::SYN, UdpClient::EOT] => {
                                            log::debug!("{}.run | {}: Start message", self_id, src_addr);
                                        }
                                        &[UdpClient::SYN, addr, type_, c1,c2,c3, c4, ..] => {
                                            count = u32::from_be_bytes([c1, c2, c3, c4]);
                                            log::debug!("{}.run | {}: addr: {} type: {} count: {}", self_id, src_addr, addr, type_, count);
                                            match &buf[4..(4 + count as usize)].try_into() {
                                                Ok(data) => {
                                                    let data: &Vec<u8> = data;
                                                }
                                                Err(_) => todo!(),
                                            }
                                        }
                                        _ => {
                                            log::debug!("{}.run | {}: Unknown message format: {:#?}...", self_id, src_addr, &buf[..=10]);
                                        }
                                    }
                                    if buf[0] == UdpClient::SYN && buf[1] == UdpClient::EOT {

                                    }
                                }
                                Err(err) => notify.add(State::UdpRecvdError, format!("{}.run | UdpSocket recv error: {:#?}", self_id, err)),
                            }
                        }
                    }
                    Err(err) => notify.add(State::UdpBindError, format!("{}.run | UdpSocket::bind error: {:#?}", self_id, err)),
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.id);
                Ok(ServiceHandles::new(vec![(self.id.clone(), handle)]))
            }
            Err(err) => {
                let message = format!("{}.run | Start failed: {:#?}", self.id, err);
                log::warn!("{}", message);
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