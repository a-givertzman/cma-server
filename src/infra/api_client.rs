use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use api_tools::{api::reply::api_reply::ApiReply, client::{api_query::{ApiQuery, ApiQueryKind, ApiQuerySql}, api_request::ApiRequest}, error::api_error::ApiError};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object}, future::{Future, Sink}, Service}, sync::{channel::{self, RecvTimeoutError}, Handles, Owner}, thread_pool::Scheduler};
use crate::{domain::{constants::constants::RECV_TIMEOUT, Receiver, Sender}, infra::ApiClientConf};

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
    send: Sender<(String, Sink<ApiReply>)>,
    recv: Owner<Receiver<(String, Sink<ApiReply>)>>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl ApiClient {
    ///
    /// Returns [ApiClient] new instance
    pub fn new(parent: impl Into<String>, conf: ApiClientConf, scheduler: Scheduler,) -> Self {
        let name = Name::new(parent, "ApiClient");
        let dbg = Dbg::new(name.parent(), name.me());
        let (send, recv) = channel::unbounded();
        Self {
            name,
            conf,
            send,
            recv: Owner::new(recv),
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Performs an API request with the parameters specified in the constructor
    fn fetch(&self, sql: impl Into<String>) -> Future<ApiReply> {
        let sql = sql.into();
        let (result, sink) = Future::new();
        if let Err(err) = self.send.send((sql.clone(), sink.clone())) {
            sink.add(
                ApiReply::error(
                    &self.conf.auth_token,
                    "not sampled",
                    true,
                    sql,
                    ApiError::new(format!("{}.fetch | Send query error: {}", self.dbg, err), ""),
                )
            );
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
        let exit = self.exit.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let error = Error::new(dbg, "run");
            let mut request = ApiRequest::new(
                &name,
                &conf.address,
                &conf.auth_token,
                ApiQuery::new(ApiQueryKind::Sql(ApiQuerySql::new(&conf.database, "")), true),
                true,
                false,
            );
            loop {
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok((sql, sink)) => {
                        match request.fetch_with(
                            &ApiQuery::new(ApiQueryKind::Sql(ApiQuerySql::new(&conf.database, "")), true),
                            true,
                        ) {
                            Ok(reply) => {
                                match serde_json::from_slice(&reply) {
                                    Ok(reply) => {
                                        let reply: ApiReply = reply;
                                        sink.add(reply);
                                    }
                                    Err(err) => sink.add(ApiReply::error(
                                        &conf.auth_token,
                                        "not sampled",
                                        true,
                                        sql,
                                        ApiError::new(error.pass_with("Deserialize reply error", err.to_string()).to_string(), ""),
                                    )),
                                }
                            }
                            Err(err) => sink.add(ApiReply::error(
                                &conf.auth_token,
                                "not sampled",
                                true,
                                sql,
                                ApiError::new(error.pass_with("Fetch error", err.to_string()).to_string(), ""),
                            )),
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
                if exit.load(Ordering::Acquire) {
                    break;
                }
            }
            log::info!("{dbg}.run | Exit");
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
