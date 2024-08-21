//!
//! MockServicePoints implements points() method only.
//! Which returns exactly the vector from which it was created
use std::{fmt::Debug, sync::atomic::{AtomicUsize, Ordering}};

use log::debug;
use crate::{conf::point_config::{name::Name, point_config::PointConfig}, core_::object::object::Object, services::service::{service::Service, service_handles::ServiceHandles}};
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
    fn run(&mut self) -> Result<ServiceHandles, String> {
        let message = format!("{}.run | Not implemented", self.id);
        debug!("{}", message);
        Err(message)
    }
    ///
    /// 
    fn exit(&self) {
        debug!("{}.run | Not implemented", self.id);
    }    
    fn points(&self) -> Vec<PointConfig> {
        debug!("{}.points | Returning: {:#?}", self.id, self.points);
        self.points.clone()
    }
}
///
/// Global static counter of FnOut instances
static COUNT: AtomicUsize = AtomicUsize::new(0);
