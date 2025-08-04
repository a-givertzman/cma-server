use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use crate::{infra::ApiClientConf, services::frdm_service::crane_conf::CraneConf};

///
/// ## Config for RopeDeprecation
#[derive(Debug, PartialEq, Clone)]
pub struct RopeDeprecationConf {
    pub name: Name,
    /// API access parameters
    pub api: ApiClientConf,
    /// Names of the database table used for storing rope deprecation values
    pub table: String,
    /// Service name, to subscribe for rope positin and crane angles event's
    pub subscribe: String,
    /// The configuration parameters for the crane elements and rope rope
    pub crane: CraneConf,
}
//
// 
impl RopeDeprecationConf {
    ///
    /// Returns [RopeDeprecationConf] new instance
    /// - `table` - database table used for storing a rope deprecation values
    pub fn new(parent: impl Into<String>, conf: ConfTree, api: ApiClientConf) -> Self {
        let parent = parent.into();
        let me = "RopeDeprecationConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::debug!("{dbg}.new | name: {:?}", name);
        let table = conf.get("table").expect(&format!("{dbg}.new | 'table' - not found or wrong configuration"));
        log::debug!("{dbg}.new | table: {:?}", table);
        let subscribe = conf.get("subscribe").expect(&format!("{dbg}.new | 'subscribe' - not found or wrong configuration"));
        log::debug!("{dbg}.new | subscribe: {:?}", subscribe);
        let crane = conf.get("crane").expect(&format!("{dbg}.new | 'crane' - not found or wrong configuration"));
        let crane = CraneConf::new(&name, crane);
        log::trace!("{dbg}.new | crane: {:?}", crane);
        Self {
            name,
            api,
            table,
            subscribe,
            crane,
        }
    }
}
