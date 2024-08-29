use indexmap::IndexMap;
use log::{debug, error, trace};
use sal_sync::{collections::map::IndexMapFxHasher, services::{conf::conf_tree::ConfTree, entity::{name::Name, point::point_config::PointConfig}, service::link_name::LinkName}};
use std::{fs, str::FromStr, time::Duration};
use crate::conf::{
    conf_keywd::ConfKind, diag_keywd::DiagKeywd,
    service_config::ServiceConfig, udp_client_config::keywd::{self, Keywd},
};

use super::udp_client_db_config::UdpClientDbConfig;
///
/// Creates config from serde_yaml::Value
/// 
/// Example
/// 
/// ```yaml
/// service UdpClient UdpIed01:            # device will be executed in the independent thread, must have unique name
///    subscribe: Multiqueue
///    in queue in-queue:
///        max-length: 10000
///    send-to: MultiQueue.in-queue
///    cycle: 1 ms                         # operating cycle time of the device
///    protocol: 'udp-raw'
///    description: 'UDP-IED-01.01'
///    ip: '192.168.100.241'
///    rack: 0
///    slot: 1
///    diagnosis:                          # internal diagnosis
///        point Status:                   # Ok(0) / Invalid(10)
///            type: 'Int'
///            # history: r
///        point Connection:               # Ok(0) / Invalid(10)
///            type: 'Int'
///            # history: r
/// 
///    db data:                            # multiple DB blocks are allowed, must have unique namewithing parent device
///        description: 'Data block of the device'
///        size: 34
///        point Sensor1: 
///            type: 'Int'
///            input: 1                    # the number of input 1...4
///                 ...
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct UdpClientConfig {
    pub(crate) name: Name,
    pub(crate) cycle: Option<Duration>,
    pub(crate) reconnect_cycle: Duration,
    pub(crate) subscribe: String,
    pub(crate) send_to: LinkName,
    pub(crate) protocol: String,
    pub(crate) description: String,
    pub(crate) ip: String,
    pub(crate) rack: u64,
    pub(crate) slot: u64,
    pub(crate) diagnosis: IndexMapFxHasher<DiagKeywd, PointConfig>,
    pub(crate) dbs: IndexMap<String, UdpClientDbConfig>,
}
//
// 
impl UdpClientConfig {
    ///
    /// Creates new instance of the [UdpClientConfig]:
    pub fn new(parent: impl Into<String>, conf_tree: &mut ConfTree) -> Self {
        println!();
        trace!("UdpClientConfig.new | conf_tree: {:#?}", conf_tree);
        let self_id = format!("UdpClientConfig({})", conf_tree.key);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.clone());
        trace!("{}.new | self_conf: {:?}", self_id, self_conf);
        let self_name = Name::new(parent, self_conf.sufix());
        debug!("{}.new | name: {:?}", self_id, self_name);
        let cycle = self_conf.get_duration("cycle");
        debug!("{}.new | cycle: {:?}", self_id, cycle);
        let reconnect_cycle = self_conf.get_duration("reconnect").map_or(Duration::from_secs(3), |reconnect| reconnect);
        debug!("{}.new | reconnectCycle: {:?}", self_id, reconnect_cycle);
        let subscribe = self_conf.get_param_value("subscribe").unwrap().as_str().unwrap().to_string();
        debug!("{}.new | sudscribe: {:?}", self_id, subscribe);
        let send_to = LinkName::new(self_conf.get_send_to().unwrap()).validate();
        debug!("{}.new | send-to: {}", self_id, send_to);
        if let Ok((_, _)) = self_conf.get_param_by_keyword("out", ConfKind::Queue) {
            error!("{}.new | Parameter 'out queue' - deprecated, use 'send-to' instead in conf: {:#?}", self_id, self_conf)
        }
        let protocol = self_conf.get_param_value("protocol").unwrap().as_str().unwrap().to_string();
        debug!("{}.new | protocol: {:?}", self_id, protocol);
        let description = self_conf.get_param_value("description").unwrap().as_str().unwrap().to_string();
        debug!("{}.new | description: {:?}", self_id, description);
        let ip = self_conf.get_param_value("ip").unwrap().as_str().unwrap().to_string();
        debug!("{}.new | ip: {:?}", self_id, ip);
        let rack = self_conf.get_param_value("rack").unwrap().as_u64().unwrap();
        debug!("{}.new | rack: {:?}", self_id, rack);
        let slot = self_conf.get_param_value("slot").unwrap().as_u64().unwrap();
        debug!("{}.new | slot: {:?}", self_id, slot);
        let diagnosis = self_conf.get_diagnosis(&self_name);
        debug!("{}.new | diagnosis: {:#?}", self_id, diagnosis);
        let mut dbs = IndexMap::new();
        for key in &self_conf.keys {
            let keyword = Keywd::from_str(key).unwrap();
            if keyword.kind() == keywd::Kind::Db {
                let db_name = keyword.name();
                let mut device_conf = self_conf.get(key).unwrap();
                debug!("{}.new | DB '{}'", self_id, db_name);
                trace!("{}.new | DB '{}'   |   conf: {:?}", self_id, db_name, device_conf);
                let node_conf = UdpClientDbConfig::new(&self_name, &db_name, &mut device_conf);
                dbs.insert(
                    db_name,
                    node_conf,
                );
            } else {
                debug!("{}.new | device expected, but found {:?}", self_id, keyword);
            }
        }
        UdpClientConfig {
            name: self_name,
            cycle,
            reconnect_cycle,
            // rx,
            // rx_max_len,
            subscribe,
            send_to,
            protocol,
            description,
            ip,
            rack,
            slot,
            diagnosis,
            dbs
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> UdpClientConfig {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, &mut ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("UdpClientConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> UdpClientConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        UdpClientConfig::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("UdpClientConfig.read | Error in config: {:?}\n\terror: {:#?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("UdpClientConfig.read | File {} reading error: {:#?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        self.dbs
            .iter()
            .fold(vec![], |mut points, (_device_name, device_conf)| {
                points.extend(device_conf.points());
                points
            })
            .into_iter()
            .chain(self.diagnosis.values().cloned())
            .collect()
    }
}
