use log::{debug, error, trace, warn};
use sal_sync::services::{conf::conf_tree::ConfTree, entity::name::Name, service::link_name::LinkName};
use std::fs;
use crate::conf::service_config::ServiceConfig;
use super::conf_keywd::ConfKind;
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service MultiQueue:
///     cycle: 1 ms
///     reconnect: 1 s  # default 3 s
///     address: 127.0.0.1:8080
///     in queue link:
///         max-length: 10000
///     send-to:                  # optional
///         - MultiQueue.queue
///                         ...
#[derive(Debug, PartialEq, Clone)]
pub struct MultiQueueConfig {
    pub(crate) name: Name,
    pub(crate) rx: String,
    pub(crate) rx_max_length: i64,
    pub(crate) send_to: Vec<LinkName>,
}
//
// 
impl MultiQueueConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service MultiQueue:
    ///     in queue in-queue:
    ///         max-length: 10000
    ///     send-to:                    # optional
    ///         - Service0.in-queue
    ///         - Service1.in-queue
    ///         ...
    ///         - ServiceN.in-queue
    ///                     ...
    pub fn new(parent: impl Into<String>, conf_tree: &mut ConfTree) -> MultiQueueConfig {
        println!();
        trace!("MultiQueueConfig.new | confTree: {:?}", conf_tree);
        let self_id = format!("MultiQueueConfig({})", conf_tree.key);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.clone());
        trace!("{}.new | self_conf: {:?}", self_id, self_conf);
        let me = if self_conf.sufix().is_empty() {self_conf.name()} else {self_conf.sufix()};
        let self_name = Name::new(parent, me);
        debug!("{}.new | self_name: {:?}", self_id, self_name);
        let (rx, rx_max_length) = self_conf.get_in_queue().unwrap();
        debug!("{}.new | 'in queue': {},\tmax-length: {}", self_id, rx, rx_max_length);
        let send_to = match self_conf.get_send_to_many() {
            crate::conf::service_config::ConfParam::Ok(send_to) => send_to.into_iter().map(LinkName::new).collect(),
            crate::conf::service_config::ConfParam::None => vec![],
            crate::conf::service_config::ConfParam::Err(err) => {
                warn!("{}.new | Get 'send-to' many error: {:?} in config: {:#?}", self_id, err, self_conf);
                vec![]
            }
        };
        debug!("{}.new | 'send-to': {:?}", self_id, send_to);
        if let Ok((_, _)) = self_conf.get_param_by_keyword("out", ConfKind::Queue) {
            error!("{}.new | Parameter 'out queue' - deprecated, use 'send-to' instead in conf: {:#?}", self_id, self_conf)
        }
        MultiQueueConfig {
            name: self_name,
            rx,
            rx_max_length,
            send_to,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> MultiQueueConfig {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, &mut ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("MultiQueueConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> MultiQueueConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        MultiQueueConfig::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("MultiQueueConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("MultiQueueConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
