//!
//! MockServicePoints implements points() method only.
//! Which returns exactly the vector from which it was created
use std::{fmt::Debug, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}};
use sal_core::error::Error;
use sal_sync::services::{entity::{Name, Object, PointConfig}, Service};
///
/// MockServicePoints implements points() method only.
/// Which returns exactly the vector from which it was created
pub struct MockServicePoints {
    id: String,
    name: Name,
    points: Vec<PointConfig>,
    is_finished: Arc<AtomicBool>,
}
//
// 
impl MockServicePoints {
    ///
    /// 
    pub fn new(parent: impl Into<String>, points: Vec<PointConfig>) -> Self {
        let name = Name::new(parent, format!("MockServicePoints{}", COUNT.fetch_add(1, Ordering::Relaxed)));
        Self {
            id: name.join(),
            name,
            points,
            is_finished: Arc::new(AtomicBool::new(false)),
        }
    }
}
//
// 
impl Object for MockServicePoints {
    fn name(&self) -> Name {
        self.name.clone()
    }
}
//
// 
impl Debug for MockServicePoints {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MockServicePoints")
            .field("id", &self.id)
            .finish()
    }
}
//
// 
impl Service for MockServicePoints {
    //
    //
    fn run(&self) -> Result<(), Error> {
        let err = Error::new(&self.id, "run").err("Not implemented");
        log::warn!("{}", err);
        Err(err)
    }
    //
    //
    fn wait(&self) -> Result<(), Error> {
        // while !self.handle.is_empty() {
        //     if let Some(handle) = self.handle.pop() {
        //         if let Err(err) = handle.join() {
        //             log::warn!("{}.wait | Error: {:?}", self.dbg, err);
        //             return Err(Error::new(&self.dbg, "wait").pass(format!("{:?}", err)));
        //         }
        //     }
        // }
        self.is_finished.store(true, Ordering::SeqCst);
        Ok(())
    }
    //
    //
    fn is_finished(&self) -> bool {
        self.is_finished.load(Ordering::SeqCst)
    }
    //
    // 
    fn exit(&self) {
        log::debug!("{}.run | Not implemented", self.id);
    }
    //
    //
    fn points(&self) -> Vec<PointConfig> {
        log::debug!("{}.points | Returning: {:#?}", self.id, self.points);
        self.points.clone()
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
