use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    services::{entity::{Cot, Name, Object, Point},
    Service, Services, SubscriptionCriteria, RECV_TIMEOUT}, sync::{channel::RecvTimeoutError, Handles, Owner},
    thread_pool::Scheduler,
};
use crate::{
    services::{RopeDeprecationRateConf, RopeSlices}
};

///
/// ## Rope deprecation rate
/// - Counting passes rope via cargo block
/// - Including:
///     - Rope width
///     - Block sizes
///     - Current rope load
pub struct RopeDeprecationRate<Updates> {
    name: Name,
    txid: usize,
    conf: RopeDeprecationRateConf,
    updates: Owner<Updates>,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl<Updates> RopeDeprecationRate<Updates> {
    pub fn new(parent: impl Into<String>, txid: usize, conf: RopeDeprecationRateConf, updates: Updates, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let name = Name::new(parent, "RopeDeprecationRate");
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            txid,
            conf,
            updates: Owner::new(updates),
            services,
            scheduler,
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
}
//
//
impl<Updates> Object for RopeDeprecationRate<Updates> {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl<Updates> std::fmt::Debug for RopeDeprecationRate<Updates> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RopeDeprecationRate")
            .field("dbg", &self.dbg)
            .finish()
    }
}
///
/// Used for logging
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NotifyState {
    Start,
    Exit,
    SendError,
}
//
// 
impl<Updates> Service for RopeDeprecationRate<Updates> where 
    Updates: Fn(f64),
    Updates: Send + Sync + 'static {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let tx_id = self.txid;
        let conf = self.conf.clone();
        let updates = self.updates.take().unwrap();
        let services = self.services.clone();
        let exit = self.exit.clone();
        let send_to = services
            .get_link(&conf.send_to)
            .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
        let points = [
            &conf.crane.rope.pos,
            &conf.crane.rope.load,
            &conf.crane.boom.main_angle,
            &conf.crane.boom.rotary_angle,
        ].map(|point| SubscriptionCriteria::new(point, Cot::Inf));
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
                let sql = Point::new(tx_id, &Name::new(dbg, "sql").join(), sql);
                if let Err(err) = send_to.send(sql) {
                    log::info!("{dbg}.run | Send 'deprecation' error: {:?}", err);
                }
            });
            loop {
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok(point) => {
                        match point.name() {
                            name if name == conf.crane.rope.pos => {
                                (updates)(point.to_double().as_double().value);
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
