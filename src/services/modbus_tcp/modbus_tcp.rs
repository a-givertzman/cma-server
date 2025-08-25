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
use sal_core::{dbg::Dbg, error::Error};
use sal_sync::{services::{entity::{Name, Object, Point}, Service, Services}, sync::{Handles, Owner}, thread_pool::Scheduler};
use crate::{domain::Sender, services::ModbusTcpConf};

///
/// Do something ...
pub struct ModbusTcp {
    name: Name,
    conf: ModbusTcpConf,
    services: Arc<Services>,
    scheduler: Scheduler,
    handles: Handles<()>,
    exit: Arc<AtomicBool>,
    dbg: Dbg,
}
//
//
impl ModbusTcp {
    //
    /// Crteates new instance of the ModbusTcp 
    pub fn new(conf: ModbusTcpConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        let dbg = Dbg::new(conf.name.parent(), conf.name.me());
        Self {
            name: conf.name.clone(),
            conf,
            services,
            scheduler,
            handles: Handles::new(&dbg),
            dbg,
            exit: Arc::new(AtomicBool::new(false)),
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
    fn get_link(&self, name: &str) -> Sender<Point> {
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
        let exit = self.exit.clone();
        log::info!("{}.run | Preparing thread...", dbg);
        let handle = self.scheduler.spawn(move || {
            loop {
                if exit.load(Ordering::SeqCst) {
                    break;
                }
            }
            Ok(())
        }).map_err(|err| Error::new(&self.dbg, "run").pass_with("Start failed", err))?;
        self.handles.push(handle);
        log::info!("{}.run | Starting - ok", self.dbg);
        Ok(())
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.handles.is_finished()
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        self.handles.wait()
    }
    //
    //
    fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }    
}