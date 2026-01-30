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
//! service VirtualDevice MocIed12:
//!     path: './test_ied12.ods'            # Optional, if signal have to be charged from the table
//!     api:                                # Optional, if databese access for example required
//!         address: 0.0.0.0:8080
//!         auth-token: 123!@#
//!         database: crane_data_server
//!     inputs:                             # Input signal to be charged from the specified table file, or calculated in `Task`
//!         point Winch.ValveEV1: 
//!             type: Bool
//!             history: rw
//!         point Winch.ValveEV2: 
//!             type: Bool
//!             history: rw
//!         point Winch.EncoderBR1: 
//!             type: Int
//!             comment: 'Скорость об/мин'
//!     results:
//!         point Result.Name1:             # the name of calculated result to be stored into the table column 'Result.Name1/result'
//!             type: Int
//!         sql Result.Name2:               # the name of result stored in the database, to be stored into the table column 'Result.Name2/result'
//!             sql: 'select Name2 from table_name'
//!             delay:  10ms                # Optional delay, to be awaited before select apears
//! ```
//! 
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    services::{Service, ServiceWaiting, Services, entity::{Name, Object}}, sync::{Handles, Owner}, thread_pool::Scheduler
};
use crate::{infra::ApiClient, services::{Table, VirtualDeviceConf}};
///
/// ## `VirtualDevice` Service | Emulation of the real device behavior
/// - Read events from the table file
/// - Calculate events in the `Task`
pub struct VirtualDevice {
    name: Name,
    conf: VirtualDeviceConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    api_client: Owner<Arc<ApiClient>>,
    handles: Arc<Handles<()>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl VirtualDevice {
    ///
    /// Crteates [VirtualDevice] new instance
    pub fn new(conf: VirtualDeviceConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf,
            services,
            scheduler,
            api_client: Owner::empty(),
            handles: Arc::new(Handles::new(&dbg)),
            exit: Arc::new(AtomicBool::new(false)),
            dbg,
        }
    }
    ///
    /// Make a select query to the API
    /// - Returns value or error in the string
    fn select(dbg: &Dbg, api_client: Arc<ApiClient>, sql: impl Into<String>) -> String {
        let sql = sql.into();
        log::debug!("{dbg}.select | fetching sql: '{sql}'");
        match api_client.fetch(&sql).wait() {
            Ok(reply) => match reply {
                Ok(reply) => {
                    let r = reply.first()
                        .map(|r| r.first())
                        .flatten()
                        .map(|r| r.1);
                    match r {
                        Some(v) => v.to_string(),
                        None => "No results".to_string()
                    }
                }
                Err(err) => {
                    let err = format!("{dbg}.select | Sql '{sql}' returns error: {:?}", err);
                    log::warn!("{err}");
                    err
                }
            },
            Err(err) => {
                let err = format!("{dbg}.select | Fetch sql '{sql}' error: {:?}", err);
                log::warn!("{err}");
                err
            }
        }
    }
}
//
//
impl Object for VirtualDevice {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for VirtualDevice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VirtualDevice")
            .field("dbg", &self.dbg)
            .finish()
    }
}
//
//
impl Service for VirtualDevice {
    //
    // 
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let name = self.name.clone();
        let conf = self.conf.clone();
        let services = self.services.clone();
        let exit = self.exit.clone();
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let table = match (&conf.path, &conf.sheet) {
            (Some(path), Some(sheet)) => Some(Table::load(&name, path, sheet)
                .map_err(|err| format!("{}.run | Can't open table '{}', error: {:?}", self.dbg, path, err))?),
            _ => None,
        };
        let api_client = Arc::new(ApiClient::new(conf.api.clone(), self.scheduler.clone()));
        self.api_client.replace(api_client.clone());
        // self.tasks.insert(api_client.name().join(), api_client.clone());
        // api_client.run()?;
        log::info!("{}.run | ApiClient ready", self.dbg);
        // let subscription: Vec<SubscriptionCriteria> = [
        //         conf.rope_deprecation.crane.rope.pos.clone(),
        //         conf.rope_deprecation.crane.rope.load.clone(),
        //     ]
        //     .iter().chain(
        //         conf.rope_deprecation.crane.booms.iter().filter_map(|(_, b)| {
        //             match &b.angle {
        //                 crate::services::frdm_service::InputKind::Const(_) => None,
        //                 crate::services::frdm_service::InputKind::Point(v) => Some(v),
        //             }
        //         }),
        //     )
        //     .map(|point| {
        //         let subscription = SubscriptionCriteria::new(point, Cot::Inf);
        //         log::trace!("{dbg}.run | Subscription: {:?}", subscription);
        //         subscription
        //     })
        //     .collect();
        // let (_, recv) = services.subscribe(&conf.subscribe, &name.join(), &subscription);
        let dbg = self.dbg.clone();
        let handle = self.scheduler.spawn(move || {
            service_release.add(Ok(()));
            log::info!("{}.run | Starting - Ok", dbg);
            match table {
                Some(mut table) => {
                    let sheet = table.sheet_mut();
                    let (y_start, y_end) = (6, 12);
                    for row in sheet.iter_rows((y_start,0)..(y_end,10)) {
                        log::debug!("{dbg}.run | row: {:?}", row);
                        // let sql: String = todo!("Get the sql from the current result");
                        // let result = Self::select(&dbg, api_client.clone(), sql);
                        if exit.load(Ordering::Acquire) {
                            break;
                        }
                    }
                }
                None => {
                    log::warn!("{}.run | Table or sheet wasn't specified", dbg);
                }
            }
            log::info!("{dbg}.run | Exit");
            Ok(())
        });
        match handle {
            Ok(handle) => {
                self.handles.push(handle);
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
        if let Some(s) = self.api_client.take() {
            s.wait()?;
        }
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
        if let Some(s) = self.api_client.take() {
            s.exit();
            self.api_client.replace(s);
        }
    }    
}

