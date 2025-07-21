use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object}, Service, ServiceCycle, Services}, sync::{channel, Handles}, thread_pool::Scheduler};
use std::{
    fmt::Debug, net::{Shutdown, TcpListener, TcpStream}, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{self}, time::Duration
};
use crate::{
    conf::tcp_server_conf::TcpServerConf,
    domain::{constants::constants::RECV_TIMEOUT},
    services::server::{
        connections::{Action, TcpServerConnections}, jds_cnnection::JdsConnection
    },
};
///
/// 
struct ConnectionInfo<'a> {
    dbg: &'a Dbg,
    self_name: &'a Name,
    connection_id: &'a str,
}
//
// 
impl<'a> ConnectionInfo<'a> {
    pub fn new(dbg: &'a Dbg, self_name: &'a Name, connection_id: &'a str) -> Self {
        Self {
            dbg,
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
    dbg: Dbg,
    name: Name,
    conf: TcpServerConf,
    connections: Arc<TcpServerConnections>,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
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
    pub fn new(conf: TcpServerConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf: conf.clone(),
            connections: Arc::new(TcpServerConnections::new(conf.name)),
            services,
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
        }
    }
    ///
    ///                 self_id: &str, self_name: &Name, connection_id: &str
    fn setup_connection(con_info: ConnectionInfo, stream: TcpStream, services: Arc<Services>, conf: TcpServerConf, exit: Arc<AtomicBool>, connections: Arc<TcpServerConnections>, scheduler: Scheduler) {
        log::info!("{}.setup_connection | Trying to repair Connection '{}'...", con_info.dbg, con_info.connection_id);
        let repair_result = connections.repair(con_info.connection_id, stream.try_clone().unwrap());
        match repair_result {
            Ok(_) => {
                log::info!("{}.setup_connection | Connection '{}' - reparied", con_info.dbg, con_info.connection_id);
            }
            Err(err) => {
                log::info!("{}.setup_connection | {}", con_info.dbg, err);
                log::info!("{}.setup_connection | New connection: '{}'", con_info.dbg, con_info.connection_id);
                let (send, recv) = channel::unbounded();
                let connection = JdsConnection::new(
                    con_info.dbg,
                    &Name::from(con_info.self_name.parent()),
                    con_info.connection_id,
                    recv, services.clone(),
                    conf.clone(),
                    scheduler,
                    exit.clone()
                );
                match connection.run() {
                    Ok(_) => {
                        match send.send(Action::Continue(stream)) {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!("{}.setup_connection | Send tcpStream error {:?}", con_info.dbg, err);
                            }
                        }
                        log::info!("{}.setup_connection | connections.lock...", con_info.dbg);
                        connections.insert(
                            con_info.connection_id,
                            Arc::new(Box::new(connection)),
                            send,
                        );
                        log::info!("{}.setup_connection | connections.lock - ok", con_info.dbg);
                    }
                    Err(err) => {
                        log::warn!("{}.setup_connection | error: {:?}", con_info.dbg, err);
                    }
                };
                log::info!("{}.setup_connection | Connection '{}' - created new", con_info.dbg, con_info.connection_id);
            }
        }
    }
    ///
    ///
    fn set_stream_timout(dbg: &Dbg, stream: &TcpStream, raad_timeout: Duration, write_timeout: Option<Duration>) {
        match stream.set_read_timeout(Some(raad_timeout)) {
            Ok(_) => {
                log::info!("{}.set_stream_timout | Socket set read timeout {:?} - ok", dbg, raad_timeout);
            }
            Err(err) => {
                log::warn!("{}.set_stream_timout | Socket set read timeout error {:?}", dbg, err);
            }
        }
        if let Some(timeout) = write_timeout {
            match stream.set_write_timeout(Some(timeout)) {
                Ok(_) => {
                    log::info!("{}.set_stream_timout | Socket set write timeout {:?} - ok", dbg, timeout);
                }
                Err(err) => {
                    log::warn!("{}.set_stream_timout | Socket set write timeout error {:?}", dbg, err);
                }
            }
        }
    }
    ///
    /// Chech if finished connection threads are present in the self.connection
    /// - removes finished connections
    fn clean(_dbg: &Dbg, connections: &Arc<TcpServerConnections>) {
        connections.clean();
    }
    
}
//
//
impl Object for TcpServer {
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
            .field("id", &self.dbg)
            .finish()
    }
}
//
//
impl Service for TcpServer {
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let connections = self.connections.clone();
        let services = self.services.clone();
        let reconnect_cycle = conf.reconnect_cycle.unwrap_or(Duration::ZERO);
        let scheduler = self.scheduler.clone();
        log::info!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            log::info!("{}.run | Preparing thread - ok", dbg);
            let mut cycle = ServiceCycle::new(&dbg, reconnect_cycle);
            'main: loop {
                cycle.start();
                log::info!("{}.run | Open socket {}...", dbg, conf.address);
                match TcpListener::bind(conf.address) {
                    Ok(listener) => {
                        log::info!("{}.run | Open socket {} - ok", dbg, conf.address);
                        for stream in listener.incoming() {
                            Self::clean(&dbg, &connections);
                            if exit.load(Ordering::SeqCst) {
                                log::debug!("{}.run | Detected exit", dbg);
                                break;
                            }
                            match stream {
                                Ok(stream) => {
                                    let connection_id = stream.peer_addr().map_or("Unknown remote IP".to_string(), |a| {a.ip().to_string()});
                                    Self::set_stream_timout(&dbg, &stream, RECV_TIMEOUT, None);
                                    log::info!("{}.run | Setting up Connection '{}'...", dbg, connection_id);
                                    Self::setup_connection(
                                        ConnectionInfo::new(&dbg, &self_name, &connection_id),
                                        stream,
                                        services.clone(),
                                        conf.clone(),
                                        exit.clone(),
                                        connections.clone(),
                                        scheduler.clone(),
                                    );
                                }
                                Err(err) => {
                                    log::warn!("{}.run | error: {:?}", dbg, err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        log::warn!("{}.run | error: {:?}", dbg, err);
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
            log::info!("{}.run | Exit...", dbg);
            connections.wait();
            log::info!("{}.run | Exit", dbg);
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
        thread::sleep(Duration::from_millis(10));
        log::info!("{}.exit | Final connection...", self.dbg);
        match TcpStream::connect_timeout(&self.conf.address, Duration::from_millis(100)) {
            Ok(stream) => {
                log::info!("{}.exit | Final connection - ok", self.dbg);
                stream.shutdown(Shutdown::Both).unwrap();
            }
            Err(err) => {
                log::info!("{}.exit | Final connection error: {:?}", self.dbg, err);
            }
        };
    }
}
