use indexmap::IndexMap;
use log::{trace, debug};
use std::{fs, str::FromStr, time::Duration};
use crate::conf::{
    conf_subscribe::ConfSubscribe, 
    conf_tree::ConfTree, 
    fn_::{fn_conf_keywd::{FnConfKeywd, FnConfKindName}, fn_config::FnConfig},
    point_config::{name::Name, point_config::PointConfig}, 
    service_config::ServiceConfig,
};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service ProducerService:
///     cycle: 1000 ms                         # operating cycle time of the module
///     out queue: /App/MultiQueue.in-queue
///     points:
///         point Winch.ValveEV1: 
///             type: Bool
///             history: rw
///             alarm: 4
///         point Winch.EncoderBR1: 
///             type: Int
///         point Winch.LVDT1: 
///             type: Real
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct ProducerServiceConfig {
    pub(crate) name: Name,
    pub(crate) cycle: Option<Duration>,
    pub(crate) send_to: String,
    // pub(crate) subscribe: ConfSubscribe,
    pub(crate) nodes: IndexMap<String, PointConfig>,
}
///
/// 
impl ProducerServiceConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service ProducerService:
    ///     cycle: 1000 ms                         # operating cycle time of the module
    ///     out queue: /App/MultiQueue.in-queue
    ///     points:
    ///         point Winch.ValveEV1: 
    ///             type: Bool
    ///             history: rw
    ///             alarm: 4
    ///         point Winch.EncoderBR1: 
    ///             type: Int
    ///         point Winch.LVDT1: 
    ///             type: Real
    /// ```
    pub fn new(parent: impl Into<String>, conf_tree: &mut ConfTree) -> ProducerServiceConfig {
        println!();
        trace!("ProducerServiceConfig.new | confTree: {:?}", conf_tree);
        let self_id = format!("ProducerServiceConfig({})", conf_tree.key);
        let mut self_conf = ServiceConfig::new(&self_id, conf_tree.clone());
        trace!("{}.new | self_conf: {:?}", self_id, self_conf);
        let self_name = Name::new(parent, self_conf.sufix());
        debug!("{}.new | name: {:?}", self_id, self_name);
        let cycle = self_conf.get_duration("cycle");
        debug!("{}.new | cycle: {:?}", self_id, cycle);
        let send_to = self_conf.get_send_to().unwrap();
        debug!("{}.new | send_to: '{}'", self_id, send_to);
        let mut nodes = IndexMap::new();
        for node_name in self_conf.keys.clone() {
            let node_conf = self_conf.get(&node_name).unwrap();
            let node_conf = ServiceConfig::new(&self_id, node_conf);
            for key in &node_conf.keys {
                let keyword = FnConfKeywd::from_str(key).unwrap();
                if keyword.kind() == FnConfKindName::Point {
                    let point_name = format!("{}/{}", self_name, keyword.data());
                    let point_conf = node_conf.get(key).unwrap();
                    trace!("{}.new | Point '{}'", self_id, point_name);
                    trace!("{}.new | Point '{}'   |   conf: {:?}", self_id, point_name, point_conf);
                    let node_conf = PointConfig::new(&self_id, &Name::new(&self_name, &node_name), &point_conf);
                    nodes.insert(
                        node_conf.name.clone(),
                        node_conf,
                    );
                } else {
                    debug!("{}.new | device expected, but found {:?}", self_id, keyword);
                }
            }
        }
        ProducerServiceConfig {
            name: self_name,
            cycle,
            send_to,
            nodes,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> ProducerServiceConfig {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, &mut ConfTree::new(key.as_str().unwrap(), value.clone()))
            },
            None => {
                panic!("ProducerServiceConfig.from_yaml | Format error or empty conf: {:#?}", value)
            },
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> ProducerServiceConfig {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        ProducerServiceConfig::from_yaml(parent, &config)
                    },
                    Err(err) => {
                        panic!("ProducerServiceConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    },
                }
            },
            Err(err) => {
                panic!("ProducerServiceConfig.read | File {} reading error: {:?}", path, err)
            },
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        self.nodes.iter().fold(vec![], |mut points, (_node_name,node_conf)| {
            points.push(node_conf.clone());
            points
        })
    }
}
