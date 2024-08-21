use indexmap::IndexMap;
use log::{debug, trace};
use std::{fs, str::FromStr, time::Duration};
use crate::conf::{
    conf_keywd::{ConfKeywd, ConfKind}, conf_tree::ConfTree, service_config::ServiceConfig
};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// name: ApplicationName
/// description: Short explanation / purpose etc.
/// 
/// service ProfinetClient Ied01:          # device will be executed in the independent thread, must have unique name
///    in queue in-queue:
///        max-length: 10000
///    send-to: MultiQueue.in-queue
///    cycle: 1 ms                     # operating cycle time of the device
///    protocol: 'profinet'
///    description: 'S7-IED-01.01'
///    ip: '192.168.100.243'
///    rack: 0
///    slot: 1
///    db db899:                       # multiple DB blocks are allowed, must have unique namewithing parent device
///        description: 'db899 | Exhibit - drive data'
///        number: 899
///        offset: 0
///        size: 34
///        point Drive.Speed: 
///            type: 'Real'
///            offset: 0
///                 ...
/// service Task task1:
///     cycle: 1 ms
///     in queue recv-queue:
///         max-length: 10000
///     let var0: 
///         input: const real 2.224
///     
///     fn ToMultiQueue:
///         in1 point CraneMovement.BoomUp: 
///             type: 'Int'
///             comment: 'Some indication'
///             input fn Add:
///                 input1 fn Add:
///                     input1: const real 0.2
///                     input2: point real '/path/Point.Name'
///     ...
/// 
#[derive(Debug, PartialEq, Clone)]
pub struct ServicesConfig {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) cycle: Option<Duration>,
    pub(crate) nodes: IndexMap<ConfKeywd, ConfTree>,
}
//
// 
impl ServicesConfig {
    ///
    /// Creates new instance of the [ServicesConfig]:
    pub fn new(conf_tree: &mut ConfTree) -> Self {
        println!();
        trace!("ServicesConfig.new | confTree: {:?}", conf_tree);
        // self conf from first sub node
        //  - if additional sub nodes presents hit warning, FnConf must have single item
        let self_id = format!("ServicesConfig({})", conf_tree.key);
        trace!("{}.new | MAPPING VALUE", self_id);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.to_owned());
        trace!("{}.new | selfConf: {:?}", self_id, self_conf);
        let self_name = self_conf.get_param_value("name").unwrap().as_str().unwrap().to_owned();
        // let service_name = self_conf.sufix();
        debug!("{}.new | name: {:?}", self_id, self_name);
        let description = self_conf.get_param_value("description").unwrap().as_str().unwrap().to_owned();
        debug!("{}.new | description: {:?}", self_id, description);
        let cycle = self_conf.get_duration("cycle");
        debug!("{}.new | cycle: {:?}", self_id, cycle);
        let mut nodes = IndexMap::new();
        println!();
        for key in &self_conf.keys {
            let keyword = ConfKeywd::from_str(key).unwrap();
            match keyword.kind() {
                ConfKind::Service | ConfKind::Task => {
                    let node_name = keyword.name();
                    let node_conf = self_conf.get(key).unwrap();
                    if log::max_level() == log::LevelFilter::Debug {
                        let sufix = match keyword.sufix().is_empty() {
                            true => "".to_owned(),
                            false => format!(": '{}'", keyword.sufix()),
                        };
                        debug!("{}.new | service '{}'{}", self_id, node_name, sufix);
                    } else if log::max_level() == log::LevelFilter::Trace {
                        trace!("{}.new | DB '{}'   |   conf: {:?}", self_id, node_name, node_conf);
                    }
                    nodes.insert(
                        keyword,
                        node_conf,
                    );
                }
                _ => {
                    panic!("{}.new | Node '{:?}' - is not allowed in the root of the application config", self_id, keyword);
                }
            }
        }
        Self {
            name: self_name,
            description,
            cycle,
            nodes,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml_value(value: &serde_yaml::Value) -> ServicesConfig {
        Self::new(&mut ConfTree::newRoot(value.clone()))
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(path: &str) -> ServicesConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        ServicesConfig::from_yaml_value(&config)
                    }
                    Err(err) => {
                        panic!("ServicesConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("ServicesConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
}
