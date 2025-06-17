use std::{
    hash::BuildHasherDefault, net::TcpStream,
    sync::{atomic::{AtomicU32, Ordering}, Arc},
};
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_core::error::Error;
use sal_sync::{
    collections::FxIndexMap,
    kernel::state::{ChangeNotify, ExitNotify},
    services::{entity::{Point, Status}, ServiceCycle}, sync::channel::Sender, thread_pool::{JoinHandle, Scheduler},
};
use crate::{
    conf::slmp_client_config::slmp_client_config::SlmpClientConfig,
    core_::{failure::ErrorLimit, Mutex},
    services::slmp_client::slmp_db::SlmpDb
};
///
/// Cyclicaly reads SLMP data ranges (DB's) specified in the [conf]
/// - exit - external signal to stop the main read cicle and exit the thread
/// - exit_pair - exit signal from / to notify 'Write' partner to exit the thread
pub struct SlmpRead {
    // tx_id: usize,
    dbg: String,
    // name: Name,
    conf: SlmpClientConfig,
    dest: Sender<Point>,
    dbs: Arc<Mutex<FxIndexMap<String, SlmpDb>>>,
    // diagnosis: Arc<Mutex<FxIndexMap<DiagKeywd, DiagPoint>>>,
    status: Arc<AtomicU32>,
    scheduler: Scheduler,
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
        // diagnosis: Arc<Mutex<FxIndexMap<DiagKeywd, DiagPoint>>>,
        status: Arc<AtomicU32>,
        scheduler: Scheduler,
        exit: Arc<ExitNotify>,
    ) -> Self {
        let dbg = format!("{}/SlmpRead", parent.into());
        let dbs = Self::build_dbs(&dbg, tx_id, &conf);
        Self {
            // tx_id,
            dbg: dbg.clone(),
            // name,
            conf,
            dest,
            dbs: Arc::new(Mutex::new(dbs)),
            // diagnosis,
            status,
            scheduler,
            exit,
        }
    }
    ///
    /// Sends all configured points from the current DB with the given status
    fn yield_status(self_id: &str, status: Status, dbs: &mut FxIndexMap<String, SlmpDb>, dest: &Sender<Point>) {
        for (db_name, db) in dbs {
            log::debug!("{}.yield_status | DB '{}' - sending Invalid status...", self_id, db_name);
            match db.yield_status(status, dest) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("{}.yield_status | send errors: \n\t{:?}", self_id, err);
                }
            };
        }
    }
    ///
    ///
    pub fn build_dbs(self_id: &str, tx_id: usize, conf: &SlmpClientConfig) -> FxIndexMap<String, SlmpDb> {
        let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
        for (db_name, db_conf) in &conf.dbs {
            log::info!("{}.build_dbs | Configuring SlmpDb: {:?}...", self_id, db_name);
            let db = SlmpDb::new(self_id, tx_id, &db_conf);
            dbs.insert(db_name.clone(), db);
            log::info!("{}.build_dbs | Configuring SlmpDb: {:?} - ok", self_id, db_name);
        }
        dbs
    }
    ///
    /// Cyclicaly reads data slice from the device,
    pub fn run(&mut self, mut tcp_stream: TcpStream) -> Result<JoinHandle<()>, Error> {
        log::info!("{}.read | starting...", self.dbg);
        let dbg = self.dbg.clone();
        let status = self.status.clone();
        let exit = self.exit.clone();
        let conf = self.conf.clone();
        let dbs = self.dbs.clone();
        let dest = self.dest.clone();
        let cycle = conf.cycle.clone();
        log::info!("{}.read | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            let mut is_connected = ChangeNotify::new(
                &dbg,
                false,
                vec![
                    (true,  Box::new(|message| log::info!("{}", message))),
                    (false, Box::new(|message| log::warn!("{}", message))),
                ],
            );
            let mut cycle = ServiceCycle::new(&dbg, cycle);
            let mut dbs = dbs.lock();
            let mut error_limit = ErrorLimit::new(3);
            'main: while !exit.get() {
                is_connected.add(true, format!("{}.read | Connection established", dbg));
                cycle.start();
                for (db_name, db) in dbs.iter_mut() {
                    log::trace!("{}.read | SlmpDb '{}' - reading...", dbg, db_name);
                    match db.read(&mut tcp_stream, &dest) {
                        Ok(_) => {
                            error_limit.reset();
                            log::trace!("{}.read | SlmpDb '{}' - reading - ok", dbg, db_name);
                        }
                        Err(err) => {
                            log::warn!("{}.read | SlmpDb '{}' - reading - error: {:?}", dbg, db_name, err);
                            if error_limit.add().is_err() {
                                log::error!("{}.read | SlmpDb '{}' - exceeded reading errors limit, trying to reconnect...", dbg, db_name);
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
                Self::yield_status(&dbg, Status::Invalid, &mut dbs, &dest);
            }
            log::info!("{}.read | Exit", dbg);
            Ok(())
        });
        log::info!("{}.read | Started", self.dbg);
        handle.map_err(|err| Error::new(&self.dbg, "run").pass_with("Start failed", err))
    }
}
