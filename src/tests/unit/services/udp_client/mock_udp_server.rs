//!
//! Service implements kind of bihavior
//! Basic configuration parameters:
//! ```yaml
//! service MockUdpServer Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
use std::{net::UdpSocket, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, RwLock}, thread, time::Duration};
use sal_sync::services::{entity::{name::Name, object::Object, point::point::Point}, service::{service::Service, service_cycle::ServiceCycle, service_handles::ServiceHandles}};
use crate::{
    // conf::tcp_server_config::MockUdpServerConfig,
    core_::{failure::errors_limit::ErrorLimit, state::change_notify::ChangeNotify}, services::{services::Services, udp_client::udp_client::UdpClient} 
};
///
/// 
#[derive(Clone)]
pub struct MockUdpServerConfig {
    pub name: Name,
    pub local_addr: String,
    pub channel: u8,
    /// `Values<i16>` in the DATA field of the single UDP message, not bytes
    pub count: usize,
    /// Maximum Transmission Unit, default 1500, [Resolve IPv4 Fragmentation, MTU...](https://www.cisco.com/c/en/us/support/docs/ip/generic-routing-encapsulation-gre/25885-pmtud-ipfrag.html)
    pub mtu: usize,
    pub cycle: Duration,
}
///
/// Do something ...
pub struct MockUdpServer {
    id: String,
    name: Name,
    conf: MockUdpServerConfig,
    services: Arc<RwLock<Services>>,
    test_data: Vec<i16>,
    exit: Arc<AtomicBool>,
}
//
//
impl MockUdpServer {
    //
    /// Crteates new instance of the MockUdpServer 
    pub fn new(parent: impl Into<String>, conf: MockUdpServerConfig, services: Arc<RwLock<Services>>, test_data: &[i16]) -> Self {
        Self {
            id: format!("{}/MockUdpServer({})", parent.into(), conf.name),
            name: conf.name.clone(),
            conf: conf.clone(),
            services,
            test_data: test_data.into(),
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
        self.name.clone()
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
    UdpRecvError,
    UdpSendError,
    WouldBlock,
    TimedOut,
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
        let mut cycle = ServiceCycle::new(&self_id, conf.cycle);
        let test_data = self.test_data.clone();
        let exit = self.exit.clone();
        log::info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            let self_id = &self_id;
            let mut notify: ChangeNotify<_, String> = ChangeNotify::new(self_id, State::Start, vec![
                (State::Start,          Box::new(|message| log::info!("{}", message))),
                (State::Exit,           Box::new(|message| log::info!("{}", message))),
                (State::UdpBindError,   Box::new(|message| log::error!("{}", message))),
                (State::UdpRecvError,   Box::new(|message| log::error!("{}", message))),
                (State::UdpSendError,   Box::new(|message| log::error!("{}", message))),
                (State::WouldBlock,   Box::new(|message| log::error!("{}", message))),
                (State::TimedOut,   Box::new(|message| log::error!("{}", message))),
            ]);
            let local_addr =  conf.local_addr;
            'main: loop {
                match UdpSocket::bind(&local_addr) {
                    Ok(socket) => {
                        let mut buf = vec![0; conf.mtu];
                        let mut error_limit = ErrorLimit::new(3);
                        if let Err(err) = socket.set_read_timeout(Some(Duration::from_millis(100))) {
                            log::error!("{}.run | Socket Set timeout error: {:?}", self_id, err);
                        }
                        'read: loop {
                            match socket.recv_from(&mut buf) {
                                Ok((_, src_addr)) => {
                                    error_limit.reset();
                                    match buf.as_slice() {
                                        // Empty message
                                        &[] => log::debug!("{}.run | {}: Empty message received", self_id, src_addr),
                                        // Start of communication
                                        &[UdpClient::SYN, UdpClient::EOT] | &[UdpClient::SYN, UdpClient::EOT, ..] => {
                                            log::debug!("{}.run | {}: Start message received", self_id, src_addr);
                                            match socket.send_to(&[UdpClient::SYN, UdpClient::EOT], src_addr) {
                                                Ok(_) => log::debug!("{}.run | {}: Start message ACK sent", self_id, src_addr),
                                                Err(err) => {
                                                    log::error!("{}.run | Send ACK to {}: error: {:#?}", self_id, src_addr, err);
                                                    // notify.add(State::UdpSendError, format!("{}.run | UdpSocket recv error: {:#?}", self_id, err))                                                        
                                                },
                                            }
                                            let message_windows = test_data.chunks(conf.count);
                                            for message in message_windows {
                                                cycle.start();
                                                log::trace!("{}.run | words: \n\t{:?}", self_id, message);
                                                let mut buf = vec![UdpClient::SYN, conf.channel, 16];
                                                buf.extend(((conf.count * 2) as u32).to_be_bytes());
                                                // let data = unsafe { message.align_to::<u8>() };
                                                // let data = message.into_iter().map(|v| {v.to_be_bytes()});
                                                buf.extend(message.into_iter().flat_map(|v| {v.to_be_bytes()}));
                                                // for b in message {
                                                //     buf.extend(b.to_be_bytes());
                                                // }
                                                match socket.send_to(&buf, src_addr) {
                                                    Ok(sent_len) => {
                                                        log::trace!("{}.run | Sent to {}: data ({}): \n\t{:?}", self_id, src_addr, sent_len, buf);
                                                    }
                                                    Err(err) => {
                                                        // log::error!("{}.run | Send to {}: error: {:#?}", self_id, src_addr, err);
                                                        notify.add(State::UdpSendError, format!("{}.run | Socket send error: {:#?}", self_id, err))                                                        
                                                    }
                                                }
                                                if !cycle.interval().is_zero() {
                                                    cycle.wait();
                                                }
                                            }
                                            log::info!("{}.run | Sent: {} values of <i16>", self_id, test_data.len());
                                            while !exit.load(Ordering::SeqCst) {
                                                thread::sleep(Duration::from_millis(100));
                                            }
                                            break 'main;
                                        }
                                        _ => log::warn!("{}.run | {}: Unknown message format: {:?}...", self_id, src_addr, &buf[..=10]),
                                    }
                                }
                                Err(err) => {
                                    // notify.add(State::UdpRecvError, format!("{}.run | UdpSocket recv error: {:#?}", self_id, err)),
                                    match err.kind() {
                                        std::io::ErrorKind::WouldBlock => {
                                            notify.add(State::WouldBlock, format!("{}.run | Socket read timeout", self_id));
                                        },
                                        std::io::ErrorKind::TimedOut => {
                                            notify.add(State::TimedOut, format!("{}.run | Socket read timeout", self_id));
                                        }
                                        _ => {
                                            let message = format!("{}.run | Read error: {:#?}", self_id, err);
                                            log::debug!("{}", message);
                                            if error_limit.add().is_err() {
                                                log::error!("{}.run | Socket read errors limit exceeded, trying to reconnect...", self_id);
                                                break 'read;
                                            }
                                        },
                                    }
                                }
                            }
                            if exit.load(Ordering::SeqCst) {
                                break 'main;
                            }
                        }
                    }
                    Err(err) => notify.add(State::UdpBindError, format!("{}.run | UdpSocket::bind error: {:#?}", self_id, err)),
                }
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
            log::info!("{}.run | Exit", self_id);
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