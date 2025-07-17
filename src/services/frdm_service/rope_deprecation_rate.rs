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
    CameraError,
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
        let scheduler = self.scheduler.clone();
        let send_to = services
            .get_link(&conf.send_to)
            .unwrap_or_else(|err| panic!("{}.run | Link {} - Not found, error: {}", dbg, conf.send_to.name(), err));
        let (send1, recv) = channel::unbounded();
        let dbg1 = dbg.clone();
        let name1 = name.clone();
        let conf_pos_service = conf.rope.pos.service();
        let conf_load_service = conf.rope.load.service();
        let exit1 = exit.clone();
        let send2 = send1.clone();
        let services1 = services.clone();
        let pos_point = SubscriptionCriteria::new(conf.rope.pos.link(), Cot::Inf);
        let mut handles = vec![];
        let handle = scheduler.spawn(move || {
            let (_, recv_pos) = services1.subscribe(&conf_pos_service, &name1.join(), &[pos_point]);
            loop {
                match recv_pos.recv_timeout(RECV_TIMEOUT) {
                    Ok(pos) => {
                        if let Err(err) = send1.send((Some(pos), None)) {
                            log::info!("{dbg1}.run | Send 'pos' error: {:?}", err);
                        }
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => {}
                        _ => {
                            log::info!("{dbg1}.run | Recv 'pos' error: {:?}", err);
                            break;
                        }
                    },
                }
                if exit1.load(Ordering::Acquire) {
                    break;
                }
            }
            Ok(())
        });
        handles.push(handle);
        let dbg2 = dbg.clone();
        let name2 = name.clone();
        let exit2 = exit.clone();
        let services2 = services.clone();
        let load_point = SubscriptionCriteria::new(conf.rope.load.link(), Cot::Inf);
        let handle = scheduler.spawn(move || {
            let (_, recv_load) = services2.subscribe(&conf_load_service, &name2.join(), &[load_point]);
            loop {
                match recv_load.recv_timeout(RECV_TIMEOUT) {
                    Ok(load) => {
                        if let Err(err) = send2.send((None, Some(load))) {
                            log::info!("{dbg2}.run | Send 'load' error: {:?}", err);
                        }
                    }
                    Err(err) => match err {
                        RecvTimeoutError::Timeout => {}
                        _ => {
                            log::info!("{dbg2}.run | Recv 'load' error: {:?}", err);
                            break;
                        }
                    },
                }
                if exit2.load(Ordering::Acquire) {
                    break;
                }
            }
            Ok(())
        });
        handles.push(handle);
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
                (NotifyState::Start,          Box::new(|message| log::info!("{}", message))),
                (NotifyState::Exit,           Box::new(|message| log::info!("{}", message))),
                (NotifyState::CameraError,    Box::new(|message| log::error!("{}", message))),
            ]);
            // let slices = (conf.rope.length.as_m() / conf.rope.segment.as_m()).ceil() as usize;
            // let mut slices: Vec<RopeSlice> = (0..slices).map(|slice| {
            //     RopeSlice::new(slice, &conf.rope.bendings)
            // }).collect();
            let conf_table = conf.table.clone();
            let mut rope_slices = RopeSlices::new(conf.rope, |ix, deprecation| {
                let dbg = &dbg.clone();
                let sql = format!("update {} set deprecation = deprecation + {} where id = {ix}", conf_table, deprecation);
                let sql = Point::new(
                    tx_id,
                    &Name::new(dbg, "sql").join(),
                    sql,
                );
                if let Err(err) = send_to.send(sql) {
                    log::info!("{dbg}.run | Send 'load' error: {:?}", err);
                }
            });
            loop {
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok((pos, load)) => {
                        rope_slices.add(pos, load);
                        // match (pos, load) {
                        //     (None, None) => {},
                        //     (None, Some(load)) => for slice in &mut slices { slice.add_load(load.clone()) },
                        //     (Some(pos), None) => for slice in &mut slices { slice.add_pos(pos.clone()) },
                        //     (Some(pos), Some(load)) => {
                        //         for slice in &mut slices {
                        //             slice.add_pos(pos.clone());
                        //             slice.add_load(load.clone());
                        //         }
                        //     }
                        // }
                        // for slice in &mut slices {
                        //     if let Some(deprecation) = slice.deprecation(&conf.rope.bendings) {
                        //         let sql = format!("update {} set deprecation = deprecation + {}", conf.table, deprecation);
                        //         let sql = Point::new(
                        //             tx_id,
                        //             &Name::new(dbg, "sql").join(),
                        //             sql,
                        //         );
                        //         if let Err(err) = send_to.send(sql) {
                        //             log::info!("{dbg}.run | Send 'load' error: {:?}", err);
                        //         }
                        //     }
                        // }
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
