use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use api_tools::client::api_request::ApiRequest;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object}, Service}, sync::Handles, thread_pool::Scheduler};

use crate::infra::ApiClientConf;

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
        Self {
            name,
            conf,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
    }    
}
//
//
impl Service for ApiClient {
    fn run(&self) -> Result<(), sal_core::error::Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            loop {
                
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
