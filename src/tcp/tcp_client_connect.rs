use coco::Stack;
use sal_sync::services::{ServiceCycle};
use std::{net::{SocketAddr, TcpStream, ToSocketAddrs}, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use log::LevelFilter;
///
/// Opens a TCP connection to a remote host
/// - returns connected Result<TcpStream, Err>
pub struct TcpClientConnect {
    dbg: String,
    addr: SocketAddr,
    stream: Arc<Stack<TcpStream>>,
    reconnect: Duration,
    exit: Arc<AtomicBool>,
}
///
/// Opens a TCP connection to a remote host
impl TcpClientConnect {
    ///
    /// Creates a new instance of TcpClientConnect
    pub fn new(parent: impl Into<String>, addr: impl ToSocketAddrs + std::fmt::Debug, reconnect: Duration, exit: Option<Arc<AtomicBool>>) -> TcpClientConnect {
        let addr = match addr.to_socket_addrs() {
            Ok(mut addr_iter) => {
                match addr_iter.next() {
                    Some(addr) => addr,
                    None => panic!("TcpClientConnect({}).connect | Empty address found: {:?}", parent.into(), addr),
                }
            }
            Err(err) => panic!("TcpClientConnect({}).connect | Address parsing error: \n\t{:?}", parent.into(), err),
        };
        Self {
            dbg: format!("{}/TcpClientConnect", parent.into()),
            addr,
            stream: Arc::new(Stack::new()),
            reconnect,
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    /// Opens a TCP connection to a remote host until succeed.
    pub fn connect(&mut self) -> Option<TcpStream> {
        let dbg = self.dbg.clone();
        log::info!("{}.connect | connecting...", dbg);
        let id = self.dbg.clone();
        let addr = self.addr;
        log::info!("{}.connect | connecting to: {:?}...", id, addr);
        let cycle = self.reconnect;
        let mut result = None;
        let exit = self.exit.clone();
        let mut cycle = ServiceCycle::new(&dbg, cycle);
        loop {
            cycle.start();
            match TcpStream::connect_timeout(&addr, Duration::from_millis(1000)) {
                Ok(stream) => {
                    let stream_name = format!("{:?}", stream.peer_addr());
                    result = Some(stream);
                    log::info!("{}.connect | connected to: \n\t{:?}", id, stream_name);
                    break;
                }
                Err(err) => {
                    if log::max_level() == LevelFilter::Debug {
                        log::warn!("{}.connect | connection error: \n\t{:?}", id, err);
                    }
                }
            };
            if exit.load(Ordering::SeqCst) {
                log::debug!("{}.connect | Exit: 'true'", id);
                break;
            }
            cycle.wait();
        }
        result
    }
    // ///
    // /// Opens a TCP connection to a remote host with a timeout.
    // pub fn connect_timeout(&self, timeout: Duration) -> Result<TcpStream, std::io::Error> {
    //     TcpStream::connect_timeout(&self.addr, timeout)
    // }
    ///
    /// Exit thread
    #[allow(unused)]
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}