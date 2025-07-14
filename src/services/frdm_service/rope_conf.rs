use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::{Name, PointConf}, task::functions::FnConfig};
///
/// ## The configuration parameters for the rope
/// 
/// ### Example:
/// ```yaml
/// rope:
///     width: 35 mm        # Diameter of the rome
///     length: 3000 m      # Total working length of the rope
///     segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///     pos: point real 'App/Winch.EncoderBR2'      # meters, current rope position
///     load: point real '/App/Winch.Load'          # tonn, current rope load 
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RopeConf {
    pub width: ConfDistance,
    pub length: ConfDistance,
    pub segment: ConfDistance,
    pub pos: FnConfig,
    pub load: FnConfig,
}
//
// 
impl RopeConf {
    ///
    /// Returns [RopeConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let parent = parent.into();
        let me = "RopeConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, name);

        let width = conf.get_distance("width").unwrap();
        log::debug!("{dbg}.new | width: {:?}", width);

        let length = conf.get_distance("length").unwrap();
        log::debug!("{dbg}.new | length: {:?}", length);

        let segment = conf.get_distance("segment").unwrap();
        log::debug!("{dbg}.new | segment: {:?}", segment);

        let pos: FnConfig = conf.get("pos").unwrap();
        log::debug!("{dbg}.new | pos: {:?}", pos);
        // let pos = pos.input_conf("pos").unwrap();
        // log::debug!("{dbg}.new | pos: {:?}", pos);

        let load: FnConfig = conf.get("load").unwrap();
        log::debug!("{dbg}.new | load: {:?}", load);
        // let pos = pos.input_conf("pos").unwrap();
        // log::debug!("{dbg}.new | pos: {:?}", pos);

        RopeConf {
            width,
            length,
            segment,
            pos,
            load,
        }
    }
}
