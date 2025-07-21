use std::{
    collections::HashMap, fmt::Debug, hash::BuildHasherDefault, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Instant 
};
use hashers::fx_hash::FxHasher;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Cot, Name, Object, Point}, Service, Services, SubscriptionCriteria}, sync::{channel::{Receiver, RecvTimeoutError, Sender}, Handles, Owner}, thread_pool::Scheduler};
use serde_json::json;
use crate::{
    conf::tcp_server_conf::TcpServerConf, 
    domain::{
        constants::constants::RECV_TIMEOUT, net::protocols::jds::{
            jds_decode_message::JdsDecodeMessage, 
            jds_deserialize::JdsDeserialize, 
            jds_encode_message::JdsEncodeMessage, 
            jds_serialize::JdsSerialize,
        }, RwLock,
    }, 
    services::server::{
            connections::Action, jds_auth::TcpServerAuth, jds_request::JdsRequest, jds_routes::{JdsRoutes, RouterReply}
        }, 
    tcp::{tcp_read_alive::TcpReadAlive, tcp_stream_write::TcpStreamWrite, tcp_write_alive::TcpWriteAlive},
};

///
/// 
#[derive(Debug)]
pub enum JdsState {
    Unknown,
    Authenticated,
}
//
// 
impl From<usize> for JdsState {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Authenticated,
            _ => Self::Unknown,
        }
    }
}
///
/// - subscribe - service (MultiQueue) to be subscribed on
/// - subscribe_receiver - self name, for example '/App/Jds/127.0.0.1'
/// - jds_state - current state of the Jds autentication pocedure:
///     - Unknown - initial
///     - Authenticated - Auth succeded
/// - auth - Jds-protocol specific kind of auturization on the current TcpServer
/// - connection_id - remote IP for now
/// - cashe - name of the CacheService
/// - req_reply_send - Sender<PointType> - to send points to the client
#[derive(Debug)]
pub struct Shared {
    pub subscribe: String,
    pub subscribe_receiver: String,
    pub jds_state: JdsState,
    pub auth: TcpServerAuth,
    pub cache: Option<String>,
    pub req_reply_send: Vec<Sender<Point>>,
}

