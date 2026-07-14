use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use api_tools::{api::reply::api_reply::ApiReply, client::{api_query::{ApiQuery, ApiQueryKind, ApiQuerySql}, api_request::ApiRequest}};
use indexmap::IndexMap;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object}, future::{Future, Sink}, Service, ServiceWaiting}, sync::{channel::{self, RecvTimeoutError}, Handles, Owner}, thread_pool::Scheduler};
use crate::{domain::{RECV_TIMEOUT, Receiver, Sender}, infra::ApiClientConf};
///
/// API Reply
type Reply = Result<Vec<IndexMap<String, serde_json::Value>>, Error>;
///
/// ## Direct access to the [API-Server](https://github.com/a-givertzman/api-server)
///
/// - Automatically connects to the server on request
/// - Keeps connection alive to be faster
/// 
/// ### Configuration
/// ```yaml
/// ```
pub struct ApiClient {
    name: Name,
    conf: ApiClientConf,
    // request: Arc<RwL ApiRequest,
    send: Sender<(String, Sink<Reply>)>,
    recv: Owner<Receiver<(String, Sink<Reply>)>>,
    scheduler: Scheduler,
    is_started: Arc<AtomicBool>,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl ApiClient {
    ///
    /// Returns [ApiClient] new instance
    pub fn new(conf: ApiClientConf, scheduler: Scheduler,) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        let (send, recv) = channel::unbounded();
        Self {
            name: conf.name.clone(),
            conf,
            send,
            recv: Owner::new(recv),
            scheduler,
            is_started: Arc::new(AtomicBool::new(false)),
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Performs an API request with the parameters specified in the constructor
    pub fn fetch(&self, sql: impl Into<String>) -> Future<Result<Vec<IndexMap<String, serde_json::Value>>, Error>> {
        let sql = sql.into();
        let (result, sink) = Future::new();
        match self.is_started.load(Ordering::Acquire) && !self.is_finished() {
            true => {
                if let Err(err) = self.send.send((sql.clone(), sink.clone())) {
                    sink.add(Err(Error::new(&self.dbg, "fetch").pass_with("Send query error", err.to_string())));
                }
            }
            false => sink.add(Err(Error::new(&self.dbg, "fetch").err("Is not started or already exited"))),
        }
        result
    }
}
//
//
impl Service for ApiClient {
    fn run(&self) -> Result<(), sal_core::error::Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let recv = self.recv.take().unwrap();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let is_started = self.is_started.clone();
        let exit = self.exit.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let error = Error::new(dbg, "run");
            is_started.store(true, Ordering::Release);
            let mut request = ApiRequest::new(
                &name,
                &conf.address,
                &conf.auth_token,
                ApiQuery::new(ApiQueryKind::Sql(ApiQuerySql::new(&conf.database, "select 1;")), true),
                true,
                false,
            );
            while let Err(err) = request.fetch(true) {
                log::warn!("{dbg}.run | Can't connect to the database '{}', \n\terror: {:?}", conf.address, err);
                std::thread::sleep(Duration::from_millis(1000));
                if exit.load(Ordering::Acquire) {
                    break;
                }
            }
            service_release.add(Ok(()));
            while !exit.load(Ordering::Acquire) {
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok((sql, sink)) => {
                        match request.fetch_with(
                            &ApiQuery::new(ApiQueryKind::Sql(ApiQuerySql::new(&conf.database, sql)), true),
                            true,
                        ) {
                            Ok(reply) => {
                                match serde_json::from_slice(&reply) {
                                    Ok(reply) => {
                                        let reply: ApiReply = reply;
                                        sink.add(Ok(reply.data));
                                    }
                                    Err(err) => sink.add(Err(error.pass_with("Deserialize reply error", err.to_string()))),
                                }
                            }
                            Err(err) => sink.add(Err(error.pass_with("Fetch error", err.to_string()))),
                        }
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => {}
                        _ => {
                            log::error!("{dbg}.run | Receive sql error: {:?}", err);
                            break;
                        }
                    }
                }
            }
            is_started.store(false, Ordering::Release);
            log::info!("{dbg}.run | Exit");
        });
        match handle {
            Ok(handle) => {
                log::info!("{}.run | Starting - ok", self.dbg);
                self.handles.push(handle);
                let r = match conf.wait_started {
                    Some(_) => {
                        log::info!("{}.run | Waiting while starting...", self.dbg);
                        service_waiting.wait()
                    }
                    None => Ok(()),
                };
                log::info!("{}.run | Starting - ok", self.dbg);
                r
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
        self.exit.store(true, Ordering::Release);
    }    
}
//
//
impl Object for ApiClient {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for ApiClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApiClient")
            .field("name", &self.name)
            .field("dbg", &self.dbg)
            .finish()
    }
}
