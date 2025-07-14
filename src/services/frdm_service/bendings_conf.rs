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
    pub fn new(parent: impl Into<String>, conf: Vec<serde_yaml::Value>) -> Self {
        let parent = parent.into();
        let me = "RopeConf";
        let dbg = Dbg::new(&parent, me);
        log::debug!("{dbg}.new | conf: {:?}", conf);
        let bend_re = Regex::new(r"^([-+]?\d[\d]*\.?[\d]+)[ \t]*\.\.[ \t]*([-+]?\d[\d]*\.?[\d]+)[ \t]*(nm|um|cm|mm|m|km|in)$").unwrap();
        // let bendings: serde_yaml::Value = conf.get("bendings").unwrap();
        // let bendings = bendings.as_sequence().expect(&format!("{dbg}.new | Wrong bending: {:?}, Expected list of items: string: start..end unit (0.5..0.8 m)", bendings));
        let bendings = conf.iter().filter_map(|bend| {
            match bend.as_str() {
                Some(bend) => {
                    match bend_re.captures(bend) {
                        Some(caps) => {
                            let start = caps.get(1).expect(&format!("{dbg}.new | Wrong bending: {:?}", bend)).as_str();
                            let end = caps.get(2).expect(&format!("{dbg}.new | Wrong bending: {:?}", bend)).as_str();
                            let unit = caps.get(3).expect(&format!("{dbg}.new | Unit is missing in the bending: {:?}", bend)).as_str();
                            let start = ConfDistance::from_str(&format!("{start} {unit}")).expect(&format!("{dbg}.new | Wrong float or unit in the bending: {:?}", bend));
                            let end = ConfDistance::from_str(&format!("{end} {unit}")).expect(&format!("{dbg}.new | Wrong float or unit in the bending: {:?}", bend));
                            Some(start.as_m()..end.as_m())
                        }
                        None => panic!("{dbg}.new | Wrong bending: {:?}", bend),
                    }
                }
                None => panic!("{dbg}.new | Wrong bending: {:?}, Expected string: start..end unit (0.5..0.8 m)", bend),
            }
        }).collect();
        log::debug!("{dbg}.new | bendings: {:?}", bendings);
        Self {
            bendings,
        }
    }
}
