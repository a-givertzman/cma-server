use sal_sync::services::{conf::{ConfKind, ConfTree, ConfTreeGet}, entity::Name, LinkName};
use std::{fs, net::SocketAddr, str::FromStr, time::Duration};
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
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct TcpClientConf {
    pub name: Name,
    pub address: SocketAddr,
    pub cycle: Option<Duration>,
    pub reconnect_cycle: Option<Duration>,
    pub rx: String,
    pub rx_buffered: bool,
    pub rx_max_len: i64,
    pub send_to: LinkName,
}
//
// 
impl TcpClientConf {
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
    /// ```
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> TcpClientConf {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("TcpClientConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, self_name);
        let address: String = conf.get("address").unwrap();
        let self_address: SocketAddr = address.parse().unwrap();
        log::debug!("{}.new | address: {:?}", dbg, self_address);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{}.new | cycle: {:?}", dbg, cycle);
        let reconnect_cycle = conf.get_duration("reconnect").ok();
        log::debug!("{}.new | reconnectCycle: {:?}", dbg, reconnect_cycle);
        let (rx, rx_max_len) = conf.get_in_queue().unwrap();
        let rx_buffered = rx_max_len > 0;
        log::debug!("{}.new | RX: {},\tmax-length: {}", dbg, rx, rx_max_len);
        let send_to: String = conf.get("send-to").unwrap();
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::debug!("{}.new | send-to: {}", dbg, send_to);
        if let Ok((_, _)) = conf.get_by_keywd("out", ConfKind::Queue) {
            log::error!("{}.new | Parameter 'out queue' - deprecated, use 'send-to' instead in conf: {:#?}", dbg, conf)
        }
        TcpClientConf {
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
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> TcpClientConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("TcpClientConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> TcpClientConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        TcpClientConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("TcpClientConf.read | Error in config: {:?}\n\terror: {:#?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("TcpClientConf.read | File {} reading error: {:#?}", path, err)
            }
        }
    }
}
