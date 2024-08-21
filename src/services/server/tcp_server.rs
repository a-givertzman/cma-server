use log::{debug, info, warn};
use std::{
    fmt::Debug, net::{Shutdown, TcpListener, TcpStream}, sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc, Mutex, RwLock}, thread, time::Duration
};
use crate::{
    conf::{point_config::name::Name, tcp_server_config::TcpServerConfig},
    core_::{constants::constants::RECV_TIMEOUT, object::object::Object},
    services::{
        safe_lock::SafeLock, server::{
            connections::{Action, TcpServerConnections}, jds_cnnection::JdsConnection
        }, service::{service::Service, service_handles::ServiceHandles}, services::Services, task::service_cycle::ServiceCycle
    },
};
///
/// 
struct ConnectionInfo<'a> {
    self_id: &'a str,
    self_name: &'a Name,
    connection_id: &'a str,
}
//
// 
impl<'a> ConnectionInfo<'a> {
    pub fn new(self_id: &'a str, self_name: &'a Name, connection_id: &'a str) -> Self {
        Self {
            self_id,
            self_name,
            connection_id,
        }
    }
}
///
/// Bounds TCP socket server
/// Listening socket for incoming connections
/// Verified incoming connections handles in the separate thread
pub struct TcpServer {
    id: String,
    name: Name,
    conf: TcpServerConfig,
    connections: Arc<Mutex<TcpServerConnections>>,
    services: Arc<RwLock<Services>>,
    exit: Arc<AtomicBool>,
}
//
//
impl TcpServer {
    ///
    /// Creates new instance of the TcpServer:
    /// - parent - name of the parent entity, used to create self name: "/parent/self_id/"
    /// - filter - all trafic from server to client will be filtered by some criterias, until Subscribe request confirmed:
    ///    - cot - [Cot] - bit mask wich will be passed
    ///    - name - exact name wich passed
    pub fn new(conf: TcpServerConfig, services: Arc<RwLock<Services>>, ) -> Self {
        Self {
            id: conf.name.join(),
            name: conf.name.clone(),
            conf: conf.clone(),
            connections: Arc::new(Mutex::new(TcpServerConnections::new(conf.name))),
            services,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    ///                 self_id: &str, self_name: &Name, connection_id: &str
    fn setup_connection(con_info: ConnectionInfo, stream: TcpStream, services: Arc<RwLock<Services>>, conf: TcpServerConfig, exit: Arc<AtomicBool>, connections: Arc<Mutex<TcpServerConnections>>) {
        info!("{}.setup_connection | Trying to repair Connection '{}'...", con_info.self_id, con_info.connection_id);
        let repair_result = connections.slock(con_info.self_id).repair(con_info.connection_id, stream.try_clone().unwrap());
        match repair_result {
            Ok(_) => {
                info!("{}.setup_connection | Connection '{}' - reparied", con_info.self_id, con_info.connection_id);
            }
            Err(err) => {
                info!("{}.setup_connection | {}", con_info.self_id, err);
                info!("{}.setup_connection | New connection: '{}'", con_info.self_id, con_info.connection_id);
                let (send, recv) = mpsc::channel();
                let mut connection = JdsConnection::new(
                    con_info.self_id,
                    &Name::from(con_info.self_name.parent()),
                    con_info.connection_id,
                    recv, services.clone(),
                    conf.clone(),
                    exit.clone()
                );
                match connection.run() {
                    Ok(handles) => {
                        if handles.len() != 1 {
                            panic!("{}.setup_connection | TcpServerConnection.run must return single handle, but returns {}", con_info.self_id, handles.len())
                        }
                        let (_, handle) = handles.into_iter().next().unwrap();
                        match send.send(Action::Continue(stream)) {
                            Ok(_) => {}
                            Err(err) => {
                                warn!("{}.setup_connection | Send tcpStream error {:?}", con_info.self_id, err);
                            }
                        }
                        info!("{}.setup_connection | connections.lock...", con_info.self_id);
                        connections.slock(con_info.self_id).insert(
                            con_info.connection_id,
                            handle,
                            send,
                        );
                        info!("{}.setup_connection | connections.lock - ok", con_info.self_id);
                    }
                    Err(err) => {
                        warn!("{}.setup_connection | error: {:?}", con_info.self_id, err);
                    }
                };
                info!("{}.setup_connection | Connection '{}' - created new", con_info.self_id, con_info.connection_id);
            }
        }
    }
    ///
    ///
    fn set_stream_timout(self_id: &str, stream: &TcpStream, raad_timeout: Duration, write_timeout: Option<Duration>) {
        match stream.set_read_timeout(Some(raad_timeout)) {
            Ok(_) => {
                info!("{}.set_stream_timout | Socket set read timeout {:?} - ok", self_id, raad_timeout);
            }
            Err(err) => {
                warn!("{}.set_stream_timout | Socket set read timeout error {:?}", self_id, err);
            }
        }
        if let Some(timeout) = write_timeout {
            match stream.set_write_timeout(Some(timeout)) {
                Ok(_) => {
                    info!("{}.set_stream_timout | Socket set write timeout {:?} - ok", self_id, timeout);
                }
                Err(err) => {
                    warn!("{}.set_stream_timout | Socket set write timeout error {:?}", self_id, err);
                }
            }
        }
    }
    ///
    /// Chech if finished connection threads are present in the self.connection
    /// - removes finished connections
    fn clean(self_id: &str, connections: &Arc<Mutex<TcpServerConnections>>) {
        match connections.lock() {
            Ok(mut connections) => {
                connections.clean()
            }
            Err(err) => {
                warn!("{}.clean | Connections lock error {:?}", self_id, err);
            }
        }
    }
    
}
//
//
impl Object for TcpServer {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Debug for TcpServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TcpServer")
            .field("id", &self.id)
            .finish()
    }
}
//
//
impl Service for TcpServer {
    //
    //
    fn run(&mut self) -> Result<ServiceHandles, String> {
        info!("{}.run | Starting...", self.id);
        let self_id = self.id.clone();
        let self_name = self.name.clone();
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let connections = self.connections.clone();
        let services = self.services.clone();
        let reconnect_cycle = conf.reconnect_cycle.unwrap_or(Duration::ZERO);
        info!("{}.run | Preparing thread...", self_id);
        let handle = thread::Builder::new().name(format!("{}.run", self_id.clone())).spawn(move || {
            info!("{}.run | Preparing thread - ok", self_id);
            let mut cycle = ServiceCycle::new(&self_id, reconnect_cycle);
            'main: loop {
                cycle.start();
                info!("{}.run | Open socket {}...", self_id, conf.address);
                match TcpListener::bind(conf.address) {
                    Ok(listener) => {
                        info!("{}.run | Open socket {} - ok", self_id, conf.address);
                        for stream in listener.incoming() {
                            Self::clean(&self_id, &connections);
                            if exit.load(Ordering::SeqCst) {
                                debug!("{}.run | Detected exit", self_id);
                                break;
                            }
                            match stream {
                                Ok(stream) => {
                                    let connection_id = stream.peer_addr().map_or("Unknown remote IP".to_string(), |a| {a.ip().to_string()});
                                    Self::set_stream_timout(&self_id, &stream, RECV_TIMEOUT, None);
                                    info!("{}.run | Setting up Connection '{}'...", self_id, connection_id);
                                    Self::setup_connection(
                                        ConnectionInfo::new(&self_id, &self_name, &connection_id),
                                        stream,
                                        services.clone(),
                                        conf.clone(),
                                        exit.clone(),
                                        connections.clone(),
                                    );
                                }
                                Err(err) => {
                                    warn!("{}.run | error: {:?}", self_id, err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        warn!("{}.run | error: {:?}", self_id, err);
                    }
                };
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
                cycle.wait();
                if exit.load(Ordering::SeqCst) {
                    break 'main;
                }
            }
            info!("{}.run | Exit...", self_id);
            // Self::waitConnections(&self_id, connections);
            connections.slock(&self_id).wait();
            info!("{}.run | Exit", self_id);
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
    ///
    ///
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(10));
        info!("{}.exit | Final connection...", self.id);
        match TcpStream::connect_timeout(&self.conf.address, Duration::from_millis(100)) {
            Ok(stream) => {
                info!("{}.exit | Final connection - ok", self.id);
                stream.shutdown(Shutdown::Both).unwrap();
            }
            Err(err) => {
                info!("{}.exit | Final connection error: {:?}", self.id, err);
            }
        };
    }
}
