use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use frdm_tools::{Eval, EvalMut};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{kernel::state::ChangeNotify, services::{entity::{Cot, Name, Object, Point, PointTxId}, Service, Services, SubscriptionCriteria, RECV_TIMEOUT}, sync::{channel::{self, RecvTimeoutError}, Handles}, thread_pool::Scheduler};
use crate::services::{RopeDeprecationRateConf, RopeSlice};

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
        let handles_clone = self.handles.clone();
        let (send1, recv) = channel::unbounded();
        let dbg1 = dbg.clone();
        let name1 = name.clone();
        let conf_pos_service = conf.pos.service();
        let conf_load_service = conf.load.service();
        let exit1 = exit.clone();
        let send2 = send1.clone();
        let services1 = services.clone();
        let pos_point = SubscriptionCriteria::new(conf.pos.link(), Cot::Inf);
        let handle1 = scheduler.spawn(move || {
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
        let dbg2 = dbg.clone();
        let name2 = name.clone();
        let exit2 = exit.clone();
        let services2 = services.clone();
        let load_point = SubscriptionCriteria::new(conf.load.link(), Cot::Inf);
        let handle2 = scheduler.spawn(move || {
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
        log::debug!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let dbg = &dbg;
            let notify: ChangeNotify<_, String> = ChangeNotify::new(dbg, NotifyState::Start, vec![
                (NotifyState::Start,          Box::new(|message| log::info!("{}", message))),
                (NotifyState::Exit,           Box::new(|message| log::info!("{}", message))),
                (NotifyState::CameraError,    Box::new(|message| log::error!("{}", message))),
            ]);
            let mut slices: Vec<RopeSlice> = (0..conf.bendings.len()).map(|slice| {
                RopeSlice::new()
            }).collect();
            loop {
                match recv.recv_timeout(RECV_TIMEOUT) {
                    Ok((pos, load)) => {
                        match (pos, load) {
                            (None, None) => {},
                            (None, Some(load)) => for slice in &mut slices { slice.add_load(load.clone()) },
                            (Some(pos), None) => for slice in &mut slices { slice.add_pos(pos.clone()) },
                            (Some(pos), Some(load)) => {
                                for slice in &mut slices {
                                    slice.add_pos(pos.clone());
                                    slice.add_load(load.clone());
                                }
                            }
                        }
                        for slice in &mut slices {
                            if let Some(deprication) = slice.deprication() {
                                let sql = format!("update ", );
                                // if let Err(err) = send2.send((None, Some(load))) {
                                //     log::info!("{dbg2}.run | Send 'load' error: {:?}", err);
                                // }
                            }
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
