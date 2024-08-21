use indexmap::IndexMap;
use log::{debug, trace};
use std::{fs, str::FromStr, time::Duration};
use crate::{
    conf::{
        conf_tree::ConfTree, diag_keywd::DiagKeywd, point_config::{name::Name, point_config::PointConfig}, service_config::ServiceConfig, slmp_client_config::{keywd::{Keywd, Kind}, slmp_db_config::SlmpDbConfig}
    }, 
    core_::types::map::IndexMapFxHasher, services::queue_name::QueueName,
};
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
    pub(crate) send_to: QueueName,
    pub(crate) description: String,
    pub(crate) ip: String,
    pub(crate) port: u64,
    pub(crate) diagnosis: IndexMapFxHasher<DiagKeywd, PointConfig>,
    pub(crate) dbs: IndexMap<String, SlmpDbConfig>,
}
//
// 
impl SlmpClientConfig {
    ///
    /// Creates new instance of the [SlmpClientConfig]:
    pub fn new(parent: impl Into<String>, conf_tree: &mut ConfTree) -> Self {
        println!();
        trace!("SlmpClientConfig.new | conf_tree: {:#?}", conf_tree);
        let self_id = format!("SlmpClientConfig({})", conf_tree.key);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.clone());
        trace!("{}.new | self_conf: {:?}", self_id, self_conf);
        let self_name = Name::new(parent, self_conf.sufix());
        debug!("{}.new | name: {:?}", self_id, self_name);
        let cycle = self_conf.get_duration("cycle");
        debug!("{}.new | cycle: {:?}", self_id, cycle);
        let reconnect_cycle = self_conf.get_duration("reconnect").map_or(Duration::from_secs(1), |reconnect| reconnect);
        debug!("{}.new | reconnectCycle: {:?}", self_id, reconnect_cycle);
        let subscribe = self_conf.get_param_value("subscribe").unwrap().as_str().unwrap().to_string();
        debug!("{}.new | sudscribe: {:?}", self_id, subscribe);
        let send_to = QueueName::new(self_conf.get_send_to().unwrap());
        debug!("{}.new | send-to: '{}'", self_id, send_to);
        let description = self_conf.get_param_value("description").unwrap().as_str().unwrap().to_string();
        debug!("{}.new | description: {:?}", self_id, description);
        let ip = self_conf.get_param_value("ip").unwrap().as_str().unwrap().to_string();
        debug!("{}.new | ip: {:?}", self_id, ip);
        let port = self_conf.get_param_value("port").unwrap().as_u64().unwrap();
        debug!("{}.new | port: {:?}", self_id, ip);
        let diagnosis = self_conf.get_diagnosis(&self_name);
        debug!("{}.new | diagnosis: {:#?}", self_id, diagnosis);
        let mut dbs = IndexMap::new();
        for key in &self_conf.keys {
            let keyword = Keywd::from_str(key).unwrap();
            if keyword.kind() == Kind::Db {
                let db_name = keyword.name();
                let mut device_conf = self_conf.get(key).unwrap();
                debug!("{}.new | DB '{}'", self_id, db_name);
                trace!("{}.new | DB '{}'   |   conf: {:?}", self_id, db_name, device_conf);
                let node_conf = SlmpDbConfig::new(&self_name, &db_name, &mut device_conf);
                dbs.insert(
                    db_name,
                    node_conf,
                );
            } else {
                debug!("{}.new | device expected, but found {:?}", self_id, keyword);
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
                Self::new(parent, &mut ConfTree::new(key.as_str().unwrap(), value.clone()))
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
