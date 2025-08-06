use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name, LinkName};
use std::{fs, net::SocketAddr, str::FromStr, time::Duration};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service ApiClient:
///     cycle: 1 ms
///     reconnect: 1 s  # default 3 s
///     address: 127.0.0.1:8080
///     in queue api-link:
///         max-length: 10000
///     send-to: MultiQueue.queue   # Used to return replies from SQL requests
///     auth-token: 123!@#
///     debug: false                # API debug mode, optional, default false
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct ApiClientConf {
    pub name: Name,
    pub address: SocketAddr,
    pub database: String,
    pub auth_token: String,
    pub cycle: Option<Duration>,
    pub reconnect_cycle: Option<Duration>,
    pub rx: String,
    pub rx_max_len: i64,
    pub send_to: Option<LinkName>,
    pub debug: bool,
}
//
// 
impl ApiClientConf {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service ApiClient:
    ///     cycle: 1 ms
    ///     reconnect: 1 s  # default 3 s
    ///     address: 127.0.0.1:8080
    ///     in queue api-link:
    ///         max-length: 10000
    ///     send-to: MultiQueue.queue   # Used to return replies from SQL requests
    ///     debug: false                # API debug mode, optional, default false
    /// ```
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("ApiClientConfig({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, self_name);
        let address: String = conf.get("address").unwrap();
        let address: SocketAddr = address.parse().unwrap();
        log::trace!("{}.new | address: {:?}", dbg, address);
        let database = conf.get("database").unwrap();
        log::trace!("{}.new | database: {:?}", dbg, database);
        let auth_token = conf.get("auth-token").unwrap();
        log::trace!("{}.new | auth-token: {:?}", dbg, auth_token);
        let cycle = conf.get_duration("cycle").ok();
        log::trace!("{}.new | cycle: {:?}", dbg, cycle);
        let reconnect_cycle = conf.get_duration("reconnect").ok();
        log::trace!("{}.new | reconnectCycle: {:?}", dbg, reconnect_cycle);
        let (rx, rx_max_len) = conf.get_in_queue().unwrap();
        log::trace!("{}.new | RX: {},\tmax-length: {:?}", dbg, rx, rx_max_len);
        let send_to: Option<String> = conf.get("send-to");
        let send_to = send_to.map(|send_to| LinkName::from_str(&send_to).unwrap());
        log::trace!("{}.new | send-to: {:?}", dbg, send_to);
        let debug: bool = conf.get("debug").unwrap_or(false);
        log::trace!("{}.new | debug: {:?}", dbg, debug);
        Self {
            name: self_name,
            address,
            database,
            auth_token,
            cycle,
            reconnect_cycle,
            rx,
            rx_max_len,
            send_to,
            debug,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    #[allow(unused)]
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
    #[allow(unused)]
    pub fn read(parent: impl Into<String>, path: &str) -> ApiClientConf {
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
