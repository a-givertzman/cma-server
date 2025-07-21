use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name, ConfSubscribe};
use std::{fs, time::Duration};
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
pub struct CacheServiceConf {
    pub(crate) name: Name,
    pub(crate) retain: bool,
    pub(crate) retain_delay: Duration,
    pub(crate) subscribe: ConfSubscribe,
}
//
// 
impl CacheServiceConf {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service CacheService Cache:
    ///     retain: true        # true / false - enables storing cache on the disk
    ///     retain-delay: 30 s  # time to wait before next store, default 3 s, store on exit unconditionally
    ///     suscribe:
    ///         /App/MultiQueue: []
    /// ````
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("CacheServiceConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, self_name);
        let retain = conf.get("retain").unwrap_or(false);
        log::debug!("{}.new | retain: {:?}", dbg, retain);
        let retain_delay = conf.get_duration("retain-delay").unwrap_or(Duration::from_secs(30));
        log::debug!("{}.new | retain-delay: {:?}", dbg, retain_delay);
        let subscribe = ConfSubscribe::new(conf.get("subscribe").unwrap_or(serde_yaml::Value::Null));
        log::debug!("{}.new | subscribe: {:?}", dbg, subscribe);
        Self {
            name: self_name,
            retain,
            retain_delay,
            subscribe,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> CacheServiceConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("CacheServiceConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> CacheServiceConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        CacheServiceConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("CacheServiceConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("CacheServiceConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
