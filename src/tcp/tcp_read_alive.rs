use concat_string::concat_string;
use log::{warn, info, LevelFilter};
use std::{
    io::{BufReader, Read}, net::TcpStream, 
    sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, Mutex}, 
    thread::{self, JoinHandle}, time::Duration,
};
use crate::{core_::{
    net::{connection_status::ConnectionStatus, protocols::jds::{jds_decode_message::JdsDecodeMessage, jds_deserialize::JdsDeserialize}}, object::object::Object, point::point_type::PointType
}, services::task::service_cycle::ServiceCycle};

use super::steam_read::{StreamRead, TcpStreamRead};

///
/// Transfering points from JdsStream (socket) to the Channel Sender<PointType>
pub struct TcpReadAlive {
    id: String,
    src_stream: Arc<Mutex<dyn TcpStreamRead>>,
    // src_stream: Arc<Mutex<impl StreamRead>>,
    send: Sender<PointType>,
    cycle: Duration,
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
        src_stream: Arc<Mutex<dyn TcpStreamRead>>,
        dest: Sender<PointType>, 
        cycle: Duration, 
        exit: Option<Arc<AtomicBool>>, 
        exit_pair: Option<Arc<AtomicBool>>
    ) -> Self {
        let self_id = format!("{}/TcpReadAlive", parent.into());
        Self {
            id: self_id.clone(),
            src_stream,
            send: dest,
            cycle,
            exit: exit.unwrap_or(Arc::new(AtomicBool::new(false))),
            exit_pair: exit_pair.unwrap_or(Arc::new(AtomicBool::new(false))),
        }
    }
    ///
    /// Main loop of the [TcpReadAlive]
    pub fn run(&mut self, tcp_stream: TcpStream) -> JoinHandle<()> {
        info!("{}.run | starting...", self.id);
        let self_id = self.id.clone();
        let exit = self.exit.clone();
        let exit_pair = self.exit_pair.clone();
        let mut cycle = ServiceCycle::new(self.cycle);
        let send = self.send.clone();
        let jds_stream = self.src_stream.clone();
        info!("{}.run | Preparing thread...", self.id);
        let handle = thread::Builder::new().name(format!("{} - Read", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            let mut tcp_stream = BufReader::new(tcp_stream);
            let mut jds_stream = jds_stream.lock().unwrap();
            info!("{}.run | Main loop started", self_id);
            loop {
                cycle.start();
                match jds_stream.read(&mut tcp_stream) {
                    ConnectionStatus::Active(point) => {
                        match point {
                            Ok(point) => {
                                match send.send(point) {
                                    Ok(_) => {},
                                    Err(err) => {
                                        warn!("{}.run | write to queue error: {:?}", self_id, err);
                                    },
                                };
                            },
                            Err(err) => {
                                if log::max_level() == LevelFilter::Trace {
                                    warn!("{}.run | error: {:?}", self_id, err);
                                }
                            },
                        }
                    },
                    ConnectionStatus::Closed(err) => {
                        warn!("{}.run | error: {:?}", self_id, err);
                        exit_pair.store(true, Ordering::SeqCst);
                        break;
                    },
                };
                if exit.load(Ordering::SeqCst) | exit_pair.load(Ordering::SeqCst) {
                    break;
                }
                cycle.wait();
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

type Rautes = fn(point: PointType) -> Option<PointType>;
pub struct Router {
    id: String,
    jds_stream: JdsDeserialize,
    rautes: Rautes,
}
///
/// 
impl Router {
    ///
    /// 
    pub fn new(parent: impl Into<String>, jds_stream: JdsDeserialize, rautes: Rautes) -> Self {
        let self_id = format!("{}/TcpReadAlive", parent.into());
        Self {
            id: self_id, 
            jds_stream,
            rautes,
        }
    }
}
///
/// 
impl Object for Router {
    fn id(&self) -> &str {
        &self.id
    }
}
///
/// 
impl TcpStreamRead for Router {
    ///
    /// Reads single point from source
    fn read(&mut self, tcp_stream: &mut BufReader<TcpStream>) -> ConnectionStatus<Result<PointType, String>, String> {
        match self.jds_stream.read(tcp_stream) {
            ConnectionStatus::Active(point) => {
                match point {
                    Ok(point) => {
                        match (self.rautes)(point) {
                            Some(point) => ConnectionStatus::Active(Ok(point)),
                            None => ConnectionStatus::Active(Err(concat_string!(self.id, ".read | Filtered by routes"))),
                        }
                    },
                    Err(err) => {
                        if log::max_level() == LevelFilter::Trace {
                            warn!("{}.read | error: {:?}", self.id, err);
                        }
                        ConnectionStatus::Active(Err(err))
                    },
                }
            },
            ConnectionStatus::Closed(err) => {
                warn!("{}.read | error: {:?}", self.id, err);
                ConnectionStatus::Closed(err)
            },
        }
    }
}