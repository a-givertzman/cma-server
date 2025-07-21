use std::fs;
use sal_sync::services::{conf::ConfTree, entity::PointConfig};
///
/// Creates config from serde_yaml::Value of following format:
/// ```yaml
/// service JdsService JdsService:          # service unique address used in the point path
///    in queue in-queue:
///        max-length: 10000
///    send-to: MultiQueue.in-queue
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct JdsServiceConf {
    pub(crate) name: String,
    pub(crate) rx: String,
    pub(crate) rx_max_len: i64,
    pub(crate) tx: String,
}
//
// 
impl JdsServiceConf {
    ///
    /// Creates new instance of the [JdsServiceConf]:
    pub fn new(conf: ConfTree) -> Self {
        log::trace!("JdsServiceConf.new | confTree: {:?}", conf);
        // self conf from first sub node
        //  - if additional sub nodes presents hit warning, FnConf must have single item
        if conf.count() > 1 {
            log::error!("JdsServiceConf.new | JdsServiceConf conf must have single item, additional items was ignored: {:?}", conf)
        };
        match conf.next() {
            Some(mut conf) => {
                let self_id = format!("JdsServiceConf({})", conf.key);
                log::trace!("{}.new | conf: {:?}", self_id, conf);
                let self_name = conf.name().unwrap();
                // let self_addr = self_conf.sufix();
                log::debug!("{}.new | name: {:?}", self_id, self_name);
                let cycle = conf.get_duration("cycle").ok();
                log::debug!("{}.new | cycle: {:?}", self_id, cycle);
                let (rx, rx_max_len) = conf.get_in_queue().unwrap();
                log::debug!("{}.new | RX: {},\tmax-length: {}", self_id, rx, rx_max_len);
                let tx = conf.get_out_queue().unwrap();
                log::debug!("{}.new | TX: {}", self_id, tx);
                JdsServiceConf {
                    name: self_name,
                    rx,
                    rx_max_len,
                    tx,
                }
            }
            None => {
                panic!("JdsServiceConf.new | Configuration is empty")
            }
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(value: &serde_yaml::Value) -> JdsServiceConf {
        Self::new(&mut ConfTree::new_root(value.clone()))
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(path: &str) -> JdsServiceConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        JdsServiceConf::from_yaml(&config)
                    }
                    Err(err) => {
                        panic!("JdsServiceConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("JdsServiceConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        panic!("JdsServiceConf.points | Not implemented for now");
    }
}
