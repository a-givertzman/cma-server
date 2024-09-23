use std::{
    hash::BuildHasherDefault, net::TcpStream,
    sync::{atomic::{AtomicU32, Ordering}, mpsc::Sender, Arc, Mutex},
    thread::{self, JoinHandle}, time::Duration,
};
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use log::{debug, error, info, trace, warn};
use sal_sync::{collections::map::IndexMapFxHasher, services::{entity::{point::point::Point, status::status::Status}, service::service_cycle::ServiceCycle}};
use crate::{
    conf::slmp_client_config::slmp_client_config::SlmpClientConfig,
    core_::{failure::errors_limit::ErrorLimit, state::{change_notify::ChangeNotify, exit_notify::ExitNotify}},
    services::slmp_client::slmp_db::SlmpDb
};
///
/// Cyclicaly reads SLMP data ranges (DB's) specified in the [conf]
/// - exit - external signal to stop the main read cicle and exit the thread
/// - exit_pair - exit signal from / to notify 'Write' partner to exit the thread
pub struct SlmpRead {
    // tx_id: usize,
    id: String,
    // name: Name,
    conf: SlmpClientConfig,
    dest: Sender<Point>,
    dbs: Arc<Mutex<IndexMapFxHasher<String, SlmpDb>>>,
    // diagnosis: Arc<Mutex<IndexMapFxHasher<DiagKeywd, DiagPoint>>>,
    status: Arc<AtomicU32>,
    exit: Arc<ExitNotify>,
}
impl SlmpRead {
    ///
    /// Creates new instance of the SlpmRead
    pub fn new(
        parent: impl Into<String>,
        tx_id: usize,
        // name: Name,
        conf: SlmpClientConfig,
        dest: Sender<Point>,
        // diagnosis: Arc<Mutex<IndexMapFxHasher<DiagKeywd, DiagPoint>>>,
        status: Arc<AtomicU32>,
        exit: Arc<ExitNotify>,
    ) -> Self {
        let self_id = format!("{}/SlmpRead", parent.into());
        let dbs = Self::build_dbs(&self_id, tx_id, &conf);
        Self {
            // tx_id,
            id: self_id.clone(),
            // name,
            conf,
            dest,
            dbs: Arc::new(Mutex::new(dbs)),
            // diagnosis,
            status,
            exit,
        }
    }
    ///
    /// Sends all configured points from the current DB with the given status
    fn yield_status(self_id: &str, status: Status, dbs: &mut IndexMapFxHasher<String, SlmpDb>, dest: &Sender<Point>) {
        for (db_name, db) in dbs {
            debug!("{}.yield_status | DB '{}' - sending Invalid status...", self_id, db_name);
            match db.yield_status(status, dest) {
                Ok(_) => {}
                Err(err) => {
                    error!("{}.yield_status | send errors: \n\t{:?}", self_id, err);
                }
            };
        }
    }
    ///
    ///
    pub fn build_dbs(self_id: &str, tx_id: usize, conf: &SlmpClientConfig) -> IndexMapFxHasher<String, SlmpDb> {
        let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
        for (db_name, db_conf) in &conf.dbs {
            info!("{}.build_dbs | Configuring SlmpDb: {:?}...", self_id, db_name);
            let db = SlmpDb::new(self_id, tx_id, &db_conf);
            dbs.insert(db_name.clone(), db);
            info!("{}.build_dbs | Configuring SlmpDb: {:?} - ok", self_id, db_name);
        }
        dbs
    }
    ///
    /// Cyclicaly reads data slice from the device,
    pub fn run(&mut self, mut tcp_stream: TcpStream) -> Result<JoinHandle<()>, std::io::Error> {
        info!("{}.read | starting...", self.id);
        let self_id = self.id.clone();
        let status = self.status.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let dbs = self.dbs.clone();
        let dest = self.dest.clone();
        let cycle = conf.cycle.map_or(None, |cycle| if cycle != Duration::ZERO {Some(cycle)} else {None});
        match cycle {
            Some(cycle_interval) => {
                info!("{}.read | Preparing thread...", self_id);
                let handle = thread::Builder::new().name(format!("{}.read", self_id)).spawn(move || {
                    let mut is_connected = ChangeNotify::new(
                        &self_id,
                        false,
                        vec![
                            (true,  Box::new(|message| info!("{}", message))),
                            (false, Box::new(|message| warn!("{}", message))),
                        ],
                    );
                    let mut cycle = ServiceCycle::new(&self_id, cycle_interval);
                    let mut dbs = dbs.lock().unwrap();
                    let mut error_limit = ErrorLimit::new(3);
                    'main: while !exit.get() {
                        is_connected.add(true, format!("{}.read | Connection established", self_id));
                        cycle.start();
                        for (db_name, db) in dbs.iter_mut() {
                            trace!("{}.read | SlmpDb '{}' - reading...", self_id, db_name);
                            match db.read(&mut tcp_stream, &dest) {
                                Ok(_) => {
                                    error_limit.reset();
                                    trace!("{}.read | SlmpDb '{}' - reading - ok", self_id, db_name);
                                }
                                Err(err) => {
                                    warn!("{}.read | SlmpDb '{}' - reading - error: {:?}", self_id, db_name, err);
                                    if error_limit.add().is_err() {
                                        error!("{}.read | SlmpDb '{}' - exceeded reading errors limit, trying to reconnect...", self_id, db_name);
                                        status.store(Status::Invalid.into(), Ordering::SeqCst);
                                        exit.exit_pair();
                                        break 'main;
                                    }
                                }
                            }
                            if exit.get() {
                                break 'main;
                            }
                        }
                        cycle.wait();
                    }
                    if status.load(Ordering::SeqCst) != u32::from(Status::Ok) {
                        Self::yield_status(&self_id, Status::Invalid, &mut dbs, &dest);
                    }
                    info!("{}.read | Exit", self_id);
                });
                info!("{}.read | Started", self.id);
                handle
            }
            None => {
                info!("{}.read | Disabled", self.id);
                let exit = self.exit.clone();
                thread::Builder::new().name(format!("{}.read", self_id)).spawn(move || {
                    info!("{}.read | Started disabled", self_id);
                    while !exit.get() {
                        thread::sleep(Duration::from_millis(64));
                    }
                    if status.load(Ordering::SeqCst) != u32::from(Status::Ok) {
                        let mut dbs = dbs.lock().unwrap();
                        Self::yield_status(&self_id, Status::Invalid, &mut dbs, &dest);
                    }
                    info!("{}.read | Exit", self_id);
                })
            }
        }
    }
}
