use std::fs;
use log::{debug, error, trace};
use sal_sync::services::{conf::conf_tree::ConfTree, entity::point::point_config::PointConfig};
use crate::conf::service_config::ServiceConfig;
///
/// Creates config from serde_yaml::Value of following format:
/// ```yaml
/// service JdsService JdsService:          # service unique address used in the point path
///    in queue in-queue:
///        max-length: 10000
///    send-to: MultiQueue.in-queue
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct JdsServiceConfig {
    pub(crate) name: String,
    pub(crate) rx: String,
    pub(crate) rx_max_len: i64,
    pub(crate) tx: String,
}
//
// 
impl JdsServiceConfig {
    ///
    /// Creates new instance of the [JdsServiceConfig]:
    pub fn new(conf_tree: &mut ConfTree) -> Self {
        println!();
        trace!("JdsServiceConfig.new | confTree: {:?}", conf_tree);
        // self conf from first sub node
        //  - if additional sub nodes presents hit warning, FnConf must have single item
        if conf_tree.count() > 1 {
            error!("JdsServiceConfig.new | JdsServiceConfig conf must have single item, additional items was ignored: {:?}", conf_tree)
        };
        match conf_tree.next() {
            Some(self_conf) => {
                let self_id = format!("JdsServiceConfig({})", self_conf.key);
                let mut self_conf = ServiceConfig::new(&self_id, self_conf);
                trace!("{}.new | selfConf: {:?}", self_id, self_conf);
                let self_name = self_conf.name();
                // let self_addr = self_conf.sufix();
                debug!("{}.new | name: {:?}", self_id, self_name);
                let cycle = self_conf.get_duration("cycle");
                debug!("{}.new | cycle: {:?}", self_id, cycle);
                let (rx, rx_max_len) = self_conf.get_in_queue().unwrap();
                debug!("{}.new | RX: {},\tmax-length: {}", self_id, rx, rx_max_len);
                let tx = self_conf.get_out_queue().unwrap();
                debug!("{}.new | TX: {}", self_id, tx);
                JdsServiceConfig {
                    name: self_name,
                    rx,
                    rx_max_len,
                    tx,
                }
            }
            None => {
                panic!("JdsServiceConfig.new | Configuration is empty")
            }
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(value: &serde_yaml::Value) -> JdsServiceConfig {
        Self::new(&mut ConfTree::new_root(value.clone()))
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(path: &str) -> JdsServiceConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        JdsServiceConfig::from_yaml(&config)
                    }
                    Err(err) => {
                        panic!("JdsServiceConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("JdsServiceConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        panic!("JdsServiceConfig.points | Not implemented for now");
    }
}
