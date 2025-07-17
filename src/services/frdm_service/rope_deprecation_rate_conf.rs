use sal_sync::services::{entity::Name, LinkName};
use crate::services::RopeConf;

///
/// Config for RopeDeprecationRate:
#[derive(Debug, PartialEq, Clone)]
pub struct RopeDeprecationRateConf {
    pub name: Name,
    pub rope: RopeConf,
    pub send_to: LinkName,
    pub table: String,
}
//
// 
impl RopeDeprecationRateConf {
    ///
    /// Returns [RopeDeprecationRateConf] new instance
    /// - `table` - database table used for storing a rope deprecation values
    pub fn new(parent: impl Into<String>, rope: RopeConf, send_to: LinkName, table: String) -> Self {
        let me = "RopeDeprecationRateConf";
        Self {
            name: Name::new(parent, me),
            rope,
            send_to,
            table,
        }
    }
}
