use std::ops::Range;

use regex::Regex;
use sal_sync::services::{conf::ConfTree, entity::Name};

///
/// ## The bendingsof the rope
/// 
/// ### Example:
/// ```yaml
/// bendings:
///     - 5.0 .. 5.15
///     - 7.23 .. 7.30
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct BendingsConf {
    pub bendings: Vec<Range<f64>>,
}
//
// 
impl BendingsConf {
    ///
    /// Returns [BendingsConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("RopeConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, name);
        let bend_re = Regex::new(r"");
        let mut bendings: Vec<Range<f64>> = vec![];
        for bend_conf in conf.sub_nodes() {
            let bend = 
        }
        log::debug!("{dbg}.new | bendings: {:?}", bendings);
        Self {
            bendings,
        }
    }
}
