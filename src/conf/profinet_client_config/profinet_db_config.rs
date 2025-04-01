use sal_sync::services::{conf::conf_tree::{ConfTree, ConfTreeGet}, entity::{name::Name, point::point_config::PointConfig}, task::functions::conf::fn_conf_keywd::{FnConfKeywd, FnConfKindName}};
use std::str::FromStr;
///
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct ProfinetDbConfig {
    pub(crate) name: Name,
    pub(crate) description: String,
    pub(crate) number: u64,
    pub(crate) offset: u64,
    pub(crate) size: u64,
    pub(crate) points: Vec<PointConfig>,
}
//
// 
impl ProfinetDbConfig {
    ///
    /// Creates new instance of the ProfinetDbConfig
    pub fn new(parent: impl Into<String>, name: &str, conf: ConfTree) -> Self {
        log::trace!("ProfinetDbConfig.new | conf: {:?}", conf);
        let dbg = format!("ProfinetDbConfig({})", name);
        let self_name = Name::new(parent, name);
        log::debug!("{}.new | name: {:?}", dbg, self_name);
        let description = conf.get("description").unwrap_or(String::new());
        log::debug!("{}.new | description: {:?}", dbg, description);
        let number = conf.get("number").unwrap();
        log::debug!("{}.new | number: {:?}", dbg, number);
        let offset = conf.get("offset").unwrap();
        log::debug!("{}.new | offset: {:?}", dbg, offset);
        let size = conf.get("size").unwrap();
        log::debug!("{}.new | size: {:?}", dbg, size);
        let mut points = vec![];
        for key in conf.keys(&["description", "number", "offset", "size"]) {
            let keyword = FnConfKeywd::from_str(&key).unwrap();
            if keyword.kind() == FnConfKindName::Point {
                let point_name = format!("{}/{}", self_name, keyword.data());
                let point_conf = conf.get(key).unwrap();
                log::trace!("{}.new | Point '{}'", dbg, point_name);
                log::trace!("{}.new | Point '{}'   |   conf: {:?}", dbg, point_name, point_conf);
                let node_conf = PointConfig::new(&self_name, &point_conf);
                points.push(
                    node_conf,
                );
            } else {
                log::debug!("{}.new | device expected, but found {:?}", dbg, keyword);
            }
        }
        Self {
            name: self_name,
            description,
            number,
            offset,
            size,
            points,
        }
    }    
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        self.points.iter().fold(vec![], |mut points, conf| {
            points.push(conf.clone());
            points
        })
    }
}