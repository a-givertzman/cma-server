use std::str::FromStr;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::Name, LinkName};
use crate::services::{BendingsConf, BoomConf, RopeConf};
///
/// ## The configuration parameters for the rope
/// 
/// ### Example:
/// ```yaml
/// crane:
///     main-boom-abgle: point real 'App/Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
///     rotary-boom-abgle: point real 'App/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CraneConf {
    pub bendings: BendingsConf,
    pub boom: BoomConf,
    pub rope: RopeConf,
}
//
// 
impl CraneConf {
    ///
    /// Returns [CraneConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "CraneConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, name);
        let bendings = conf.get("bendings").expect(&format!("{dbg}.new | 'bendings' - not found or wrong configuration"));
        let bendings = BendingsConf::new(&name, bendings);
        log::debug!("{dbg}.new | bendings: {:#?}", bendings);
        let boom = conf.get("boom").expect(&format!("{dbg}.new | 'boom' - not found or wrong configuration"));
        let boom = BoomConf::new(&name, boom);
        log::debug!("{dbg}.new | boom: {:#?}", boom);
        let rope = conf.get("rope").expect(&format!("{dbg}.new | 'rope' - not found or wrong configuration"));
        let rope = RopeConf::new(&name, rope);
        log::debug!("{dbg}.new | rope: {:#?}", rope);
        Self {
            bendings,
            boom,
            rope,
        }
    }
}
