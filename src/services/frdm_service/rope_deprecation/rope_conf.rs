use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree}, entity::Name};
///
/// ## The configuration parameters for the rope
/// 
/// ### Example:
/// ```yaml
/// rope:
///     width: 35 mm        # Diameter of the rome
///     length: 3000 m      # Total working length of the rope
///     segment: 100 mm     # Rope segmetn length. Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///     pos: point real 'App/Winch.EncoderBR2'      # meters, current rope position
///     load: point real '/App/Winch.Load'          # tonn, current rope load 
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RopeConf {
    /// Diameter of the rome
    pub width: ConfDistance,
    /// Total working length of the rope
    pub length: ConfDistance,
    /// Rope segmetn length.
    /// Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
    pub segment: ConfDistance,
    pub pos: String,
    pub load: String,
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
        log::trace!("{}.new | name: {:?}", dbg, name);
        let width = conf.get_distance("width").expect(&format!("{dbg}.new | 'width' - not found or wrong configuration"));
        log::trace!("{dbg}.new | width: {:?}", width);
        let length = conf.get_distance("length").expect(&format!("{dbg}.new | 'length' - not found or wrong configuration"));
        log::trace!("{dbg}.new | length: {:?}", length);
        let segment = conf.get_distance("segment").expect(&format!("{dbg}.new | 'segment' - not found or wrong configuration"));
        log::trace!("{dbg}.new | segment: {:?}", segment);
        let pos = conf.get_fn_config(&dbg, "pos", &mut vec![]).unwrap().name();
        log::trace!("{dbg}.new | pos: {:?}", pos);
        let load = conf.get_fn_config(&dbg, "load", &mut vec![]).unwrap().name();
        log::trace!("{dbg}.new | load: {:?}", load);
        Self {
            width,
            length,
            segment,
            pos,
            load,
        }
    }
}
