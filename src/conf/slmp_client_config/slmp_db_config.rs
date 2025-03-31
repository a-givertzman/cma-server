use sal_sync::services::{conf::conf_tree::{ConfTree, ConfTreeGet}, entity::{name::Name, point::point_config::PointConfig}, task::functions::conf::fn_conf_keywd::{FnConfKeywd, FnConfKindName}};
use std::{str::FromStr, time::Duration};
use crate::services::slmp_client::slmp::device_code::DeviceCode;
///
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct SlmpDbConfig {
    pub(crate) name: Name,
    pub(crate) description: String,
    pub(crate) device_code: DeviceCode,
    pub(crate) offset: u32,
    pub(crate) size: u16,
    pub(crate) cycle: Option<Duration>,
    pub(crate) points: Vec<PointConfig>,
}
//
// 
impl SlmpDbConfig {
    ///
    /// Creates new instance of the SlmpDbConfig
    pub fn new(parent: impl Into<String>, name: &str, mut conf: ConfTree) -> Self {
        log::trace!("SlmpDbConfig.new | conf: {:?}", conf);
        let self_conf = conf.clone();
        let self_id = format!("SlmpDbConfig({})", self_conf.key);
        let self_name = Name::new(parent, name);
        log::debug!("{}.new | name: {:?}", self_id, self_name);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{}.new | cycle: {:?}", self_id, cycle);
        let description = conf.get("description").unwrap_or(String::new());
        log::debug!("{}.new | description: {:?}", self_id, description);
        let device_code: String = conf.get("device-code").unwrap();
        let device_code = DeviceCode::from(device_code.as_str());
        log::debug!("{}.new | device-code: {:?}", self_id, device_code);
        let offset: u64 = conf.get("offset").unwrap();
        log::debug!("{}.new | offset: {:?}", self_id, offset);
        let size: u64 = conf.get("size").unwrap();
        log::debug!("{}.new | size: {:?}", self_id, size);
        let mut points = vec![];
        for key in conf.keys(&["cycle", "description", "device-code", "offset", "size"]) {
            let keyword = FnConfKeywd::from_str(&key).unwrap();
            if keyword.kind() == FnConfKindName::Point {
                let point_name = format!("{}/{}", self_name, keyword.data());
                let point_conf = conf.get(key).unwrap();
                log::trace!("{}.new | Point '{}'", self_id, point_name);
                log::trace!("{}.new | Point '{}'   |   conf: {:?}", self_id, point_name, point_conf);
                let node_conf = PointConfig::new(&self_name, &point_conf);
                points.push(
                    node_conf,
                );
            } else {
                log::debug!("{}.new | device expected, but found {:?}", self_id, keyword);
            }
        }
        Self {
            name: self_name,
            description,
            device_code,
            offset: u32::try_from(offset).unwrap(),
            size: u16::try_from(size).unwrap(),
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
