use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    services::{entity::{Cot, Name, Object}, Service, ServiceWaiting, Services, SubscriptionCriteria, RECV_TIMEOUT}, sync::{channel::RecvTimeoutError, Handles},
    thread_pool::Scheduler,
};
use crate::{infra::ApiClient, services::frdm_service::{Bendings, BlockArcs, Blocks, Booms, Deprication, LooseRopeSections, RopeDeprecationConf}, sync::AtomicUsizeOption};

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
    /// rope position, mm
    rope_pos: Arc<AtomicUsizeOption>,
    api_client: Arc<ApiClient>,
    services: Arc<Services>,
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
        api_client: Arc<ApiClient>,
        services: Arc<Services>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "RopeDeprecation");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            rope_pos: Arc::new(AtomicUsizeOption::new(None)),
            api_client,
            services,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Returns current rope pos, mm
    pub fn rope_pos(&self) -> Option<f64> {
        match self.rope_pos.load() {
            Some(val) => Some(val as f64),
            None => None,
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
        let rope_pos = self.rope_pos.clone();
        let services = self.services.clone();
        let exit = self.exit.clone();
        let points = [
            &conf.crane.rope.pos,
            &conf.crane.rope.load,
            // &conf.crane.booms.main_angle,
            // &conf.crane.booms.rotary_angle,
        ].map(|point| {
            let subscription = SubscriptionCriteria::new(point, Cot::Inf);
            log::trace!("{dbg}.run | Subscription: {:?}", subscription);
            subscription
        });
        let api_client = self.api_client.clone();   // Arc::new(ApiClient::new(&name, conf.api.clone(), self.scheduler.clone()));
        let mut handles = vec![];
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let (_, recv) = services.subscribe(&conf.subscribe, &name.join(), &points);
            // let mut notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
            //     (NotifyState::Start,          Box::new(|message| log::info!("{}", message))),
            //     (NotifyState::Exit,           Box::new(|message| log::info!("{}", message))),
            //     (NotifyState::SendError,      Box::new(|message| log::error!("{}", message))),
            // ]);
            let conf_table = conf.table.clone();
            let mut subscriptions = vec![];
            let mut deprecation = Deprication::new(
                dbg,
                &conf.crane,
                Bendings::new(
                    dbg,
                    conf.crane.rope.pos.clone(),
                    conf.crane.rope.winch_len,
                    BlockArcs::new(
                        dbg,
                        LooseRopeSections::new(
                            dbg,
                            Blocks::new(
                                dbg,
                                &conf.crane.blocks,
                                Booms::new(dbg, &conf.crane.booms, &mut subscriptions),
                            ),
                        ),
                    ),
                ),
                subscriptions,
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
            // let mut rope_slices = RopeSlices::new(&name, conf.crane.clone(), |ix, deprecation| {
            //     let dbg = &dbg.clone();
            //     log::trace!("{dbg}.run | Deprecation om slice {}: {:?}", ix, deprecation);
            //     let sql = format!(r"
            //         insert into {conf_table} (id, deprecation) values ({ix}, {deprecation})
            //         on conflict (id) do update 
            //             set deprecation = {conf_table}.deprecation + {deprecation} where {conf_table}.id = {ix};
            //     ");
            //     log::trace!("{dbg}.run | Fetching sql: {:?}", sql);
            //     let reply = api_client.fetch(sql).wait();
            //     log::trace!("{dbg}.run | Sql reply: {:?}", reply);
            // });
            service_release.add(Ok(()));
            loop {
                log::trace!("{dbg}.run | Receiving points...");
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        log::debug!("{dbg}.run | Received point: {:?}: {}", point.name(), point.to_string().as_string().value);
                        deprecation.eval(&point);
                        if let Some(pos) = deprecation.get(&conf.crane.rope.pos) {
                            log::debug!("{dbg}.run | Received rope pos: {:.4?} m", pos);
                            rope_pos.store(Some((pos * 1000.0).round() as usize));
                        }
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
