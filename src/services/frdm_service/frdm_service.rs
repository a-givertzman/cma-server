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
    services::{entity::{Name, Object}, Service, Services},
    thread_pool::Scheduler,
};
use crate::{infra::ApiClient, services::frdm_service::{FrdmServiceConf, Rope, RopeDefect, RopeDeprecation}};
///
/// FRDM Service | Fiber Rope Defects Monitoring
pub struct FrdmService {
    name: Name,
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
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf,
            services,
            scheduler,
            tasks: Arc::new(DashMap::new()),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    pub fn update_db_settings(&self, exit: Arc<AtomicBool>) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let table = self.conf.table_settings.clone();
        let rope_length = self.conf.rope_deprecation.crane.rope.length.as_m();
        let defect_slices = (rope_length / self.conf.rope_defect.first().unwrap().segment.as_m()).round() as usize;
        let deprecation_slices = (rope_length / self.conf.rope_deprecation.crane.rope.segment.as_m()).round() as usize;
        let api_client = Arc::new(ApiClient::new(&self.conf.name, self.conf.api.clone(), self.scheduler.clone()));
        let _ = self.scheduler.spawn(move || {
            let sql = format!(r"
                insert into {table} (id, value) values
                    ('rope_length', {rope_length})
                    ('defect_slices', {defect_slices})
                    ('deprecation_slices', {deprecation_slices})
            ");
            log::trace!("{dbg}.run | Fetching sql: {:?}", sql);
            loop {
                match api_client.fetch(&sql).wait() {
                    Ok(reply) => {
                        if reply.is_ok() {
                            break;
                        }
                        log::trace!("{dbg}.run | Sql reply: {:?}", reply);
                    },
                    Err(err) => {
                        log::error!("{dbg}.run | Fetch error: {:?}", err);
                        break;
                    }
                }
                if exit.load(Ordering::Acquire) {
                    break;
                }
            }
            Ok(())
        })?;
        Ok(())
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
//
//
impl Service for FrdmService {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let conf = self.conf.clone();
        let services = self.services.clone();
        let scheduler = self.scheduler.clone();
        let storage_path = Path::new("./files").join(
            self.name.join()
                .chars()
                .enumerate()
                .filter(|(ix, ch)| !((*ix == 0) & (*ch == '/')))
                .map(|(_, ch)| ch)
                .collect::<String>()
        );
        self.update_db_settings(self.exit.clone())?;
        let rope_deprecation = Arc::new(RopeDeprecation::new(
            &self.name,
            conf.rope_deprecation,
            services.clone(),
            scheduler.clone(),
        ));
        rope_deprecation.run()?;
        log::info!("{}.run | RopeDeprecation ready", self.dbg);
        self.tasks.insert(rope_deprecation.name().join(), rope_deprecation.clone());
        match conf.rope_defect.first() {
            Some(conf_rope_defect) => {
                log::info!("{}.run | Camera's configured: {}", self.dbg, conf.rope_defect.len());
                let rope = Arc::new(Rope::new(
                    &self.name,
                    conf_rope_defect.camera_offset,
                    conf_rope_defect.segment,
                    conf_rope_defect.segment_threshold,
                    rope_deprecation,
                ));
                for conf_rope_defect in &conf.rope_defect {
                    log::info!("{}.run | Camera '{}'", self.dbg, conf_rope_defect.camera.name);
                    let defect_detection = RopeDefect::new(
                        &self.name,
                        conf_rope_defect.to_owned(),
                        storage_path.clone(),
                        rope.clone(),
                        scheduler.clone(),
                    );
                    defect_detection.run()?; 
                    self.tasks.insert(defect_detection.name().join(), Arc::new(defect_detection));
                }
            }
            None => log::warn!("{}.run | No Camera's configured", self.dbg),
        }
        log::info!("{}.run | RopeDefect's ready", self.dbg);
        log::info!("{}.run | Starting - Ok", self.dbg);
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
            .then(|| {
                log::info!("{}.run | Exit", self.dbg);
                ()
            })
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

