use std::sync::Arc;
use sal_sync::{services::Services, thread_pool::Scheduler};

///
/// - Counting passes rope via cargo block
/// - Including:
///     - Rope width
///     - Block sizes
///     - Current rope load
pub struct RopeDeprecationRate {}
//
//
impl RopeDeprecationRate {
    pub fn new(conf: RopeDeprecationRateConf, services: Arc<Services>, scheduler: Scheduler) -> Self {
        Self {

        }
    }
}