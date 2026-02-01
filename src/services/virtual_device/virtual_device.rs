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
use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Instant};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    collections::FxIndexMap, services::{Service, ServiceWaiting, Services, entity::{Name, Object, PointTxId}}, sync::{Handles, Owner}, thread_pool::Scheduler
};
use crate::{domain::constants::constants::RECV_TIMEOUT, infra::ApiClient, services::{Header, InputBlock, ResultBlock, Table, VirtualDeviceConf}};
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
        let dbg = self.dbg.clone();
        log::info!("{dbg}.run | Starting...");
        let name = self.name.clone();
        let txid = PointTxId::from_str(&name.join());
        let conf = self.conf.clone();
        let services = self.services.clone();
        let exit = self.exit.clone();
        let error = Error::new(&dbg, "run");
        let service_waiting = ServiceWaiting::new(&name, conf.wait_started);
        let service_release = service_waiting.release();
        let table = match (&conf.path, &conf.sheet) {
            (Some(path), Some(sheet)) => Some(Table::load(&name, path, sheet)
                .map_err(|err| format!("{dbg}.run | Can't open table '{}', error: {:?}", path, err))?),
            _ => None,
        };
        let api_client = Arc::new(ApiClient::new(conf.api.clone(), self.scheduler.clone()));
        self.api_client.replace(api_client.clone());
        // api_client.run()?;
        log::info!("{dbg}.run | ApiClient ready");
        let send_to = services.get_link(&conf.send_to).map_err(|err| error.pass_with("Can't get 'send-to' link", err))?;
        let (_, recv) = {
            let points = services.points(&name).wait().map_err(|err| error.pass_with("Can't get points", err))?;
            let subscriptions = conf.subscribe.with(&points);
            let (service, points) = subscriptions.iter().next().ok_or(error.err("Can't find subscription in the config"))?;
            services.subscribe(service, &name.join(), points.as_ref().unwrap_or(&vec![]))
        };
        let handle = self.scheduler.spawn(move || {
            service_release.add(Ok(()));
            log::info!("{dbg}.run | Starting - Ok");
            match table {
                Some(mut table) => {
                    let header = Header::from(table.sheet());
                    let input_block = InputBlock::new("Input values", "time", "name", "value", &header);
                    let rows = table.sheet().row_header_max();
                    let columns = table.sheet().col_header_max();
                    let start = header.end() + 1;
                    for  row_ix in start..(rows - start) {
                        let row = table.row(row_ix, columns);
                        if let Some(index) = row.get(0) {
                            if let spreadsheet_ods::Value::Number(ix) = index {
                                if ix >= &0.0 {
                                    log::trace!("{dbg}.run | row {row_ix} | Index {ix} | {:?}", row);
                                    match input_block.from_row(&row) {
                                        Some(event) => {
                                            log::debug!("{dbg}.run | row {row_ix} | Index {ix} | Event {:?}", event);
                                            std::thread::sleep(event.time);
                                            match conf.inputs.get(&event.name) {
                                                Some(event_conf) => {
                                                    let point = event.to_point(txid);
                                                    if let Err(err) = send_to.send(point) {
                                                        log::warn!("{dbg}.run | Can't send Event {:?}", event);
                                                    }
                                                }
                                                None => log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Can't find Event '{}' in the config, skipped", event.name),
                                            }
                                            let mut results = FxIndexMap::default();
                                            let time = Instant::now();
                                            let delay = RECV_TIMEOUT * 3;
                                            while time.elapsed() <= delay {
                                                match recv.recv_timeout(RECV_TIMEOUT) {
                                                    Ok(point) => {
                                                        log::debug!("{dbg}.run | row {row_ix} | Index {ix} | Event {:?}", event);
                                                        results.insert(point.name(), point);
                                                    }
                                                    Err(err) => match err {
                                                        kanal::ReceiveErrorTimeout::Timeout => {
                                                            break;
                                                        }
                                                        _ => {
                                                            log::error!("{dbg}.run | row {row_ix} | Index {ix} | Cant recv events, error {:?}", err);
                                                            exit.store(true, Ordering::Release);
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            for (result_name, result_kind) in &conf.results {
                                                let result_block = ResultBlock::new(result_name, "target", "result", "status", &header);
                                                match result_kind {
                                                    crate::services::ResultKind::Event(point_conf) => {
                                                        match results.get(&point_conf.name) {
                                                            Some(point) => {
                                                                log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Result '{}': {:?}", point_conf.name, point.value());
                                                                let result = point.to_double().as_double().value;
                                                                result_block.write(row_ix, result, &mut table);
                                                            }
                                                            None => {
                                                                log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Can't find '{}' in the results", point_conf.name);
                                                            }
                                                        }
                                                    }
                                                    crate::services::ResultKind::Sql(sql_result) => {
                                                        match api_client.fetch(&sql_result.sql).wait().flatten() {
                                                            Ok(result) => {
                                                                log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Result '{}': {:?}", sql_result.name, result);
                                                                // result_block.write(row_ix, result, &mut table);
                                                            }
                                                            Err(err) => {
                                                                log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Can't fetch '{}' result from API", sql_result.name);
                                                            }
                                                        }
                                                    }
                                                }

                                            }

                                            // Write Results here
                                            // let result = ResultBlock::new(&self, &header, row)
                                        }
                                        None => log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Can't parse Input Event, skipped"),
                                    }
                                }
                            }
                        }
                    }
                    // for ((row, col), cell) in sheet.iter_rows((row_start, 0)..(row_end, columns)) {
                    //     log::debug!("{dbg}.run | row {} col {} | {:?}", row, col, cell.value);
                    //     // let sql: String = todo!("Get the sql from the current result");
                    //     // let result = Self::select(&dbg, api_client.clone(), sql);
                    //     if exit.load(Ordering::Acquire) {
                    //         break;
                    //     }
                    // }
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

