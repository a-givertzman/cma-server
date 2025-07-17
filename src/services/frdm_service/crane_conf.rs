use std::str::FromStr;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::Name, LinkName};
use crate::services::BendingsConf;
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
    pub main_boom_len: ConfDistance,
    pub rotary_boom_len: ConfDistance,
    pub bendings: BendingsConf,
    pub main_boom_angle: LinkName,
    pub rotary_boom_angle: LinkName,
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
        let main_boom_len = conf.get_distance("main_boom_len").unwrap();
        log::debug!("{dbg}.new | main_boom_len: {:?}", main_boom_len);
        let rotary_boom_len = conf.get_distance("rotary_boom_len").unwrap();
        log::debug!("{dbg}.new | rotary_boom_len: {:?}", rotary_boom_len);
        let bendings = conf.get("bendings").expect(&format!("{dbg}.new | 'bendings' - not found or wrong configuration"));
        let bendings = BendingsConf::new(&name, bendings);
        log::debug!("{dbg}.new | bendings: {:#?}", bendings);
        let main_boom_angle = LinkName::from_str(&conf.get_fn_config(&dbg, "main_boom_angle", &mut vec![]).unwrap().name()).unwrap();
        log::debug!("{dbg}.new | main_boom_angle: {:?}", main_boom_angle);
        let rotary_boom_angle = LinkName::from_str(&conf.get_fn_config(&dbg, "rotary_boom_angle", &mut vec![]).unwrap().name()).unwrap();
        log::debug!("{dbg}.new | rotary_boom_angle: {:?}", rotary_boom_angle);
        Self {
            main_boom_len,
            rotary_boom_len,
            bendings,
            main_boom_angle,
            rotary_boom_angle,
        }
    }
}
