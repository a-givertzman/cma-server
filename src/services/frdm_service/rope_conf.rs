use std::str::FromStr;

use sal_core::dbg::Dbg;
use sal_sync::services::{conf::{ConfDistance, ConfTree, ConfTreeGet}, entity::Name, task::functions::FnConfKind, LinkName};

use crate::services::BendingsConf;
///
/// ## The configuration parameters for the rope
/// 
/// ### Example:
/// ```yaml
/// rope:
///     width: 35 mm        # Diameter of the rome
///     length: 3000 m      # Total working length of the rope
///     segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
///     bendings:           # Rope bloks with diameter, inter and exit
///           Block Diameter   inter   exit
///         - D200mm           5.0  .. 5.15 m
///         - D300mm           7.23 .. 7.30 mm
///     pos: point real 'App/Winch.EncoderBR2'      # meters, current rope position
///     load: point real '/App/Winch.Load'          # tonn, current rope load 
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RopeConf {
    pub width: ConfDistance,
    pub length: ConfDistance,
    pub segment: ConfDistance,
    pub bendings: BendingsConf,
    pub pos: LinkName,
    pub load: LinkName,
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
        let bendings = conf.get("bendings").expect(&format!("{dbg}.new | 'bendings' - not found or wrong configuration"));
        let bendings = BendingsConf::new(&name, bendings);
        log::debug!("{dbg}.new | bendings: {:#?}", bendings);
        let pos = LinkName::from_str(&conf.get_fn_config(&dbg, "pos", &mut vec![]).unwrap().name()).unwrap();
        log::debug!("{dbg}.new | pos: {:?}", pos);
        let load = LinkName::from_str(&conf.get_fn_config(&dbg, "load", &mut vec![]).unwrap().name()).unwrap();
        log::debug!("{dbg}.new | load: {:?}", load);
        Self {
            width,
            length,
            segment,
            bendings,
            pos,
            load,
        }
    }
    ///
    /// Return a type of 
    fn type_(conf: &FnConfKind) -> String {
        match conf {
            FnConfKind::Fn(conf) => format!("{:?}", conf.type_),
            FnConfKind::Var(conf) => format!("{:?}", conf.type_),
            FnConfKind::Const(conf) => format!("{:?}", conf.type_),
            FnConfKind::Point(conf) => format!("{:?}", conf.type_),
            FnConfKind::PointConf(conf) => format!("{:?}", conf.conf.type_),
            FnConfKind::Param(conf) => format!("{:?}", conf),
        }
    }
}
