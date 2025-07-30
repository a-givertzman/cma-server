use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use crate::services::{BendingsConf, BoomConf, RopeConf};
///
/// ## The configuration parameters for the rope
/// 
/// ### Example:
/// ```yaml
/// crane:
///     bendings:           # Rope bloks with diameter, inter and exit
///         # Block Diameter   inter   exit
///         - D200mm           5.0  .. 5.15 m
///         - D300mm           7.23 .. 7.30 mm
///     boom:
///         main-len: 5.3 m                                        # length of the main boom
///         main-angle: point real 'App/Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
///         rotary-len: 2.1 m                                      # length of the rotary boom
///         rotary-angle: point real 'App/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
///     rope:
///         width: 35 mm        # Diameter of the rome
///         length: 3000 m      # Total working length of the rope
///         segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///         pos: point real 'App/Winch.EncoderBR2'      # meters, current rope position
///         load: point real '/App/Winch.Load'          # tonn, current rope load 
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
        log::trace!("{dbg}.new | bendings: {:#?}", bendings);
        let boom = conf.get("boom").expect(&format!("{dbg}.new | 'boom' - not found or wrong configuration"));
        let boom = BoomConf::new(&name, boom);
        log::trace!("{dbg}.new | boom: {:#?}", boom);
        let rope = conf.get("rope").expect(&format!("{dbg}.new | 'rope' - not found or wrong configuration"));
        let rope = RopeConf::new(&name, rope);
        log::trace!("{dbg}.new | rope: {:#?}", rope);
        Self {
            bendings,
            boom,
            rope,
        }
    }
}