///
/// Single Jds over TCP connection
pub struct JdsConnection {
    dbg: Dbg,
    name: Name,
    connection_id: String,
    action_recv: Owner<Receiver<Action>>, 
    services: Arc<Services>,
    conf: TcpServerConf,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
}
//
// 
impl JdsConnection {
    ///
    /// Creates new instance of the [JdsConnection]
    /// - parent - id of the parent
    /// - path - path of the parent
    pub fn new(parent_id: &Dbg, parent: &Name, connection_id: &str, action_recv: Receiver<Action>, services: Arc<Services>, conf: TcpServerConf, scheduler: Scheduler, exit: Arc<AtomicBool>) -> Self {
        let dbg = Dbg::new(parent_id, format!("JdsConnection/{}", connection_id));
        let name = Name::new(parent, "Jds");
        log::debug!("{}.new | name: {:#?}", dbg, name);
        Self {
            name,
            connection_id: connection_id.into(),
            action_recv: Owner::new(action_recv),
            services,
            conf,
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit,
        }
    }

}
//
//
impl Object for JdsConnection {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
//
impl Service for JdsConnection {
    ///
    /// Main loop of the connection 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let self_name = self.name.clone();
        let conf = self.conf.clone();
        let self_conf_send_to = conf.send_to.clone();
        let receiver_name = Name::new(&self_name, &self.connection_id).join();
        let subscribe = self_conf_send_to.service();
        let shared_options: Arc<RwLock<Shared>> = Arc::new(RwLock::new(Shared {
                subscribe: subscribe.clone(), 
                subscribe_receiver: receiver_name.clone(), 
                jds_state: match conf.auth {
                    TcpServerAuth::None => JdsState::Authenticated,
                    _                   => JdsState::Unknown,
                }, 
                auth: conf.auth.clone(),
                // connection_id: self.connection_id.clone(),
                cache: conf.cache.clone(),
                req_reply_send: vec![],
        }));
        let rx_max_length = conf.rx_max_len;
        let action_recv = self.action_recv.take().unwrap();
        let services = self.services.clone();
        let scheduler = self.scheduler.clone();
        let exit = self.exit.clone();
        let exit_pair = Arc::new(AtomicBool::new(false));
        log::info!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            log::info!("{}.run | Preparing thread - ok", dbg);
            let receivers = Arc::new(RwLock::new(
                HashMap::with_hasher(BuildHasherDefault::<FxHasher>::default()),
            ));
            receivers.write().insert(Cot::Req, services.get_link(&self_conf_send_to));
            let points = services.points(&dbg)
                .then(
                    |points| points,
                    |err| {
                        log::error!("{}.functions | Functions::PointId | Requesting points error: {:?}", dbg, err);
                        vec![]
                    },
                )            
                .iter().fold(vec![], |mut points, point_conf| {
                    // points.push(SubscriptionCriteria::new(&point_conf.name, Cot::Inf));
                    // points.push(SubscriptionCriteria::new(&point_conf.name, Cot::ActCon));
                    // points.push(SubscriptionCriteria::new(&point_conf.name, Cot::ActErr));
                    points.push(SubscriptionCriteria::new(&point_conf.name, Cot::ReqCon));
                    points.push(SubscriptionCriteria::new(&point_conf.name, Cot::ReqErr));
                    points
                });
            let send = services.get_link(&self_conf_send_to).unwrap_or_else(|err| {
                panic!("{}.run | services.get_link error: {:#?}", dbg, err);
            });
            log::debug!("{}.run | subscribe: {:?}", dbg, subscribe);
            let (req_reply_send, recv) = services.subscribe(&subscribe, &receiver_name, &points);
            shared_options.write().req_reply_send = vec![req_reply_send.clone()];
            let buffered = rx_max_length > 0;
            let tcp_read_alive = TcpReadAlive::new(
                &dbg,
                Box::new(JdsRoutes::new(
                    &dbg,
                    &self_name,
                    services.clone(),
                    JdsDeserialize::new(
                        format!("{}/TcpReadAlive/JdsRoutes", dbg),
                        JdsDecodeMessage::new(
                            format!("{}/TcpReadAlive/JdsRoutes/JdsDeserialize", dbg),
                        ),
                    ),
                    req_reply_send,
                    |parent_id, parent_name, point, services, shared, scheduler| {
                        let parent_id: Dbg = parent_id;
                        let parent: Name = parent_name;
                        let point: Point = point;
                        log::debug!("{}.run | point from socket: Point( name: {:?}, status: {:?}, cot: {:?}, timestamp: {:?})", parent, point.name(), point.status(), point.cot(), point.timestamp());
                        log::trace!("{}.run | point from socket: \n\t{:?}", parent, point);
                        match point.cot() {
                            Cot::Req => JdsRequest::handle(&parent_id, &parent, 0, point, services, shared, scheduler),
                            _        => {
                                match shared.read().jds_state {
                                    JdsState::Unknown => {
                                        log::warn!("{}.run | Rejected point from socket: \n\t{:?}", parent_id, json!(&point).to_string());
                                        RouterReply::new(None, None)
                                    }
                                    JdsState::Authenticated => {
                                        log::debug!("{}.run | Passed point from socket: \n\t{:?}", parent, json!(&point).to_string());
                                        RouterReply::new(Some(point), None)
                                    }
                                }
                            }
                        }
                    },
                    shared_options,
                    scheduler.clone(),
                )),
                send,
                None,
                Some(exit.clone()),
                Some(exit_pair.clone()),
                Some(scheduler.clone()),
            );
            let tcp_write_alive = TcpWriteAlive::new(
                &dbg,
                None,
                TcpStreamWrite::new(
                    format!("{}/TcpWriteAlive", dbg),
                    buffered,
                    Some(rx_max_length as usize),
                    Box::new(JdsEncodeMessage::new(
                        format!("{}/TcpWriteAlive/TcpStreamWrite", dbg),
                        JdsSerialize::new(
                            format!("{}/TcpWriteAlive/TcpStreamWrite/JdsEncodeMessage", dbg),
                            recv,
                        ),
                    )),
                ),
                Some(exit.clone()),
                Some(exit_pair.clone()),
                Some(scheduler.clone()),
            );
            let keep_timeout = conf.keep_timeout;
            let mut duration = Instant::now();
            loop {
                exit_pair.store(false, Ordering::SeqCst);
                match action_recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(action) => {
                        match action {
                            Action::Continue(tcp_stream) => {
                                log::info!("{}.run | Action - Continue received", dbg);
                                let read = tcp_read_alive.run(tcp_stream.try_clone().unwrap());
                                let write = tcp_write_alive.run(tcp_stream);
                                match (read, write) {
                                    (Ok(_), Ok(_)) => {}
                                    (Ok(_), Err(err)) => log::error!("{}.run | Error: {:?}", dbg, err),
                                    (Err(err), Ok(_)) => log::error!("{}.run | Error: {:?}", dbg, err),
                                    (Err(err1), Err(err2)) => log::error!("{}.run | Errors: \n\t{:?},\n\t{:?}", dbg, err1, err2),
                                }
                                if let Err(err) = tcp_read_alive.wait() {
                                    log::error!("{}.run | Error wait for TcpReadAlive: {:?}", dbg, err);
                                }
                                if let Err(err) = tcp_write_alive.wait() {
                                    log::error!("{}.run | Error wait for TcpWriteAlive: {:?}", dbg, err);
                                }
                                log::info!("{}.run | Finished", dbg);
                                duration = Instant::now();
                            }
                            Action::Exit => {
                                log::info!("{}.run | Action - Exit received", dbg);
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        match err {
                            RecvTimeoutError::Timeout => {}
                            _ => break,
                        }
                    }
                }
                if exit.load(Ordering::SeqCst) {
                    log::info!("{}.run | Detected exit", dbg);
                    break;
                }
                if keep_timeout.checked_sub(duration.elapsed()).is_none() {
                    log::info!("{}.run | Keeped lost connection timeout({:?}) exceeded", dbg, keep_timeout);
                    break;
                }
            }
            if let Err(err) = services.unsubscribe(&subscribe, &receiver_name, &[]) {
                log::error!("{}.run | Unsubscribe error: {:#?}", dbg, err);
            }
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
    }
}
//
//
impl Debug for JdsConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JdsConnection")
            .field("dbg", &self.dbg)
            .field("name", &self.name)
            .field("connection_id", &self.connection_id)
            .finish()
    }
}