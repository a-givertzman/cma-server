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
    pub l1: ConfDistance,
    pub l2: ConfDistance,
    pub l3: ConfDistance,
    pub l4: ConfDistance,
    pub len: ConfDistance,
    pub angle: String,
    // pub main_len: ConfDistance,
    // pub main_angle: String,
    // pub rotary_len: ConfDistance,
    // pub rotary_angle: String,
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
        log::trace!("{}.new | name: {:?}", dbg, name);
        let l1 = conf.get_distance("l1").expect(&format!("{dbg}.new | 'l1' - not found or wrong config"));
        let l2 = conf.get_distance("l2").expect(&format!("{dbg}.new | 'l2' - not found or wrong config"));
        let l3 = conf.get_distance("l3").expect(&format!("{dbg}.new | 'l3' - not found or wrong config"));
        let l4 = conf.get_distance("l4").expect(&format!("{dbg}.new | 'l4' - not found or wrong config"));
        let len = conf.get_distance("len").expect(&format!("{dbg}.new | 'len' - not found or wrong config"));
        let angle = conf.get_fn_config(&dbg, "angle", &mut vec![]).expect(&format!("{dbg}.new | 'angle' - not found or wrong config")).name();
        // let main_len = conf.get_distance("main-len").expect(&format!("{dbg}.new | 'main-len' - not found or wrong config"));
        // log::trace!("{dbg}.new | main-len: {:?}", main_len);
        // let rotary_len = conf.get_distance("rotary-len").expect(&format!("{dbg}.new | 'rotary-len' - not found or wrong config"));
        // log::trace!("{dbg}.new | rotary-len: {:?}", rotary_len);
        // let main_angle = conf.get_fn_config(&dbg, "main-angle", &mut vec![]).unwrap().name();
        // log::trace!("{dbg}.new | main-angle: {:?}", main_angle);
        // let rotary_angle = conf.get_fn_config(&dbg, "rotary-angle", &mut vec![]).unwrap().name();
        // log::trace!("{dbg}.new | rotary-angle: {:?}", rotary_angle);
        Self {
            l1,
            l2,
            l3,
            l4,
            len,
            angle,
            // main_len: todo!("To be removed"),
            // rotary_len: todo!("To be removed"),
            // main_angle: todo!("To be removed"),
            // rotary_angle: todo!("To be removed"),
        }
    }
}
