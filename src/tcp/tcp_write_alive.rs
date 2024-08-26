use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, thread::{JoinHandle, self}, net::TcpStream, time::Duration};
use log::{info, warn};
use sal_sync::services::service::service_cycle::ServiceCycle;
use crate::{
    core_::net::connection_status::ConnectionStatus, services::safe_lock::SafeLock, tcp::tcp_stream_write::{OpResult, TcpStreamWrite} 
};
///
/// Transfering points from Channel Sender<PointType> to the JdsStream (socket)
#[derive(Debug)]
pub struct TcpWriteAlive {
    id: String,
    cycle: Option<Duration>,
    stream_write: Arc<Mutex<TcpStreamWrite>>,
    exit: Arc<AtomicBool>,
    exit_pair: Arc<AtomicBool>,
}
//
// 
impl TcpWriteAlive {
    ///
    /// Creates new instance of [TcpWriteAlive]
    /// - [parent] - the ID if the parent entity
    /// - [exit] - notification from parent to exit 
    /// - [exitPair] - notification from / to sibling pair to exit 
    pub fn new(parent: impl Into<String>, cycle: Option<Duration>, stream_write: Arc<Mutex<TcpStreamWrite>>, exit: Option<Arc<AtomicBool>>, exit_pair: Option<Arc<AtomicBool>>) -> Self {
        Self {
            id: format!("{}/TcpWriteAlive", parent.into()),
            cycle,
            stream_write,
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
            exit_pair: exit_pair.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    /// 
    pub fn run(&self, tcp_stream: TcpStream) -> JoinHandle<()> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let exit_pair = self.exit_pair.clone();
        let mut cycle = self.cycle.map(|cycle| ServiceCycle::new(&self_id, cycle));
        let stream_write = self.stream_write.clone();
        info!("{}.run | Preparing thread...", self.id);
        let handle = thread::Builder::new().name(format!("{} - Write", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            let mut stream_write = stream_write.slock(&self_id);
            info!("{}.run | Main loop started", self_id);
            'main: loop {
                if let Some(cycle) = &mut cycle {cycle.start()}
                match stream_write.write(&tcp_stream) {
                    ConnectionStatus::Active(result) => {
                        match result {
                            OpResult::Ok(_) => {
                                if let Some(cycle) = &mut cycle {cycle.wait()}
                            }
                            OpResult::Err(err) => {
                                warn!("{}.run | error: {:?}", self_id, err);
                                if let Some(cycle) = &mut cycle {cycle.wait()}
                            }
                            OpResult::Timeout() => {}
                        }
                    }
                    ConnectionStatus::Closed(err) => {
                        warn!("{}.run | error: {:?}", self_id, err);
                        exit_pair.store(true, Ordering::SeqCst);
                        break 'main;
                    }
                };
                if exit.load(Ordering::SeqCst) | exit_pair.load(Ordering::SeqCst) {
                    break 'main;
                }
            }
            info!("{}.run | Exit", self_id);
        }).unwrap();
        info!("{}.run | started", self.id);
        handle
    }
    ///
    /// 
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
