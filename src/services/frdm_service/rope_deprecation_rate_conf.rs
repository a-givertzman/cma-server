use sal_sync::services::{entity::Name, LinkName};
use crate::services::BendingsConf;

///
/// Config for RopeDeprecationRate:
#[derive(Debug, PartialEq, Clone)]
pub struct RopeDeprecationRateConf {
    pub name: Name,
    pub pos: LinkName,
    pub load: LinkName,
    pub bendings: BendingsConf,
}
//
// 
impl RopeDeprecationRateConf {
    ///
    /// Returns [RopeDeprecationRateConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, pos: LinkName, load: LinkName, bendings: BendingsConf) -> Self {
        let me = "RopeDeprecationRateConf";
        Self {
            name: Name::new(parent, me),
            pos,
            load,
            bendings,
        }
    }
}
