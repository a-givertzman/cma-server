use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfDistanceUnit, ConfTree}, entity::Name};
///
/// ## The configuration parameters for the rope
///
/// ### Example:
/// ```yaml
/// rope:
///     width: 35 mm            # Diameter of the rome
///     length: 3000 m          # Total working length of the rope
///     aux-length: 1.200 m     # Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
///     segment: 100 mm         # Rope segmetn length. Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///     pos: point real 'Winch.EncoderBR2'      # meters, current rope position (длина каната размотанного с барабана считая от парковочного)
///     load: point real 'Winch.Load'           # tonn, current rope load
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RopeConf {
    /// Diameter of the rome
    pub width: ConfDistance,
    /// Total working length of the rope
    pub length: ConfDistance,
    /// Auxiliary whip line. Length of the rope from the last block located on the end of last boom to the hook
    pub aux_length: ConfDistance,
    // /// Length of the rope on the winch drum in the parking position, when rope pos is zero
    // pub winch_len: ConfDistance,
    /// Rope segmetn length.
    /// Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
    pub segment: ConfDistance,
    /// Name of input event of current rope position (длина каната размотанного с барабана считая от парковочного), meters
    pub pos: String,
    /// Name of input event of current rope load, tonn
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
        let width = conf.get_distance("width").unwrap_or_else(|_| panic!("{dbg}.new | 'width' - not found or wrong configuration"));
        log::trace!("{dbg}.new | width: {:?}", width);
        let length = conf.get_distance("length").unwrap_or_else(|_| panic!("{dbg}.new | 'length' - not found or wrong configuration"));
        log::trace!("{dbg}.new | length: {:?}", length);
        let aux_length = conf.get_distance("aux-length").unwrap_or_else(|_| panic!("{dbg}.new | 'aux-length' - not found or wrong configuration"));
        log::trace!("{dbg}.new | aux-length: {:?}", aux_length);
        // let winch_len = conf.get_distance("winch-length").unwrap_or_else(|_| panic!("{dbg}.new | 'winch-length' - not found or wrong configuration"));
        // log::trace!("{dbg}.new | winch-length: {:?}", winch_len);
        let segment = conf.get_distance("segment").unwrap_or_else(|_| panic!("{dbg}.new | 'segment' - not found or wrong configuration"));
        let min = ConfDistance::new((0.5 * width.as_mm()).round(), ConfDistanceUnit::Millimeter);
        let max = ConfDistance::new((7.0 * width.as_mm()).round(), ConfDistanceUnit::Millimeter);
        if segment.as_mm() < min.as_mm() || segment.as_mm() > max.as_mm() {
            panic!("{dbg}.new | 'segment' = {:?} - Choose proper segment size: {:?} ... {:?}", segment, min, max)
        }
        log::trace!("{dbg}.new | segment: {:?}", segment);
        let pos = conf.get_fn_config(&dbg, "pos", &mut vec![]).unwrap().name();
        log::trace!("{dbg}.new | pos: {:?}", pos);
        let load = conf.get_fn_config(&dbg, "load", &mut vec![]).unwrap().name();
        log::trace!("{dbg}.new | load: {:?}", load);
        Self {
            width,
            length,
            aux_length,
            // winch_len,
            segment,
            pos,
            load,
        }
    }
}
//
//
impl Default for RopeConf {
    fn default() -> Self {
        Self {
            width: Default::default(),
            length: Default::default(),
            aux_length: Default::default(),
            segment: Default::default(),
            pos: Default::default(),
            load: Default::default(),
        }
    }
}
