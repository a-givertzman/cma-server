use sal_sync::services::{conf::conf_tree::{ConfTree, ConfTreeGet}, entity::name::Name};
use std::{fs, time::Duration, net::SocketAddr};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service ApiClient:
///     cycle: 1 ms
///     reconnect: 1 s  # default 3 s
///     address: 127.0.0.1:8080
///     in queue api-link:
///         max-length: 10000
///     debug: false                # API debug mode, optional, default false
///                         ...
#[derive(Debug, PartialEq, Clone)]
pub struct ApiClientConfig {
    pub(crate) name: Name,
    pub(crate) address: SocketAddr,
    pub(crate) database: String,
    pub(crate) auth_token: String,
    pub(crate) cycle: Option<Duration>,
    pub(crate) reconnect_cycle: Option<Duration>,
    pub(crate) rx: String,
    pub(crate) rx_max_len: i64,
    pub(crate) debug: bool,
}
//
// 
impl ApiClientConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service ApiClient:
    ///     cycle: 1 ms
    ///     reconnect: 1 s  # default 3 s
    ///     address: 127.0.0.1:8080
    ///     in queue api-link:
    ///         max-length: 10000
    ///     debug: false                # API debug mode, optional, default false
    ///                     ...
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("ApiClientConfig({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, self_name);
        let address: String = conf.get("address").unwrap();
        let address: SocketAddr = address.parse().unwrap();
        log::debug!("{}.new | address: {:?}", dbg, address);
        let database = conf.get("database").unwrap();
        log::debug!("{}.new | database: {:?}", dbg, database);
        let auth_token = conf.get("auth_token").unwrap();
        log::debug!("{}.new | auth_token: {:?}", dbg, auth_token);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{}.new | cycle: {:?}", dbg, cycle);
        let reconnect_cycle = conf.get_duration("reconnect").ok();
        log::debug!("{}.new | reconnectCycle: {:?}", dbg, reconnect_cycle);
        let (rx, rx_max_len) = conf.get_in_queue().unwrap();
        log::debug!("{}.new | RX: {},\tmax-length: {:?}", dbg, rx, rx_max_len);
        let debug: bool = conf.get("debug").unwrap_or(false);
        log::debug!("{}.new | debug: {:?}", dbg, debug);
        Self {
            name: self_name,
            address,
            database,
            auth_token,
            cycle,
            reconnect_cycle,
            rx,
            rx_max_len,
            debug,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> Self {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("ApiClientConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    pub fn read(parent: impl Into<String>, path: &str) -> ApiClientConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        Self::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("ApiClientConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("ApiClientConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
