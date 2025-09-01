use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::Name, LinkName};
use std::{fs, str::FromStr, time::Duration};

///
/// Config for SendService format:
/// ```yaml
/// service SendService SendService1:
///     cycle: 100 ms
///     send-to: /App/ApiClient.in-queue
///```
#[derive(Debug, PartialEq, Clone)]
pub struct SendServiceConf {
    pub name: Name,
    pub cycle: Option<Duration>,
    pub send_to: LinkName,
}
//
// 
impl SendServiceConf {
    ///
    /// Returns [SendServiceConf] built from `ConfTree`:
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> Self {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("SendServiceConf({})", me);
        log::trace!("{dbg}.new | conf: {:?}", conf);
        let name = Name::new(parent, me);
        log::trace!("{dbg}.new | name: {:?}", name);
        let cycle = conf.get_duration("cycle").ok();
        log::trace!("{dbg}.new | cycle: {:?}", cycle);
        let send_to: String = conf.get("send-to").expect(&format!("{dbg}.new | 'send-to' - not found or wrong configuration"));
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::trace!("{dbg}.new | send-to: {}", send_to);
        Self {
            name,
            cycle,
            send_to,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> SendServiceConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("SendServiceConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> SendServiceConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        SendServiceConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("SendServiceConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("SendServiceConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
