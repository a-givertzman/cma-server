use indexmap::IndexMap;
use sal_sync::{
    collections::FxIndexMap, 
    services::{conf::{ConfKind, ConfTree, ConfTreeGet, DiagKeywd},
        entity::{Name, PointConf}, LinkName
    },
};
use std::{fs, str::FromStr, time::Duration};
use crate::conf::profinet_client_conf::{keywd::{Keywd, Kind}, profinet_db_conf::ProfinetDbConf};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service ProfinetClient Ied01:          # device will be executed in the independent thread, must have unique name
///    subscribe: Multiqueue
///    in queue in-queue:
///        max-length: 10000
///    send-to: MultiQueue.in-queue
///    cycle: 1 ms                         # operating cycle time of the device, default 100 ms
///    reconnect: 1000 ms                  # reconnect timeout when connection is lost
///    protocol: 'profinet'
///    description: 'S7-IED-01.01'
///    ip: '192.168.100.243'
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
pub struct ProfinetClientConf {
    pub(crate) name: Name,
    pub(crate) cycle: Duration,
    pub(crate) reconnect_cycle: Duration,
    pub(crate) subscribe: String,
    pub(crate) send_to: LinkName,
    pub(crate) protocol: String,
    pub(crate) description: String,
    pub(crate) ip: String,
    pub(crate) rack: u64,
    pub(crate) slot: u64,
    pub(crate) diagnosis: FxIndexMap<DiagKeywd, PointConf>,
    pub(crate) dbs: IndexMap<String, ProfinetDbConf>,
}
//
// 
impl ProfinetClientConf {
    ///
    /// Creates new instance of the [ProfinetClientConf]:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("ProfinetClientConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, self_name);
        let cycle = conf.get_duration("cycle").unwrap_or(Duration::from_millis(100));
        log::trace!("{}.new | cycle: {:?}", dbg, cycle);
        let reconnect_cycle = conf.get_duration("reconnect").map_or(Duration::from_secs(3), |reconnect| reconnect);
        log::trace!("{}.new | reconnectCycle: {:?}", dbg, reconnect_cycle);
        let subscribe = conf.get("subscribe").unwrap();
        log::trace!("{}.new | subscribe: {:?}", dbg, subscribe);
        let send_to: String = conf.get("send-to").unwrap();
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::trace!("{}.new | send-to: {}", dbg, send_to);
        if let Ok((_, _)) = conf.get_by_keywd("out", ConfKind::Queue) {
            log::error!("{}.new | Parameter 'out queue' - deprecated, use 'send-to' instead in conf: {:#?}", dbg, conf)
        }
        let protocol = conf.get("protocol").unwrap();
        log::trace!("{}.new | protocol: {:?}", dbg, protocol);
        let description = conf.get("description").unwrap();
        log::trace!("{}.new | description: {:?}", dbg, description);
        let ip = conf.get("ip").unwrap();
        log::trace!("{}.new | ip: {:?}", dbg, ip);
        let rack = conf.get("rack").unwrap();
        log::trace!("{}.new | rack: {:?}", dbg, rack);
        let slot = conf.get("slot").unwrap();
        log::trace!("{}.new | slot: {:?}", dbg, slot);
        let diagnosis = conf.get_diagnosis(&self_name);
        log::trace!("{}.new | diagnosis: {:#?}", dbg, diagnosis.iter().map(|(k, v)| format!("{}: {}", k, v.name)).collect::<Vec<_>>());
        let mut dbs = IndexMap::new();
        for key in conf.keys(&["cycle", "reconnect", "subscribe", "send-to", "protocol", "description", "ip", "rack", "slot", "diagnosis", "wait-started"]) {
            let keyword = Keywd::from_str(&key).unwrap();
            if keyword.kind() == Kind::Db {
                let db_name = keyword.name();
                let device_conf = conf.get(key).unwrap();
                log::trace!("{}.new | DB '{}'", dbg, db_name);
                log::trace!("{}.new | DB '{}'   |   conf: {:?}", dbg, db_name, device_conf);
                let node_conf = ProfinetDbConf::new(&self_name, &db_name, device_conf);
                dbs.insert(
                    db_name,
                    node_conf,
                );
            } else {
                log::warn!("{}.new | device expected, but found {:?}", dbg, keyword);
            }
        }
        ProfinetClientConf {
            name: self_name,
            cycle,
            reconnect_cycle,
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
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> ProfinetClientConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("ProfinetClientConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> ProfinetClientConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        ProfinetClientConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("ProfinetClientConf.read | Error in config: {:?}\n\terror: {:#?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("ProfinetClientConf.read | File {} reading error: {:#?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConf> {
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
