//!
//! # `VirtualDevice` `Service` is emulation of the real device behavior. 
//! 
//! It's signals can be charged from file-based test data or calculated in the `Task`-based calculations
//! 
//! Can be used instead of a real device connections such as `UdpClient` or `ProfinetClient` etc.
//! - Signals configuration can be directly copied from the real device
//! - Loading test sequences from the table files (ODS)
//! - Storing results nier by the corresponding input event
//! - Comparison of target and result values to highlight test failures
//! 
use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{
    collections::FxIndexMap, services::{Service, ServiceWaiting, Services, entity::{Name, Object, PointTxId, PointType}}, sync::{Handles, Owner}, thread_pool::Scheduler
};
use crate::{domain::constants::constants::RECV_TIMEOUT, infra::ApiClient, services::{CmdKind, Header, InputBlock, ResultBlock, Table, VirtualDeviceConf}};
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
    fn fetch(dbg: &Dbg, api_client: &Arc<ApiClient>, typ: &PointType, sql: impl Into<String>, delay: Duration) -> impl Into<spreadsheet_ods::Value> {
        let sql = sql.into();
        log::trace!("{dbg}.select | Fetching sql: '{sql}'");
        std::thread::sleep(delay);
        match api_client.fetch(&sql).wait() {
            Ok(reply) => match reply {
                Ok(reply) => {
                    log::trace!("{dbg}.select | Reply: '{:?}'", reply);
                    reply.first()
                        .map(|r| r.first())
                        .flatten()
                        .map(|r| match typ {
                            sal_sync::services::entity::PointConfType::Bool => match r.1 {
                                serde_json::Value::Null => spreadsheet_ods::Value::Text("Empty".to_owned()),
                                serde_json::Value::Bool(v) => spreadsheet_ods::Value::Boolean(*v),
                                serde_json::Value::Number(v) => spreadsheet_ods::Value::Number(v.as_f64().unwrap()),
                                serde_json::Value::String(v) => match v.parse() {
                                    Ok(v) => spreadsheet_ods::Value::Boolean(v),
                                    Err(_) => spreadsheet_ods::Value::Text(v.to_owned()),
                                }
                                serde_json::Value::Array(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                                serde_json::Value::Object(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                            }
                            sal_sync::services::entity::PointConfType::Bytes => todo!(),
                            sal_sync::services::entity::PointConfType::Int => match r.1 {
                                serde_json::Value::Null => spreadsheet_ods::Value::Text("Empty".to_owned()),
                                serde_json::Value::Bool(v) => spreadsheet_ods::Value::Boolean(*v),
                                serde_json::Value::Number(v) => spreadsheet_ods::Value::Number(v.as_f64().unwrap()),
                                serde_json::Value::String(v) => match v.parse() {
                                    Ok(v) => spreadsheet_ods::Value::Number(v),
                                    Err(_) => spreadsheet_ods::Value::Text(v.to_owned()),
                                }
                                serde_json::Value::Array(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                                serde_json::Value::Object(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                            }
                            sal_sync::services::entity::PointConfType::Real => match r.1 {
                                serde_json::Value::Null => spreadsheet_ods::Value::Text("Empty".to_owned()),
                                serde_json::Value::Bool(v) => spreadsheet_ods::Value::Boolean(*v),
                                serde_json::Value::Number(v) => spreadsheet_ods::Value::Number(v.as_f64().unwrap()),
                                serde_json::Value::String(v) => match v.parse() {
                                    Ok(v) => spreadsheet_ods::Value::Number(v),
                                    Err(_) => spreadsheet_ods::Value::Text(v.to_owned()),
                                }
                                serde_json::Value::Array(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                                serde_json::Value::Object(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                            }
                            sal_sync::services::entity::PointConfType::Double => match r.1 {
                                serde_json::Value::Null => spreadsheet_ods::Value::Text("Empty".to_owned()),
                                serde_json::Value::Bool(v) => spreadsheet_ods::Value::Boolean(*v),
                                serde_json::Value::Number(v) => spreadsheet_ods::Value::Number(v.as_f64().unwrap()),
                                serde_json::Value::String(v) => match v.parse() {
                                    Ok(v) => spreadsheet_ods::Value::Number(v),
                                    Err(_) => spreadsheet_ods::Value::Text(v.to_owned()),
                                }
                                serde_json::Value::Array(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                                serde_json::Value::Object(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                            }
                            sal_sync::services::entity::PointConfType::String => match r.1 {
                                serde_json::Value::Null => spreadsheet_ods::Value::Text("Empty".to_owned()),
                                serde_json::Value::Bool(v) => spreadsheet_ods::Value::Boolean(*v),
                                serde_json::Value::Number(v) => spreadsheet_ods::Value::Number(v.as_f64().unwrap()),
                                serde_json::Value::String(v) => spreadsheet_ods::Value::Text(v.to_owned()),
                                serde_json::Value::Array(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                                serde_json::Value::Object(v) => spreadsheet_ods::Value::Text(format!("{:?}", v)),
                            }
                            sal_sync::services::entity::PointConfType::Json => todo!(),
                        }).unwrap_or(spreadsheet_ods::Value::Text("Missed".to_string()))
                }
                Err(err) => {
                    let err = format!("{dbg}.select | Sql '{sql}' returns error: {:?}", err);
                    log::warn!("{err}");
                    spreadsheet_ods::Value::Text(err)
                }
            },
            Err(err) => {
                let err = format!("{dbg}.select | Fetch sql '{sql}' error: {:?}", err);
                log::warn!("{err}");
                spreadsheet_ods::Value::Text(err)
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
        api_client.run()?;
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
            log::info!("{dbg}.run | Starting...");
            match table {
                Some(mut table) => {
                    let header = Header::from(&name, table.sheet());
                    let input_block = InputBlock::new("time", "name", "value", &header);
                    let rows = table.sheet().row_header_max();
                    let columns = table.sheet().col_header_max();
                    let start = header.end() + 1;
                    log::info!("{dbg}.run | Setup...");
                    for cmd in conf.before {
                        match cmd {
                            CmdKind::Sql(sql) => {
                                match api_client.fetch(&sql).wait() {
                                    Ok(reply) => match reply {
                                        Ok(reply) => log::trace!("{dbg}.run | Before sql '{sql}' \n\treply {:?}", reply),
                                        Err(err) => log::warn!("{dbg}.run | Before sql '{sql}' \n\terror {:?}", err),
                                    },
                                    Err(err) => log::warn!("{dbg}.run | Before sql '{sql}' \n\terror {:?}", err),
                                }
                            }
                        }
                    }
                    log::info!("{dbg}.run | Setup - Ok");
                    let mut time = Instant::now();
                    'main: for  row_ix in start..(rows - start) {
                        let row = table.row(row_ix, columns);
                        if let Some(index) = row.get(0) {
                            if let spreadsheet_ods::Value::Number(ix) = index {
                                if ix >= &0.0 {
                                    log::trace!("{dbg}.run | row {row_ix} | Index {ix} | {:?}", row);
                                    match input_block.from_row(&row) {
                                        Ok(event) => {
                                            log::trace!("{dbg}.run | row {row_ix} | Index {ix} | Event {:?}", event);
                                            match conf.inputs.get(&event.name) {
                                                Some(_) => {
                                                    let time_elapsed = time.elapsed();
                                                    if time_elapsed > event.time {
                                                        log::warn!("{dbg}.run | row {} | Index {} | Elapsed {:?} Event.time {:?}, Exceeded {:?}", row_ix - 1, ix - 1.0, time_elapsed, event.time, time_elapsed - event.time);
                                                    } else {
                                                        log::debug!("{dbg}.run | row {} | Index {} | Elapsed {:?} Event.time {:?}", row_ix - 1, ix - 1.0, time_elapsed, event.time);
                                                    }
                                                    log::debug!("{dbg}.run | row {row_ix} | Index {ix} | Input '{}': {:?}", event.name, event.value);
                                                    let point = event.to_point(txid);
                                                    if time_elapsed + Duration::from_millis(1) < event.time {
                                                        std::thread::sleep(event.time - time_elapsed);
                                                    }
                                                    match send_to.send(point) {
                                                        Ok(_) => {
                                                            time = Instant::now();
                                                            let mut results = FxIndexMap::default();
                                                            let delay = 2 * event.time / 3;
                                                            log::trace!("{dbg}.run | row {row_ix} | Index {ix} | Try recv result events in {:?}...", delay);
                                                            let t = Instant::now();
                                                            while !exit.load(Ordering::Acquire) && (t.elapsed() <= delay) {
                                                                match recv.recv_timeout(RECV_TIMEOUT) {
                                                                    Ok(point) => {
                                                                        // log::debug!("{dbg}.run | row {row_ix} | Index {ix} | Result Event {:?}", point);
                                                                        results.insert(point.name().split("/").last().unwrap().to_owned(), point);
                                                                    }
                                                                    Err(err) => match err {
                                                                        kanal::ReceiveErrorTimeout::Timeout => {}
                                                                        _ => {
                                                                            log::error!("{dbg}.run | row {row_ix} | Index {ix} | Cant recv result events, error {:?}", err);
                                                                            exit.store(true, Ordering::Release);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            match results.is_empty() {
                                                                true => log::warn!("{dbg}.run | row {row_ix} | Index {ix} | No result events received"),
                                                                false => log::trace!("{dbg}.run | row {row_ix} | Index {ix} | {} result events received", results.len()),
                                                            }
                                                            if exit.load(Ordering::Acquire) {
                                                                break 'main;
                                                            }
                                                            for (result_name, result_kind) in &conf.results {
                                                                let result_block_name = result_name.split('/').last().unwrap();
                                                                // log::debug!("{dbg}.run | row {row_ix} | Index {ix} | Result name '{}', block '{}'...", result_name, result_block_name);
                                                                let result_block = ResultBlock::new(result_block_name, "target", "result", "status", &header);
                                                                match result_kind {
                                                                    crate::services::ResultKind::Event(_) => {
                                                                        match results.get(result_block_name) {
                                                                            Some(point) => {
                                                                                log::debug!("{dbg}.run | row {row_ix} | Index {ix} | Result '{}': {:?}", result_block_name, point.value());
                                                                                let result = point.to_double().as_double().value;
                                                                                result_block.write_result(row_ix, result, &mut table);
                                                                            }
                                                                            None => {
                                                                                log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Result '{}' - is missed", result_block_name);
                                                                                result_block.write_result(row_ix, "Missed", &mut table);
                                                                            }
                                                                        }
                                                                    }
                                                                    crate::services::ResultKind::Sql(sql_result) => {
                                                                        // log::debug!("{dbg}.run | row {row_ix} | Index {ix} | Result '{}'", sql_result.name);
                                                                        let result = Self::fetch(&dbg, &api_client, &sql_result.typ, &sql_result.sql, sql_result.delay.to_duration()).into();
                                                                        log::debug!("{dbg}.run | row {row_ix} | Index {ix} | Result '{}': {:?}", result_block_name, result);
                                                                        result_block.write_result(row_ix, result, &mut table);
                                                                    }
                                                                }
                                                                if let Err(err) = table.store() {
                                                                    log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Can't write table, errpr: {:?}", err);
                                                                }
                                                                if exit.load(Ordering::Acquire) {
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        Err(err) => {
                                                            log::warn!("{dbg}.run | Can't send Event {:?}, error: {:?}", event, err);
                                                        }
                                                    }
                                                }
                                                None => log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Skipped. Event '{}' isn't configured", event.name),
                                            }
                                        }
                                        Err(err) => log::warn!("{dbg}.run | row {row_ix} | Index {ix} | Skipped. {err}"),
                                    }
                                }
                            }
                        }
                    }
                }
                None => {
                    log::warn!("{}.run | Table or sheet wasn't specified", dbg);
                }
            }
            log::info!("{dbg}.run | Cleaning...");
            for cmd in conf.after {
                match cmd {
                    CmdKind::Sql(sql) => {
                        match api_client.fetch(&sql).wait() {
                            Ok(reply) => match reply {
                                Ok(reply) => log::trace!("{dbg}.run | After sql '{sql}' \n\treply {:?}", reply),
                                Err(err) => log::warn!("{dbg}.run | After sql '{sql}' \n\terror {:?}", err),
                            },
                            Err(err) => log::warn!("{dbg}.run | After sql '{sql}' \n\terror {:?}", err),
                        }
                    }
                }
            }
            log::info!("{dbg}.run | Cleaning - Ok");
            api_client.exit();
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
    fn points(&self) -> Vec<sal_sync::services::entity::PointConf> {
        self.conf.inputs.values().cloned().collect()
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
    }    
}

