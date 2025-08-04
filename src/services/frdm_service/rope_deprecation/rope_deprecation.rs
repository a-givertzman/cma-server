use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    services::{entity::{Cot, Name, Object},
    Service, Services, SubscriptionCriteria, RECV_TIMEOUT}, sync::{channel::RecvTimeoutError, Handles},
    thread_pool::Scheduler,
};
use crate::{domain::RwLock, infra::ApiClient, services::frdm_service::{RopeDeprecationConf, RopeSlices}};

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
    rope_pos: Arc<RwLock<Option<f64>>>,
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
        services: Arc<Services>,
        scheduler: Scheduler,
    ) -> Self {
        let name = Name::new(parent, "RopeDeprecationRate");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            conf,
            rope_pos: Arc::new(RwLock::new(None::<f64>)),
            services,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Returns current rope pos
    pub fn rope_pos(&self) -> Arc<RwLock<Option<f64>>> {
        self.rope_pos.clone()
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
            .debug_struct("RopeDeprecationRate")
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
        // let updates = self.updates.take().unwrap();
        let rope_pos = self.rope_pos.clone();
        let services = self.services.clone();
        let exit = self.exit.clone();
        let points = [
            &conf.crane.rope.pos,
            &conf.crane.rope.load,
            &conf.crane.boom.main_angle,
            &conf.crane.boom.rotary_angle,
        ].map(|point| SubscriptionCriteria::new(point, Cot::Inf));
        let api_client = ApiClient::new(&name, conf.api.clone(), self.scheduler.clone());
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
            let mut rope_slices = RopeSlices::new(conf.crane.clone(), |ix, deprecation| {
                let dbg = &dbg.clone();
                let sql = format!("update {} set deprecation = deprecation + {} where id = {ix}", conf_table, deprecation);
                api_client.fetch(sql).then(
                    |_| {},
                    |err| {
                        log::warn!("{dbg}.run | Send sql error: {:?}", err);
                    },
                );
            });
            loop {
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        match point.name() {
                            name if name == conf.crane.rope.pos => {
                                *rope_pos.write() = Some(point.to_double().as_double().value);
                                rope_slices.eval(Some(point), None);
                            }
                            name if name == conf.crane.rope.load => {
                                rope_slices.eval(None, Some(point));
                            }
                            name if name == conf.crane.boom.main_angle => {}
                            name if name == conf.crane.boom.rotary_angle => {}
                            _ => log::info!("{dbg}.run | Unknown point name: {:?}", point.name()),
                        }
                        // (pos, load)
                        
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
                    log::info!("{}.run | Starting - ok", self.dbg);
                    self.handles.push(handle);
                }
                Err(err) => {
                    let err = Error::new(&self.dbg, "run").pass_with("Start failed", err.to_string());
                    log::warn!("{}", err);
                    return Err(err);
                }
            }
        }
        Ok(())
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
