use sal_sync::{
    collections::FxIndexMap,
    services::{conf::{ConfTree, ConfTreeGet, DiagKeywd},
    entity::{Name, PointConf},
    task::functions::{FnConfKeywd, FnConfKindName}, LinkName}
};
use std::{fs, str::FromStr, time::Duration};

///
/// ## Config for `ModbusTcp` format:
/// ```yaml
/// service ModbusTcp FrdmService1:
///     wait-started: 10 ms         # optional, next service will wait until current completely started plus specified time
///     subscribe: Multiqueue
///     send-to: MultiQueue.in-queue
///     description: 'S7-IED-01.01'
///     ip: '192.168.100.243'
///     port: 502
///     cycle: 100 ms
///     diagnosis:                          # internal diagnosis
///         point Status:                   # Ok(0) / Invalid(10)
///             type: 'Int'
///             # history: r
///         point Connection:               # Ok(0) / Invalid(10)
///             type: 'Int'
///             # history: r
///     function:                       # Modbus function code, groups multiple registers
///         point Drive.Speed: 
///             type: 'Real'
///             offset: 0
///```
#[derive(Debug, PartialEq, Clone)]
pub struct ModbusTcpConf {
    pub name: Name,
    /// Next service will wait until current completely started plus specified time, optional
    pub wait_started: Option<Duration>,
    /// Service name to subscribe for incoming Client's commands to be written to the `Device`
    pub subscribe: String,
    /// Name of service to send the data received from the `Device`
    pub send_to: LinkName,
    /// User comment to the `Service`
    pub description: String,
    /// The Device TCP/IP address
    pub ip: String,
    /// The Device TCP/IP port, default 502
    pub port: u64,
    pub diagnosis: FxIndexMap<DiagKeywd, PointConf>,
    /// Defined registers and corresponding `Point`'s
    pub points: Vec<PointConf>,
}
//
// 
impl ModbusTcpConf {
    ///
    /// Returns [ModbusTcpConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("ModbusTcpConf '{}'", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let wait_started: Option<Duration> = conf.get_duration("wait-started").ok();
        log::trace!("{}.new | wait-started: {:?}", dbg, wait_started);
        let subscribe = conf.get("subscribe").expect(&format!("{dbg}.new | 'subscribe' - not found or wrong config"));
        log::trace!("{}.new | subscribe: {:?}", dbg, wait_started);
        let send_to: String = conf.get("send-to").expect(&format!("{dbg}.new | 'send-to' - not found or wrong config"));
        let send_to = LinkName::from_str(&send_to).expect(&format!("{dbg}.new | 'send-to' - wrong config"));
        log::debug!("{}.new | send-to: '{}'", dbg, send_to);
        let description = conf.get("description").expect(&format!("{dbg}.new | 'description' - not found or wrong config"));
        log::trace!("{dbg}.new | description: {:?}", description);
        let ip = conf.get("ip").expect(&format!("{dbg}.new | 'ip' - not found or wrong config"));
        log::trace!("{dbg}.new | ip: {:?}", ip);
        let port = conf.get("port").unwrap_or(502);
        log::trace!("{dbg}.new | port: {:?}", port);
        let diagnosis = conf.get_diagnosis(&name);
        log::trace!("{dbg}.new | diagnosis: {:?}", diagnosis);
        let points = conf.nodes()
            .filter(|node| FnConfKeywd::from_str(&node.key).map_or(false, |keywd| keywd.kind() == FnConfKindName::Point))
            .map(|point| {
                PointConf::new(&name, &point)
            }).collect();
        Self {
            name,
            wait_started,
            subscribe,
            send_to,
            description,
            ip,
            port,
            diagnosis,
            points,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> ModbusTcpConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("ModbusTcpConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> ModbusTcpConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        ModbusTcpConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("ModbusTcpConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("ModbusTcpConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
