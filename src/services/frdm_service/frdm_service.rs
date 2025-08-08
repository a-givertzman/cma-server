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
    pub fn update_db_settings(&self, winch: usize, api_client: Arc<ApiClient>, exit: Arc<AtomicBool>) -> Result<(), Error> {
        let dbg = self.dbg.clone();
        let table = self.conf.table_settings.clone();
        let rope_length = self.conf.rope_deprecation.crane.rope.length.as_m();
        let defect_slices = (rope_length / self.conf.rope_defect.segment.as_m()).round() as usize;
        let deprecation_slices = (rope_length / self.conf.rope_deprecation.crane.rope.segment.as_m()).round() as usize;
        let _ = self.scheduler.spawn(move || {
            log::debug!("{dbg}.update_db_settings | Updating db settings...");
            let sql = format!(r"
                do $$
                begin
                    insert into {table} (id, value) values ('winch{winch}-rope_length', {rope_length})
                    on conflict (id) do
                        update set value = {rope_length} where {table}.id = 'winch{winch}-rope_length';
                    insert into {table} (id, value) values ('winch{winch}-defect_slices', {defect_slices})
                    on conflict (id) do
                        update set value = {defect_slices} where {table}.id = 'winch{winch}-defect_slices';
                    insert into {table} (id, value) values ('winch{winch}-deprecation_slices', {deprecation_slices})
                    on conflict (id) do
                        update set value = {deprecation_slices} where {table}.id = 'winch{winch}-deprecation_slices';
                end; $$
                language plpgsql;
            ");
            log::trace!("{dbg}.update_db_settings | Fetching sql: {:?}", sql);
            loop {
                match api_client.fetch(&sql).wait() {
                    Ok(reply) => {
                        if reply.is_ok() {
                            log::debug!("{dbg}.update_db_settings | Updating db settings - Ok {:?}", reply.unwrap());
                            break;
                        }
                        log::warn!("{dbg}.update_db_settings | Sql reply: {:?}", reply);
                    },
                    Err(err) => {
                        log::error!("{dbg}.update_db_settings | Fetch error: {:?}", err);
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
        let api_client = Arc::new(ApiClient::new(conf.api.clone(), scheduler.clone()));
        self.tasks.insert(api_client.name().join(), api_client.clone());
        api_client.run()?;
        log::info!("{}.run | ApiClient ready", self.dbg);
        self.update_db_settings(1, api_client.clone(), self.exit.clone())?;
        let rope_deprecation = Arc::new(RopeDeprecation::new(
            &self.name,
            conf.rope_deprecation,
            api_client.clone(),
            services.clone(),
            scheduler.clone(),
        ));
        self.tasks.insert(rope_deprecation.name().join(), rope_deprecation.clone());
        rope_deprecation.run()?;
        log::info!("{}.run | RopeDeprecation ready", self.dbg);
        let rope = Arc::new(Rope::new(
            &self.name,
            conf.rope_defect.camera_offset,
            conf.rope_defect.segment,
            conf.rope_defect.segment_threshold,
            rope_deprecation,
        ));
        if !conf.rope_defect.cameras.is_empty() {
            log::info!("{}.run | Camera's configured: {}", self.dbg, conf.rope_defect.cameras.len());
            for (camera_id, camera_conf) in &conf.rope_defect.cameras {
                log::info!("{}.run | Camera '{}' [{}]", self.dbg, camera_conf.name, **camera_id);
                let defect_detection = Arc::new(RopeDefect::new(
                    &self.name,
                    conf.rope_defect.clone(),
                    **camera_id,
                    storage_path.clone(),
                    rope.clone(),
                    api_client.clone(),
                    scheduler.clone(),
                ));
                self.tasks.insert(defect_detection.name().join(), defect_detection.clone());
                defect_detection.run()?; 
            }
        } else {
            log::warn!("{}.run | No Camera's configured", self.dbg);
        }
        log::info!("{}.run | RopeDefect's ready", self.dbg);
        log::info!("{}.run | Starting - Ok", self.dbg);
        Ok(())
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        let mut errors = vec![];
        for task in self.tasks.iter() {
            if let Err(err) = task.value().wait() {
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
        for task in self.tasks.iter() {
            is_finished = is_finished & task.value().is_finished();
        }
        is_finished
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::Release);
        for task in self.tasks.iter() {
            task.value().exit();
        }
    }    
}

