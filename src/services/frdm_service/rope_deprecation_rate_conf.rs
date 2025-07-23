use sal_sync::services::{entity::Name, LinkName};
use crate::services::CraneConf;

///
/// Config for RopeDeprecationRate:
#[derive(Debug, PartialEq, Clone)]
pub struct RopeDeprecationRateConf {
    pub name: Name,
    pub crane: CraneConf,
    pub send_to: LinkName,
    pub subscribe: String,
    pub table: String,
}
//
// 
impl RopeDeprecationRateConf {
    ///
    /// Returns [RopeDeprecationRateConf] new instance
    /// - `table` - database table used for storing a rope deprecation values
    pub fn new(parent: impl Into<String>, crane: CraneConf, send_to: LinkName, subscribe: String, table: String) -> Self {
        let me = "RopeDeprecationRateConf";
        Self {
            name: Name::new(parent, me),
            crane,
            send_to,
            subscribe,
            table,
        }
    }
}
