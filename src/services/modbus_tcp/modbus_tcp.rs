//!
//! Service implements kind of bihavior in the separate thread
//! 
//! Basic configuration parameters:
//! ```yaml
//! service ModbusTcp Id:
//!     parameter: value    # meaning
//!     parameter: value    # meaning
//! ```
use std::{sync::{Arc,atomic::{AtomicBool, Ordering}}};
use dashmap::DashMap;
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object, Point}, Service, Services}, thread_pool::Scheduler};
use crate::{domain::Sender, services::{ModbusTcpConf, ModbusTcpRead}};

///
/// Do something ...
pub struct ModbusTcp {
    name: Name,
    conf: ModbusTcpConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    tasks: Arc<DashMap<String, Arc<dyn Service>>>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl ModbusTcp {
    //
    /// Crteates new instance of the [ModbusTcp] 
    pub fn new(conf: ModbusTcpConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
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
}
//
//
impl Object for ModbusTcp {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl std::fmt::Debug for ModbusTcp {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModbusTcp")
            .field("name", &self.name)
            .finish()
    }
}
//
// 
impl Service for ModbusTcp {
    //
    // 
    fn get_link(&self, _: &str) -> Sender<Point> {
        panic!("{}.get_link | Does not support get_link", self.dbg)
        // match self.rxSend.get(name) {
        //     Some(send) => send.clone(),
        //     None => panic!("{}.run | link '{:?}' - not found", self.id, name),
        // }
    }
    //
    //
    fn run(&self) -> Result<(), Error> {
        log::info!("{}.run | Starting...", self.dbg);
        let dbg = self.dbg.clone();
        log::info!("{}.run | Preparing thread...", dbg);
        let read = Arc::new(ModbusTcpRead::new(self.conf.clone(), self.services.clone(), self.scheduler.clone()));
        self.tasks.insert(read.name().join(), read.clone());
        read.run().map_err(|err| Error::new(&self.dbg, "run").pass_with("Start 'Read' failed", err))?;
        let write = Arc::new(ModbusTcpRead::new(self.conf.clone(), self.services.clone(), self.scheduler.clone()));
        self.tasks.insert(write.name().join(), write.clone());
        write.run().map_err(|err| Error::new(&self.dbg, "run").pass_with("Start 'Write' failed", err))?;
        log::info!("{}.run | Starting - ok", self.dbg);
        Ok(())
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.tasks.iter().all(|task| task.value().is_finished())
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
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
        for task in self.tasks.iter() {
            task.value().exit();
        }
    }    
}