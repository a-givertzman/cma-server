use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use crate::{infra::ApiClientConf, services::frdm_service::CraneConf};

///
/// ## Config for RopeDeprecation
/// 
/// ### Conf example
/// ```yaml
/// rope-deprecation:
///     table: 'public.frdm_deprecation'
///     subscribe: MultiQueue                                          # Service name, to subscribe for rope positin and crane angles event's
///     crane:
///         bendings:           # Rope bloks with diameter, inter and exit
///             # Block Diameter   inter   exit
///             - D200mm           5.0  .. 5.15 m
///             - D300mm           7.23 .. 7.30 mm
///         boom:
///             main-len: 5.3 m                                         # length of the main boom
///             main-angle: point real 'App/MultiQueue/Load.MainBoomAngle'        # degrees, current angle of the main boom to horisontal axis
///             rotary-len: 2.1 m                                       # length of the rotary boom
///             rotary-angle: point real 'App/MultiQueue/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to horisontal axis
///         rope:
///             width: 35 mm        # Diameter of the rome
///             length: 3000 m      # Total working length of the rope
///             segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///             pos: point real 'App/MultiQueue/Winch.EncoderBR2'      # meters, current rope position
///             load: point real 'App/MultiQueue/Winch.Load'           # tonn, current rope load
/// ```
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
