use indexmap::IndexMap;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::{Name, PointConf}, LinkName, task::functions::{FnConfKeywd, FnConfKindName}};
use std::{fs, str::FromStr, time::Duration};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service ProducerService:
///     cycle: 1000 ms                          # operating cycle time of the module
///     send-to: /App/MultiQueue.in-queue
///     debug: true                             # each point will be debugged
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
    pub(crate) send_to: LinkName,
    pub(crate) debug: bool,
    // pub(crate) subscribe: ConfSubscribe,
    pub(crate) nodes: IndexMap<String, PointConf>,
}
//
// 
impl ProducerServiceConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// service ProducerService:
    ///     cycle: 1000 ms                          # operating cycle time of the module
    ///     send-to: /App/MultiQueue.in-queue
    ///     debug: true                             # each point will be debugged
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
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> ProducerServiceConfig {
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("ProducerServiceConfig({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::debug!("{}.new | name: {:?}", dbg, self_name);
        let cycle = conf.get_duration("cycle").ok();
        log::debug!("{}.new | cycle: {:?}", dbg, cycle);
        let send_to: String = conf.get("send-to").unwrap();
        let send_to = LinkName::from_str(&send_to).unwrap();
        log::debug!("{}.new | send-to: '{}'", dbg, send_to);
        let debug = conf.get("debug").unwrap_or(false);
        log::debug!("{}.new | debug: '{}'", dbg, debug);
        let mut nodes = IndexMap::new();
        for node_name in conf.keys(&["cycle", "send-to", "debug"]) {
            let node_conf: ConfTree = conf.get(&node_name).unwrap();
            for key in &node_conf.keys(&[""]) {
                let keyword = FnConfKeywd::from_str(key).unwrap();
                if keyword.kind() == FnConfKindName::Point {
                    let point_name = format!("{}/{}", self_name, keyword.data());
                    let point_conf = node_conf.get(key).unwrap();
                    log::trace!("{}.new | Point '{}'", dbg, point_name);
                    log::trace!("{}.new | Point '{}'   |   conf: {:?}", dbg, point_name, point_conf);
                    let node_conf = PointConf::new(&Name::new(&self_name, &node_name), &point_conf);
                    nodes.insert(
                        node_conf.name.clone(),
                        node_conf,
                    );
                } else {
                    log::debug!("{}.new | device expected, but found {:?}", dbg, keyword);
                }
            }
        }
        ProducerServiceConfig {
            name: self_name,
            cycle,
            send_to,
            debug,
            nodes,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> ProducerServiceConfig {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("ProducerServiceConfig.from_yaml | Format error or empty conf: {:#?}", value)
            }
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
                    }
                    Err(err) => {
                        panic!("ProducerServiceConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("ProducerServiceConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConf> {
        self.nodes.iter().fold(vec![], |mut points, (_node_name,node_conf)| {
            points.push(node_conf.clone());
            points
        })
    }
}
