use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use sal_sync::{collections::map::IndexMapFxHasher, services::{conf::conf_tree::ConfTree, entity::{name::Name, point::point_config::PointConfig}, service::link_name::LinkName, subscription::conf_subscribe::ConfSubscribe}};
use std::{fs, hash::BuildHasherDefault, str::FromStr, time::Duration};
use crate::conf::{
    diag_keywd::DiagKeywd,
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
///    description: 'UDP-IED-01.01'
///    subscribe: Multiqueue
///    send-to: MultiQueue.in-queue
///    cycle: 1 ms                         # operating cycle time of the device
///    reconnect: 1000 ms                  # reconnect timeout when connection is lost
///    protocol: 'udp-raw'
///    local-address: 192.168.100.100:15180
///    remote-address: 192.168.100.241:15180
///    mtu: 1500                           # Maximum Transmission Unit, default 1500
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
///        size: 1024                      # corresponding to the length of the array transmitted in the UDP message
///        point Sensor1: 
///            type: 'Int'
///            input: 0                    # the number of input 0..8 (0 - first input channel)
///                 ...
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct UdpClientConfig {
    pub name: Name,
    pub description: String,
    pub subscribe: ConfSubscribe,
    pub send_to: LinkName,
    pub cycle: Option<Duration>,
    pub reconnect: Duration,
    pub protocol: String,
    pub local_addr: String,
    pub remote_addr: String,
    /// Maximum Transmission Unit, default 1500, [Resolve IPv4 Fragmentation, MTU...](https://www.cisco.com/c/en/us/support/docs/ip/generic-routing-encapsulation-gre/25885-pmtud-ipfrag.html)
    pub mtu: usize,
    pub diagnosis: IndexMapFxHasher<DiagKeywd, PointConfig>,
    pub dbs: IndexMapFxHasher<String, UdpClientDbConfig>,
}
//
// 
impl UdpClientConfig {
    ///
    /// Creates new instance of the [UdpClientConfig]:
    pub fn new(parent: impl Into<String>, conf_tree: &mut ConfTree) -> Self {
        println!();
        log::trace!("UdpClientConfig.new | conf_tree: {:#?}", conf_tree);
        let self_id = format!("UdpClientConfig({})", conf_tree.key);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.clone());
        log::trace!("{}.new | self_conf: {:?}", self_id, self_conf);
        let sufix = self_conf.sufix();
        let self_name = Name::new(parent, if sufix.is_empty() {self_conf.name()} else {sufix});
        log::debug!("{}.new | name: {:?}", self_id, self_name);
        let description = self_conf.get_param_value("description").unwrap().as_str().unwrap().to_string();
        log::debug!("{}.new | description: {:?}", self_id, description);
        let subscribe = ConfSubscribe::new(self_conf.get_param_value("subscribe").unwrap_or(serde_yaml::Value::Null));
        log::debug!("{}.new | sudscribe: {:?}", self_id, subscribe);
        let send_to = LinkName::from_str(self_conf.get_send_to().unwrap().as_str()).unwrap();
        log::debug!("{}.new | send-to: {}", self_id, send_to);
        let cycle = self_conf.get_duration("cycle");
        log::debug!("{}.new | cycle: {:?}", self_id, cycle);
        let reconnect = self_conf.get_duration("reconnect").map_or(Duration::from_secs(3), |reconnect| reconnect);
        log::debug!("{}.new | reconnect: {:?}", self_id, reconnect);
        let protocol = self_conf.get_param_value("protocol").unwrap().as_str().unwrap().to_string();
        log::debug!("{}.new | protocol: {:?}", self_id, protocol);
        let local_address = self_conf.get_param_value("local-address").unwrap().as_str().unwrap().to_string();
        log::debug!("{}.new | local-address: {:?}", self_id, local_address);
        let remote_address = self_conf.get_param_value("remote-address").unwrap().as_str().unwrap().to_string();
        log::debug!("{}.new | remote-address: {:?}", self_id, remote_address);
        let mtu = self_conf.get_param_value("mtu");
        log::debug!("{}.new | mtu: {:?}", self_id, mtu);
        let diagnosis = self_conf.get_diagnosis(&self_name);
        log::debug!("{}.new | diagnosis: {:#?}", self_id, diagnosis);
        let mut dbs = IndexMap::with_hasher(BuildHasherDefault::<FxHasher>::default());
        for key in &self_conf.keys {
            let keyword = Keywd::from_str(key).unwrap();
            if keyword.kind() == keywd::Kind::Db {
                let db_name = keyword.name();
                let mut device_conf = self_conf.get(key).unwrap();
                log::debug!("{}.new | DB '{}'", self_id, db_name);
                log::trace!("{}.new | DB '{}'   |   conf: {:?}", self_id, db_name, device_conf);
                let node_conf = UdpClientDbConfig::new(&self_name, &db_name, &mut device_conf);
                dbs.insert(
                    db_name,
                    node_conf,
                );
            } else {
                log::debug!("{}.new | device expected, but found {:?}", self_id, keyword);
            }
        }
        UdpClientConfig {
            name: self_name,
            description,
            subscribe,
            send_to,
            cycle,
            reconnect,
            protocol,
            local_addr: local_address,
            remote_addr: remote_address,
            mtu: mtu.unwrap_or(serde_yaml::Value::Null).as_u64().unwrap_or(1500) as usize,
            diagnosis,
            dbs,
        }
    }
    ///
    /// Returns config build from serde_yaml::Value
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
    /// Returns config build from path
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
