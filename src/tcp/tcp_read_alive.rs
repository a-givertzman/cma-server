use log::{info, warn, LevelFilter};
use std::{
    io::BufReader, net::TcpStream, 
    sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, Mutex}, 
    thread::{self, JoinHandle}, time::Duration,
};
use crate::{core_::{
    net::connection_status::ConnectionStatus, point::point::Point
}, services::{safe_lock::SafeLock, task::service_cycle::ServiceCycle}, tcp::tcp_stream_write::OpResult};
use super::steam_read::TcpStreamRead;

///
/// Transfering points from JdsStream (socket) to the Channel Sender<PointType>
#[derive(Debug)]
pub struct TcpReadAlive {
    id: String,
    stream_read: Arc<Mutex<dyn TcpStreamRead>>,
    send: Sender<Point>,
    cycle: Option<Duration>,
    exit: Arc<AtomicBool>,
    exit_pair: Arc<AtomicBool>,
}
impl TcpReadAlive {
    ///
    /// Creates new instance of [TcpReadAlive]
    /// - [parent] - the ID if the parent entity
    /// - [exit] - notification from parent to exit 
    /// - [exitPair] - notification from / to sibling pair to exit 
    pub fn new(
        parent: impl Into<String>, 
        stream_read: Arc<Mutex<dyn TcpStreamRead>>,
        dest: Sender<Point>, 
        cycle: Option<Duration>, 
        exit: Option<Arc<AtomicBool>>, 
        exit_pair: Option<Arc<AtomicBool>>
    ) -> Self {
        let self_id = format!("{}/TcpReadAlive", parent.into());
        Self {
            id: self_id.clone(),
            stream_read,
            send: dest,
            cycle,
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
            exit_pair: exit_pair.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    /// Main loop of the [TcpReadAlive]
    pub fn run(&mut self, tcp_stream: TcpStream) -> JoinHandle<()> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let exit_pair = self.exit_pair.clone();
        let mut cycle = self.cycle.map(|cycle| ServiceCycle::new(&self_id, cycle));
        let send = self.send.clone();
        let tcp_stream_read = self.stream_read.clone();
        info!("{}.run | Preparing thread...", self.id);
        let handle = thread::Builder::new().name(format!("{} - Read", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            let mut tcp_stream = BufReader::new(tcp_stream);
            let mut tcp_stream_read = tcp_stream_read.slock(&self_id);
            info!("{}.run | Main loop started", self_id);
            loop {
                if let Some(cycle) = &mut cycle {cycle.start()}
                match tcp_stream_read.read(&mut tcp_stream) {
                    ConnectionStatus::Active(point) => {
                        match point {
                            OpResult::Ok(point) => {
                                // debug!("{}.run | read point: {:?}", self_id, point);
                                match send.send(point) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        warn!("{}.run | write to queue error: {:?}", self_id, err);
                                    }
                                };
                                if let Some(cycle) = &mut cycle {cycle.wait()}
                            }
                            OpResult::Err(err) => {
                                if log::max_level() == LevelFilter::Trace {
                                    warn!("{}.run | error: {:?}", self_id, err);
                                }
                                if let Some(cycle) = &mut cycle {cycle.wait()}
                            }
                            OpResult::Timeout() => {}
                        }
                    }
                    ConnectionStatus::Closed(err) => {
                        warn!("{}.run | error: {:?}", self_id, err);
                        exit_pair.store(true, Ordering::SeqCst);
                        break;
                    }
                };
                if exit.load(Ordering::SeqCst) | exit_pair.load(Ordering::SeqCst) {
                    break;
                }
            }
            info!("{}.run | Exit", self_id);
        }).unwrap();
        info!("{}.run | started", self.id);
        handle
    }
    ///
    /// Sends exit event into the main loop
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}
