use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    services::{entity::{Name, Object}, Service, ServiceWaiting, RECV_TIMEOUT},
    sync::{channel::RecvTimeoutError, Handles},
    thread_pool::Scheduler,
};
use crate::{infra::ApiClient, services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, Deprecation, Inputs, RopeSections, RopeDeprecationConf}};

///
/// ## Rope deprecation rate
/// - Counting passes rope via cargo block
/// - Including:
///     - Rope width
///     - Block sizes
///     - Current rope load
pub struct RopeDeprecation {
    name: Name,
    conf: RopeDeprecationConf,
    inputs: Arc<Inputs>,
    api_client: Arc<ApiClient>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl RopeDeprecation {
    ///
    /// - `updates` - callback for share rope position
    pub fn new(
        parent: impl Into<String>,
        conf: RopeDeprecationConf,
        inputs: Arc<Inputs>,
        api_client: Arc<ApiClient>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "RopeDeprecation");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            inputs,
            api_client,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
}
//
//
impl Object for RopeDeprecation {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for RopeDeprecation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RopeDeprecation")
            .field("dbg", &self.dbg)
            .finish()
    }
}
// ///
// /// Used for logging
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// enum NotifyState {
//     Start,
//     Exit,
//     SendError,
// }
//
// 
impl Service for RopeDeprecation where {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let conf = self.conf.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let inputs = self.inputs.clone();
        let exit = self.exit.clone();
        let api_client = self.api_client.clone();   // Arc::new(ApiClient::new(&name, conf.api.clone(), self.scheduler.clone()));
        let mut handles = vec![];
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let recv = inputs.listen();
            let conf_table = conf.table.clone();
            let mut deprecation = Deprecation::new(
                dbg,
                &conf.crane,
                inputs.clone(),
                Bendings::new(
                    dbg,
                    &conf.crane.rope,
                    BlockArcs::new(
                        dbg,
                        RopeSections::new(
                            dbg,
                            Blocks::new(
                                dbg,
                                conf.crane.rope.aux_length,
                                &conf.crane.blocks,
                                true,
                                Booms::new(dbg, &conf.crane.booms, inputs, true),
                            ),
                        ),
                    ),
                ),
                |slice_ix, deprecation| {
                    let dbg = &dbg.clone();
                    log::trace!("{dbg}.run | Deprecation om slice {}: {:?}", slice_ix, deprecation);
                    let sql = format!(r"
                        insert into {conf_table} (id, deprecation) values ({slice_ix}, {deprecation})
                        on conflict (id) do update 
                            set deprecation = {conf_table}.deprecation + {deprecation} where {conf_table}.id = {slice_ix};
                    ");
                    log::trace!("{dbg}.run | Fetching sql: {:?}", sql);
                    let reply = api_client.fetch(sql).wait();
                    log::trace!("{dbg}.run | Sql reply: {:?}", reply);
                },
            );
            service_release.add(Ok(()));
            loop {
                log::trace!("{dbg}.run | Receiving events...");
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        log::debug!("{dbg}.run | Received event: {:?}: {}", point.name(), point.to_string().as_string().value);
                        deprecation.eval();
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => {}
                        _ => {
                            break;
                        }
                    },
                }
                if exit.load(Ordering::Acquire) {
                    break;
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        });
        handles.push(handle);
        for handle in handles {
            match handle {
                Ok(handle) => {
                    self.handles.push(handle);
                }
                Err(err) => {
                    let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                    log::warn!("{}", err);
                    return Err(err);
                }
            }
        }
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
