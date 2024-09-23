use log::{debug, error, trace};
use sal_sync::services::{conf::conf_tree::ConfTree, entity::name::Name, service::link_name::LinkName};
use std::{fs, net::SocketAddr, str::FromStr, time::Duration};
use crate::{conf::{conf_keywd::ConfKind, service_config::ServiceConfig}, services::server::jds_auth::TcpServerAuth};

///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service TcpServer:
///     cycle: 1 ms
///     address: 127.0.0.1:8080
///     reconnect: 1 s      # default 3 s
///     keep-timeout: 3s    # timeot keeping lost connection
///     auth: none          # none / secret / ssh
///     in queue link:
///         max-length: 10000
///     send-to: MultiQueue.queue
///                         ...
#[derive(Debug, PartialEq, Clone)]
pub struct TcpServerConfig {
    pub(crate) name: Name,
    pub(crate) cycle: Option<Duration>,
    pub(crate) address: SocketAddr,
    pub(crate) reconnect_cycle: Option<Duration>,
    pub(crate) keep_timeout: Duration,
    pub(crate) auth: TcpServerAuth,
    pub(crate) rx: String,
    pub(crate) rx_max_len: i64,
    pub(crate) send_to: LinkName,//String,
    pub(crate) cache: Option<String>,
}
//
// 
impl TcpServerConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service TcpServer:
    ///     cycle: 1 ms
    ///     address: 127.0.0.1:8080
    ///     reconnect: 1 s      # default 3 s
    ///     keep-timeout: 3s    # timeot keeping lost connection, default 10 s
    ///     auth: none          # none / secret / ssh
    ///     in queue link:
    ///         max-length: 10000
    ///     send-to: MultiQueue.queue
    ///                     ...
    pub fn new(parent: impl Into<String>, conf_tree: &mut ConfTree) -> TcpServerConfig {
        println!();
        trace!("TcpServerConfig.new | confTree: {:?}", conf_tree);
        let self_id = format!("TcpServerConfig({})", conf_tree.key);
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
        let keep_timeout = self_conf.get_duration("keep-timeout").unwrap_or(Duration::from_secs(10));
        debug!("{}.new | keepTimeout: {:?}", self_id, reconnect_cycle);
        let auth = self_conf.get_param_conf("auth");
        let auth = auth.or(self_conf.get_param_conf("auth-secret"));
        let auth = auth.or(self_conf.get_param_conf("auth-ssh"));
        let auth = auth.expect("{}.new | 'auth' or 'auth-secret' or 'auth-ssh' - not found");
        let auth = TcpServerAuth::new(auth);
        debug!("{}.new | auth: {:?}", self_id, auth);
        let (rx, rx_max_len) = self_conf.get_in_queue().unwrap();
        debug!("{}.new | 'in queue': {},\tmax-length: {}", self_id, rx, rx_max_len);
        let send_to = LinkName::from_str(self_conf.get_send_to().unwrap().as_str()).unwrap();
        debug!("{}.new | send-to: {:?}", self_id, send_to);
        if let Ok((_, _)) = self_conf.get_param_by_keyword("out", ConfKind::Queue) {
            error!("{}.new | Parameter 'out queue' - deprecated, use 'send-to' instead in conf: {:#?}", self_id, self_conf)
        }
        let cache = self_conf.get_param_value("cache").map_or_else(|_| None, |v| v.as_str().map(|v| v.to_owned()));
        debug!("{}.new | cache: {:?}", self_id, cache);
        TcpServerConfig {
            name: self_name,
            cycle,
            address: self_address,
            reconnect_cycle,
            keep_timeout,
            auth,
            rx,
            rx_max_len,
            send_to,
            cache,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> TcpServerConfig {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, &mut ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("TcpServerConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> TcpServerConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        TcpServerConfig::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("TcpServerConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("TcpServerConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
