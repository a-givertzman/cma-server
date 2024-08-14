use indexmap::IndexMap;
use log::{debug, info, trace};
use std::{fs, path::Path, str::FromStr};
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
pub struct AppConfig {
    pub(crate) name: String,
    pub(crate) description: String,
    // pub(crate) cycle: Option<Duration>,
    pub(crate) nodes: IndexMap<ConfKeywd, ConfTree>,
}
//
// 
impl AppConfig {
    ///
    /// Creates new instance of the [AppConfig]:
    pub fn new(conf_tree: &mut ConfTree) -> Self {
        println!();
        trace!("AppConfig.new | confTree: {:?}", conf_tree);
        let self_id = format!("AppConfig({})", conf_tree.key);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.to_owned());
        trace!("{}.new | selfConf: {:?}", self_id, self_conf);
        let self_name = self_conf.get_param_value("name").unwrap().as_str().unwrap().to_owned();
        debug!("{}.new | name: {:?}", self_id, self_name);
        let description = self_conf.get_param_value("description").unwrap().as_str().unwrap().to_owned();
        debug!("{}.new | description: {:?}", self_id, description);
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
            // cycle,
            nodes,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml_value(value: &serde_yaml::Value) -> AppConfig {
        Self::new(&mut ConfTree::newRoot(value.clone()))
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read<P>(path: Vec<P>) -> AppConfig where P: AsRef<Path> {
        let self_id = "AppConfig";
        info!("{}.read | Reading configuration files...", self_id);
        let mut files = vec![];
        for p in path {
            match fs::read_to_string(&p) {
                Ok(f) => {
                    files.push(f)
                }
                Err(err) => {
                    panic!("{}.read | File '{}' reading error: {:?}", self_id, p.as_ref().display(), err)
                }
            }
        }
        let yaml_string = files.join("\n");
        match serde_yaml::from_str(&yaml_string) {
            Ok(config) => {
                info!("{}.read | Reading configuration files - ok", self_id);
                AppConfig::from_yaml_value(&config)
            }
            Err(err) => {
                panic!("{}.read | Error in config: {:?}\n\terror: {:?}", self_id, yaml_string, err)
            }
        }
    }
}
