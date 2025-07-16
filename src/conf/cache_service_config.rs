use log::{trace, debug};
use sal_sync::services::{conf::conf_tree::ConfTree, entity::name::Name, subscription::conf_subscribe::ConfSubscribe};
use std::{fs, time::Duration};
use crate::conf::service_config::ServiceConfig;
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service CacheService Cache:
///     retain: true        # true / false - enables storing cache on the disk
///     retain-delay: 30 s  # time to wait before next store, store on exit unconditionally
///     suscribe:
///         /App/MultiQueue: []
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct CacheServiceConfig {
    pub(crate) name: Name,
    pub(crate) retain: bool,
    pub(crate) retain_delay: Duration,
    pub(crate) subscribe: ConfSubscribe,
}
//
// 
impl CacheServiceConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service CacheService Cache:
    ///     retain: true        # true / false - enables storing cache on the disk
    ///     retain-delay: 30 s  # time to wait before next store, default 3 s, store on exit unconditionally
    ///     suscribe:
    ///         /App/MultiQueue: []
    /// ````
    pub fn new(parent: impl Into<String>, conf_tree: &mut ConfTree) -> Self {
        println!();
        trace!("CacheServiceConfig.new | conf_tree: {:?}", conf_tree);
        let self_id = format!("CacheServiceConfig({})", conf_tree.key);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.clone());
        trace!("{}.new | self_conf: {:?}", self_id, self_conf);
        let sufix = self_conf.sufix();
        let self_name = Name::new(parent, if sufix.is_empty() {self_conf.name()} else {sufix});
        debug!("{}.new | name: {:?}", self_id, self_name);
        let retain = self_conf.get_param_value("retain").unwrap_or(serde_yaml::Value::Bool(false)).as_bool().unwrap();
        debug!("{}.new | retain: {:?}", self_id, retain);
        let retain_delay = self_conf.get_duration("retain-delay").unwrap_or(Duration::from_secs(30));
        debug!("{}.new | retain-delay: {:?}", self_id, retain_delay);
        let subscribe = ConfSubscribe::new(self_conf.get_param_value("subscribe").unwrap_or(serde_yaml::Value::Null));
        debug!("{}.new | sudscribe: {:?}", self_id, subscribe);
        Self {
            name: self_name,
            retain,
            retain_delay,
            subscribe,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> CacheServiceConfig {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, &mut ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("CacheServiceConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> CacheServiceConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        CacheServiceConfig::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("CacheServiceConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("CacheServiceConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
