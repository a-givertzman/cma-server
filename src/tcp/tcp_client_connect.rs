use sal_sync::services::service::service_cycle::ServiceCycle;
use std::{net::{SocketAddr, TcpStream, ToSocketAddrs}, sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock}, thread, time::Duration};
use log::{warn, LevelFilter, debug, info};
use crate::services::safe_lock::rwlock::SafeLock;
///
/// Opens a TCP connection to a remote host
/// - returns connected Result<TcpStream, Err>
pub struct TcpClientConnect {
    id: String,
    addr: SocketAddr,
    stream: Arc<RwLock<Vec<TcpStream>>>,
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
            id: format!("{}/TcpClientConnect", parent.into()),
            addr,
            stream: Arc::new(RwLock::new(Vec::new())),
            reconnect,
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    /// Opens a TCP connection to a remote host until succeed.
    pub fn connect(&mut self) -> Option<TcpStream> {
        let self_id = self.id.clone();
        info!("{}.connect | connecting...", self_id);
        let id = self.id.clone();
        let addr = self.addr;
        info!("{}.connect | connecting to: {:?}...", id, addr);
        let cycle = self.reconnect;
        let self_stream = self.stream.clone();
        let exit = self.exit.clone();
        let handle = thread::spawn(move || {
            let mut cycle = ServiceCycle::new(&self_id, cycle);
            loop {
                cycle.start();
                match TcpStream::connect_timeout(&addr, Duration::from_millis(1000)) {
                    Ok(stream) => {
                        self_stream.wlock(&self_id).push(stream);
                        info!("{}.connect | connected to: \n\t{:?}", id, self_stream.rlock(&self_id).first().unwrap());
                        break;
                    }
                    Err(err) => {
                        if log::max_level() == LevelFilter::Debug {
                            warn!("{}.connect | connection error: \n\t{:?}", id, err);
                        }
                    }
                };
                if exit.load(Ordering::SeqCst) {
                    debug!("{}.connect | Exit: 'true'", id);
                    break;
                }
                cycle.wait();
            }
            debug!("{}.connect | Exit", id);
        });
        handle.join().unwrap();
        let mut tcp_stream = self.stream.wlock(&self.id);
        tcp_stream.pop()
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