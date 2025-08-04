use std::{ops::Range, str::FromStr};
use regex::Regex;
use sal_core::dbg::Dbg;
use sal_sync::services::conf::ConfDistance;

///
/// ## The bendingsof the rope
/// 
/// ### Example:
/// ```yaml
/// bendings:
///       Block Diameter   inter   exit
///     - D200mm           5.0  .. 5.15 m
///     - D300mm           7.23 .. 7.30 mm
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct BendingsConf {
    /// Collection of (Block diameter, enter..exit)
    pub bendings: Vec<(ConfDistance, Range<f64>)>,
}
//
// 
impl BendingsConf {
    ///
    /// Returns [BendingsConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: Vec<serde_yaml::Value>) -> Self {
        let parent = parent.into();
        let me = "RopeConf";
        let dbg = Dbg::new(&parent, me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let bend_re = Regex::new(r"^D(\d[\d]*\.?[\d]+[ \t]*(?:nm|um|cm|mm|m|km|in))[ \t]+([-+]?\d[\d]*\.?[\d]+)[ \t]*\.\.[ \t]*([-+]?\d[\d]*\.?[\d]+)[ \t]*(nm|um|cm|mm|m|km|in)$").unwrap();
        let bendings = conf.iter().filter_map(|bend| {
            match bend.as_str() {
                Some(bend) => {
                    match bend_re.captures(bend) {
                        Some(caps) => {
                            let diameter = caps.get(1).expect(&format!("{dbg}.new | Wrong 'diameter', expected format: 'D200.0mm' in the: {:?}", bend)).as_str();
                            let start = caps.get(2).expect(&format!("{dbg}.new | Wrong bending, expected format: 'D200mm 5.0  .. 5.15m' in the: {:?}", bend)).as_str();
                            let end = caps.get(3).expect(&format!("{dbg}.new | Wrong bending, expected format: 'D200mm 5.0  .. 5.15m' in the: {:?}", bend)).as_str();
                            let unit = caps.get(4).expect(&format!("{dbg}.new | Unit is missing in the bending: {:?}", bend)).as_str();
                            let diameter = ConfDistance::from_str(diameter).expect(&format!("{dbg}.new | Wrong 'diameter' float or unit in the: {:?}", bend));
                            let start = ConfDistance::from_str(&format!("{start} {unit}")).expect(&format!("{dbg}.new | Wrong 'start' float or unit in the: {:?}", bend));
                            let end = ConfDistance::from_str(&format!("{end} {unit}")).expect(&format!("{dbg}.new | Wrong 'end' float or unit in the: {:?}", bend));
                            Some((diameter, start.as_m()..end.as_m()))
                        }
                        None => panic!("{dbg}.new | Wrong bending: {:?}", bend),
                    }
                }
                None => panic!("{dbg}.new | Wrong bending: {:?}, Expected string: start..end unit (0.5..0.8 m)", bend),
            }
        }).collect();
        log::trace!("{dbg}.new | bendings: {:?}", bendings);
        Self {
            bendings,
        }
    }
    // ///
    // /// Returns the number of bendings in the collection
    // pub fn len(&self) -> usize {
    //     self.bendings.len()
    // }
}
