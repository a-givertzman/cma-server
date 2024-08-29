use log::{debug, trace};
use sal_sync::services::{conf::conf_tree::ConfTree, entity::{name::Name, point::point_config::PointConfig}};
use std::{str::FromStr, time::Duration};
use crate::conf::{
    fn_::fn_conf_keywd::{FnConfKeywd, FnConfKindName}, 
    service_config::ServiceConfig,
};
///
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct UdpClientDbConfig {
    pub(crate) name: Name,
    pub(crate) description: String,
    pub(crate) number: u64,
    pub(crate) offset: u64,
    pub(crate) size: u64,
    pub(crate) cycle: Option<Duration>,
    pub(crate) points: Vec<PointConfig>,
}
//
// 
impl UdpClientDbConfig {
    ///
    /// Creates new instance of the UdpClientDbConfig
    pub fn new(parent: impl Into<String>, name: &str, conf_tree: &mut ConfTree) -> Self {
        trace!("UdpClientDbConfig.new | confTree: {:?}", conf_tree);
        let self_conf = conf_tree.clone();
        let self_id = format!("UdpClientDbConfig({})", self_conf.key);
        let mut self_conf = ServiceConfig::new(&self_id, self_conf);
        trace!("{}.new | self_conf: {:?}", self_id, self_conf);
        let self_name = Name::new(parent, name);
        debug!("{}.new | name: {:?}", self_id, self_name);
        let cycle = self_conf.get_duration("cycle");
        debug!("{}.new | cycle: {:?}", self_id, cycle);
        let description = self_conf.get_param_value("description").unwrap_or(serde_yaml::Value::String(String::new())).as_str().unwrap().to_string();
        debug!("{}.new | description: {:?}", self_id, description);
        let number = self_conf.get_param_value("number").unwrap().as_u64().unwrap();
        debug!("{}.new | number: {:?}", self_id, number);
        let offset = self_conf.get_param_value("offset").unwrap().as_u64().unwrap();
        debug!("{}.new | offset: {:?}", self_id, offset);
        let size = self_conf.get_param_value("size").unwrap().as_u64().unwrap();
        debug!("{}.new | size: {:?}", self_id, size);
        let mut points = vec![];
        for key in &self_conf.keys {
            let keyword = FnConfKeywd::from_str(key).unwrap();
            if keyword.kind() == FnConfKindName::Point {
                let point_name = format!("{}/{}", self_name, keyword.data());
                let point_conf = self_conf.get(key).unwrap();
                trace!("{}.new | Point '{}'", self_id, point_name);
                trace!("{}.new | Point '{}'   |   conf: {:?}", self_id, point_name, point_conf);
                let node_conf = PointConfig::new(&self_name, &point_conf);
                points.push(
                    node_conf,
                );
            } else {
                debug!("{}.new | device expected, but found {:?}", self_id, keyword);
            }
        }
        Self {
            name: self_name,
            description,
            number,
            offset,
            size,
            cycle,
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