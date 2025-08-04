use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree}, entity::Name};
///
/// ## The configuration parameters for the crane's boom
/// 
/// ### Example:
/// ```yaml
/// boom:
///     main-len: 5.3 m                                        # length of the main boom
///     main-angle: point real 'App/Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
///     rotary-len: 2.1 m                                      # length of the rotary boom
///     rotary-angle: point real 'App/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BoomConf {
    pub main_len: ConfDistance,
    pub main_angle: String,
    pub rotary_len: ConfDistance,
    pub rotary_angle: String,
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
        let main_len = conf.get_distance("main-len").expect(&format!("{dbg}.new | 'main-len' - not found or wrong configuration"));
        log::debug!("{dbg}.new | main_len: {:?}", main_len);
        let rotary_len = conf.get_distance("rotary-len").expect(&format!("{dbg}.new | 'rotary-len' - not found or wrong configuration"));
        log::debug!("{dbg}.new | rotary_len: {:?}", rotary_len);
        let main_angle = conf.get_fn_config(&dbg, "main-angle", &mut vec![]).unwrap().name();
        log::debug!("{dbg}.new | main_angle: {:?}", main_angle);
        let rotary_angle = conf.get_fn_config(&dbg, "rotary-angle", &mut vec![]).unwrap().name();
        log::debug!("{dbg}.new | rotary_angle: {:?}", rotary_angle);
        Self {
            main_len,
            rotary_len,
            main_angle,
            rotary_angle,
        }
    }
}
