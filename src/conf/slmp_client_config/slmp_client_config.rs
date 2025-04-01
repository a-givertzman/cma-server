use indexmap::IndexMap;
use sal_sync::{collections::map::FxIndexMap, services::{conf::{conf_tree::{ConfTree, ConfTreeGet}, diag_keywd::DiagKeywd}, entity::{name::Name, point::point_config::PointConfig}, service::link_name::LinkName}};
use std::{fs, str::FromStr, time::Duration};
use crate::conf::slmp_client_config::{keywd::{Keywd, Kind}, slmp_db_config::SlmpDbConfig};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service ProfinetClient Ied01:          # device will be executed in the independent thread, must have unique name
///    subscribe: Multiqueue
///    send-to: MultiQueue.in-queue
///    cycle: 1 ms                     # operating cycle time of the device
///    description: 'S7-IED-01.01'
///    ip: '192.168.100.243'
///    diagnosis:                          # internal diagnosis
///        point Status:                   # Ok(0) / Invalid(10)
///            type: 'Int'
///            # history: r
///        point Connection:               # Ok(0) / Invalid(10)
///            type: 'Int'
///            # history: r
/// 
///    db db899:                       # multiple DB blocks are allowed, must have unique namewithing parent device
///        description: 'db899 | Exhibit - drive data'
///        number: 899
///        offset: 0
///        size: 34
///        point Drive.Speed: 
///            type: 'Real'
///            offset: 0
///                 ...
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct SlmpClientConfig {
    pub(crate) name: Name,
    pub(crate) cycle: Option<Duration>,
    pub(crate) reconnect_cycle: Duration,
    pub(crate) subscribe: String,
    pub(crate) send_to: LinkName,
    pub(crate) description: String,
    pub(crate) ip: String,
    pub(crate) port: u64,
    pub(crate) diagnosis: FxIndexMap<DiagKeywd, PointConfig>,
    pub(crate) dbs: IndexMap<String, SlmpDbConfig>,
}
//
// 
impl SlmpClientConfig {
    ///
    /// Creates new instance of the [SlmpClientConfig]:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("SlmpClientConfig({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, self_name);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{}.new | cycle: {:?}", dbg, cycle);
        let reconnect_cycle = conf.get_duration("reconnect").map_or(Duration::from_secs(1), |reconnect| reconnect);
        log::debug!("{}.new | reconnect: {:?}", dbg, reconnect_cycle);
        let subscribe = conf.get("subscribe").unwrap();
        log::debug!("{}.new | subscribe: {:?}", dbg, subscribe);
        let send_to: String = conf.get("send-to").unwrap();
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::debug!("{}.new | send-to: '{}'", dbg, send_to);
        let description = conf.get("description").unwrap();
        log::debug!("{}.new | description: {:?}", dbg, description);
        let ip = conf.get("ip").unwrap();
        log::debug!("{}.new | ip: {:?}", dbg, ip);
        let port = conf.get("port").unwrap();
        log::debug!("{}.new | port: {:?}", dbg, ip);
        let diagnosis = conf.get_diagnosis(&self_name);
        log::debug!("{}.new | diagnosis: {:#?}", dbg, diagnosis);
        let mut dbs = IndexMap::new();
        for key in conf.keys(&["cycle", "reconnect", "subscribe", "send-to", "description", "ip", "port", "diagnosis"]) {
            let keyword = Keywd::from_str(&key).unwrap();
            if keyword.kind() == Kind::Db {
                let db_name = keyword.name();
                let device_conf = conf.get(key).unwrap();
                log::debug!("{}.new | DB '{}'", dbg, db_name);
                log::trace!("{}.new | DB '{}'   |   conf: {:?}", dbg, db_name, device_conf);
                let node_conf = SlmpDbConfig::new(&self_name, &db_name, device_conf);
                dbs.insert(
                    db_name,
                    node_conf,
                );
            } else {
                log::debug!("{}.new | device expected, but found {:?}", dbg, keyword);
            }
        }
        SlmpClientConfig {
            name: self_name,
            cycle,
            reconnect_cycle,
            subscribe,
            send_to,
            description,
            ip,
            port,
            diagnosis,
            dbs
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> SlmpClientConfig {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("SlmpClientConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> SlmpClientConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        SlmpClientConfig::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("SlmpClientConfig.read | Error in config: {:?}\n\terror: {:#?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("SlmpClientConfig.read | File {} reading error: {:#?}", path, err)
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
