use std::str::FromStr;
use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree}, entity::Name, LinkName};
///
/// ## The configuration parameters for the crane's boom
/// 
/// ### Example:
/// ```yaml
/// boom:
///     main-abgle: point real 'App/Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
///     rotary-abgle: point real 'App/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BoomConf {
    pub main_len: ConfDistance,
    pub rotary_len: ConfDistance,
    pub main_angle: LinkName,
    pub rotary_angle: LinkName,
}
//
// 
impl BoomConf {
    ///
    /// Returns [BoomConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "BoomConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, name);
        let main_len = conf.get_distance("main_len").unwrap();
        log::debug!("{dbg}.new | main_len: {:?}", main_len);
        let rotary_len = conf.get_distance("rotary_len").unwrap();
        log::debug!("{dbg}.new | rotary_len: {:?}", rotary_len);
        let main_angle = LinkName::from_str(&conf.get_fn_config(&dbg, "main_angle", &mut vec![]).unwrap().name()).unwrap();
        log::debug!("{dbg}.new | main_angle: {:?}", main_angle);
        let rotary_angle = LinkName::from_str(&conf.get_fn_config(&dbg, "rotary_angle", &mut vec![]).unwrap().name()).unwrap();
        log::debug!("{dbg}.new | rotary_angle: {:?}", rotary_angle);
        Self {
            main_len,
            rotary_len,
            main_angle,
            rotary_angle,
        }
    }
}
