use log::{debug, error, trace};
use sal_sync::services::{conf::conf_tree::ConfTree, entity::name::Name, service::link_name::LinkName};
use std::{fs, time::Duration, net::SocketAddr};
use crate::conf::{conf_keywd::ConfKind, service_config::ServiceConfig};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service TcpClient:
///     cycle: 1 ms
///     reconnect: 1 s  # default 3 s
///     address: 127.0.0.1:8080
///     in queue link:
///         max-length: 10000
///     send-to: MultiQueue.queue
///                         ...
#[derive(Debug, PartialEq, Clone)]
pub struct TcpClientConfig {
    pub(crate) name: Name,
    pub(crate) address: SocketAddr,
    pub(crate) cycle: Option<Duration>,
    pub(crate) reconnect_cycle: Option<Duration>,
    pub(crate) rx: String,
    pub(crate) rx_buffered: bool,
    pub(crate) rx_max_len: i64,
    pub(crate) send_to: LinkName,
}
//
// 
impl TcpClientConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service TcpClient:
    ///     cycle: 1 ms
    ///     reconnect: 1 s  # default 3 s
    ///     address: 127.0.0.1:8080
    ///     in queue link:
    ///         buffered: true
    ///         max-length: 10000
    ///     send-to: MultiQueue.queue
    ///                     ...
    pub fn new(parent: impl Into<String>, conf_tree: &mut ConfTree) -> TcpClientConfig {
        println!();
        trace!("TcpClientConfig.new | confTree: {:?}", conf_tree);
        let self_id = format!("TcpClientConfig({})", conf_tree.key);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.clone());
        trace!("{}.new | selfConf: {:?}", self_id, self_conf);
        let self_name = Name::new(parent, self_conf.name());
        debug!("{}.new | name: {:?}", self_id, self_name);
        let self_address: SocketAddr = self_conf.get_param_value("address").unwrap().as_str().unwrap().parse().unwrap();
        debug!("{}.new | address: {:?}", self_id, self_address);
        let cycle = self_conf.get_duration("cycle");
        debug!("{}.new | cycle: {:?}", self_id, cycle);
        let reconnect_cycle = self_conf.get_duration("reconnect");
        debug!("{}.new | reconnectCycle: {:?}", self_id, reconnect_cycle);
        let (rx, rx_max_len) = self_conf.get_in_queue().unwrap();
        let rx_buffered = rx_max_len > 0;
        debug!("{}.new | RX: {},\tmax-length: {}", self_id, rx, rx_max_len);
        let send_to = LinkName::new(self_conf.get_send_to().unwrap()).validate();
        debug!("{}.new | send-to: {}", self_id, send_to);
        if let Ok((_, _)) = self_conf.get_param_by_keyword("out", ConfKind::Queue) {
            error!("{}.new | Parameter 'out queue' - deprecated, use 'send-to' instead in conf: {:#?}", self_id, self_conf)
        }
        TcpClientConfig {
            name: self_name,
            address: self_address,
            cycle,
            reconnect_cycle,
            rx,
            rx_buffered,
            rx_max_len,
            send_to,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> TcpClientConfig {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, &mut ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("TcpClientConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> TcpClientConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        TcpClientConfig::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("TcpClientConfig.read | Error in config: {:?}\n\terror: {:#?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("TcpClientConfig.read | File {} reading error: {:#?}", path, err)
            }
        }
    }
}
