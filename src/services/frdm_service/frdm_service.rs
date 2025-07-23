//!
//! # FRDM (Fiber Rope Defects Monitoring)
//! 
//! - Communication with Camera 
//! - Receives current rope position
//! - Scanning the rope for defects
//! - Calculates Rope Depreciation Rate
//! 
//! ## Basic configuration parameters:
//! 
//! ```yaml
//! service FrdmService FrdmService1:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
//! 
use std::{path::Path, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use dashmap::DashMap;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    kernel::state::ChangeNotify,
    services::{conf::{ConfDistance, ConfDistanceUnit}, entity::{Cot, Name, Object, PointTxId}, Service, Services, SubscriptionCriteria},
    thread_pool::Scheduler,
};
use crate::{domain::RwLock, services::{DefectDetection, FrdmServiceConf, RopeDeprecationRate, RopeDeprecationRateConf}};
///
/// FRDM Service (Fiber Rope Defects Monitoring)
pub struct FrdmService {
    name: Name,
    txid: usize,
    conf: FrdmServiceConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    tasks: Arc<DashMap<String, Arc<dyn Service>>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl FrdmService {
    ///
    /// Crteates [FrdmService] new instance
    pub fn new(conf: FrdmServiceConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let tx_id = PointTxId::from_str(&conf.name.join());
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            txid: tx_id,
            conf,
            services,
            scheduler,
            tasks: Arc::new(DashMap::new()),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
}
//
//
impl Object for FrdmService {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for FrdmService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrdmService")
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
impl Service for FrdmService {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        let name = self.name.clone();
        let txid = self.txid;
        let conf = self.conf.clone();
        let services = self.services.clone();
        let scheduler = self.scheduler.clone();
        let storage_path = Path::new("./files").join(
            name.join()
                .chars()
                .enumerate()
                .filter(|(ix, ch)| !((*ix == 0) & (*ch == '/')))
                .map(|(_, ch)| ch)
                .collect::<String>()
        );
        let (_, rope_pos_recv) = services.subscribe(&conf.crane.rope.pos.service(), &name.join(), &[SubscriptionCriteria::new(conf.crane.rope.pos.link(), Cot::Inf)]);
        let rope_pos = Arc::new(RwLock::new(None::<f64>));
        let rope_pos_clone = rope_pos.clone();
        let rope_deprecation = RopeDeprecationRate::new(
            &dbg,
            txid,
            RopeDeprecationRateConf::new(&name, conf.crane.clone(), conf.send_to.clone(), conf.tables.deprecation.clone()),
            move |rope_pos: f64| {
                *rope_pos_clone.write() = Some(rope_pos);
            },
            services.clone(),
            scheduler.clone(),
        );
        rope_deprecation.run()?;
        self.tasks.insert(rope_deprecation.name().join(), Arc::new(rope_deprecation));
        let defect_detection = DefectDetection::new(&name, txid, conf, storage_path, rope_pos, services, scheduler);
        defect_detection.run()?; 
        self.tasks.insert(defect_detection.name().join(), Arc::new(defect_detection));
        Ok(())
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        let mut errors = vec![];
        for item in self.tasks.iter() {
            let service = item.value();
            if let Err(err) = service.wait() {
                errors.push(err);
            }
        }
        errors
            .is_empty()
            .then(|| ())
            .ok_or(
                Error::new(&self.dbg, "wait").pass(errors.iter().fold(String::new(), |acc, err| format!("{}\n{}", acc, err)))
            )
    }
    //
    //
    fn is_finished(&self) -> bool {
        let mut is_finished = false;
        for item in self.tasks.iter() {
            let service = item.value();
            is_finished = is_finished & service.is_finished();
        }
        is_finished
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
        for item in self.tasks.iter() {
            let service = item.value();
            service.exit();
        }
    }    
}

