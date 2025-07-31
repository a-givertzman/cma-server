use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name};
use std::fs;

///
/// Config for [RecvService] format:
/// ```yaml
/// service RecvService RecvService1:
///     recv-limit: 100
///     in queue in-queue:
///         max-length: 10000
///```
#[derive(Debug, PartialEq, Clone)]
pub struct RecvServiceConf {
    pub name: Name,
    pub recv_limit: Option<usize>,
    pub in_queue: String,
    pub recv_length: usize,
}
//
// 
impl RecvServiceConf {
    ///
    /// Returns [RecvServiceConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("RecvServiceConf({})", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::debug!("{dbg}.new | name: {:?}", name);
        let recv_limit: Option<u64> = conf.get("recv-limit");
        log::debug!("{}.new | recv-limit: {:?}", dbg, recv_limit);
        let (recv, recv_length) = conf.get_in_queue().unwrap();
        log::debug!("{}.new | recv: {},\tmax-length: {:?}", dbg, recv, recv_length);
        Self {
            name,
            recv_limit: recv_limit.map(|v| v as usize),
            in_queue: recv,
            recv_length: recv_length as usize,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> RecvServiceConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("RecvServiceConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> RecvServiceConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        RecvServiceConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("RecvServiceConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("RecvServiceConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
