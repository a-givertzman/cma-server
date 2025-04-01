//!
//! MockServicePoints implements points() method only.
//! Which returns exactly the vector from which it was created
use std::{fmt::Debug, sync::atomic::{AtomicUsize, Ordering}};
use sal_core::error::Error;
use sal_sync::services::{entity::{name::Name, object::Object, point::point_config::PointConfig}, service::{service::Service, service_handles::ServiceHandles}};
///
/// MockServicePoints implements points() method only.
/// Which returns exactly the vector from which it was created
pub struct MockServicePoints {
    id: String,
    name: Name,
    points: Vec<PointConfig>,
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
        }
    }
}
//
// 
impl Object for MockServicePoints {
    fn id(&self) -> &str {
        &self.id
    }
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
    fn run(&mut self) -> Result<ServiceHandles<()>, Error> {
        let err = Error::new(&self.id, "run").err("Not implemented");
        log::warn!("{}", err);
        Err(err)
    }
    ///
    /// 
    fn exit(&self) {
        log::debug!("{}.run | Not implemented", self.id);
    }    
    fn points(&self) -> Vec<PointConfig> {
        log::debug!("{}.points | Returning: {:#?}", self.id, self.points);
        self.points.clone()
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
