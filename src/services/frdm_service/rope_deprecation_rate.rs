use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    kernel::state::ChangeNotify, services::{entity::{Cot, Name, Object, Point, PointTxId},
    Service, Services, SubscriptionCriteria, RECV_TIMEOUT}, sync::{channel::{self, RecvTimeoutError}, Handles},
    thread_pool::Scheduler,
};
use crate::services::{RopeDeprecationRateConf, RopeSlices};

///
/// ## Rope deprecation rate
/// - Counting passes rope via cargo block
/// - Including:
///     - Rope width
///     - Block sizes
///     - Current rope load
pub struct RopeDeprecationRate {
    name: Name,
    txid: usize,
    conf: RopeDeprecationRateConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl RopeDeprecationRate {
    pub fn new(parent: impl Into<String>, conf: RopeDeprecationRateConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let name = Name::new(parent, "RopeDeprecationRate");
        let txid = PointTxId::from_str(&name.join());
        let dbg = Dbg::new(name.parent(), name.me());
        Self {
            name,
            txid,
            conf,
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
impl Object for RopeDeprecationRate {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for RopeDeprecationRate {
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
impl Service for RopeDeprecationRate {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let tx_id = self.txid;
        let conf = self.conf.clone();
        let exit = self.exit.clone();
        let services = self.services.clone();
        let send_to = services
            .get_link(&conf.send_to)
            .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
        let conf_service = conf.crane.rope.pos.service();
        let rope_pos_link = conf.crane.rope.pos.link();
        let rope_load_link = conf.crane.rope.load.link();
        let boom_main_angle_link = conf.crane.boom.main_angle.link();
        let boom_rotary_angle_link = conf.crane.boom.rotary_angle.link();
        let points = [
            &rope_pos_link,
            &rope_load_link,
            &boom_main_angle_link,
            &boom_rotary_angle_link,
        ].map(|point| SubscriptionCriteria::new(point, Cot::Inf));
        let mut handles = vec![];
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let (_, recv) = services.subscribe(&conf_service, &name.join(), &points);
            // let mut notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
            //     (NotifyState::Start,          Box::new(|message| log::info!("{}", message))),
            //     (NotifyState::Exit,           Box::new(|message| log::info!("{}", message))),
            //     (NotifyState::SendError,      Box::new(|message| log::error!("{}", message))),
            // ]);
            let conf_table = conf.table.clone();
            let mut rope_slices = RopeSlices::new(conf.crane, |ix, deprecation| {
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
                            name if name.ends_with(&rope_pos_link) => {
                                rope_slices.eval(Some(point), None);
                            }
                            name if name.ends_with(&rope_load_link) => {
                                rope_slices.eval(None, Some(point));
                            }
                            name if name.ends_with(&boom_main_angle_link) => {}
                            name if name.ends_with(&boom_rotary_angle_link) => {}
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
