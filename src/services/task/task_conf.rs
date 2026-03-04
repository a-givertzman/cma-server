use indexmap::IndexMap;
use sal_sync::services::{conf::{ConfTree, ConfTreeGet}, entity::{Name, PointConf}, ConfSubscribe, task::functions::{FnConfKind, FnConfig}};
use std::{fs, time::Duration};
///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// service Task operatingMetric:
///     cycle: 100 ms
///         in queue recv-queue:
///             max-length: 10000
///     metrics:
///         fn sqlUpdateMetric:
///             table: "TableName"
///             sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
///             initial: 123.456
///             inputs:
///                 input1:
///                     fn functionName:
///                         ...
///                 input2:
///                     fn SqlMetric:
///                         ...
#[derive(Debug, PartialEq, Clone)]
pub struct TaskConf {
    pub name: Name,
    pub cycle: Option<Duration>,
    pub rx: String,
    pub rx_max_length: i64,
    pub subscribe: ConfSubscribe,
    pub nodes: IndexMap<String, FnConfKind>,
    pub vars: Vec<String>,
}
//
// 
impl TaskConf {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// task taskName:
    ///     cycle: 100 ms
    ///     in queue recv-queue:
    ///         max-length: 10000
    ///     fn sqlUpdateMetric:
    ///         table: "TableName"
    ///         sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
    ///         initial: 123.456
    ///         inputs:
    ///             input1:
    ///                 fn functionName:
    ///                     ...
    ///             input2:
    ///                 fn SqlMetric:
    ///                     ...
    pub fn new(parent: impl Into<String>, conf: ConfTree) -> TaskConf {
        let mut vars = vec![];
        let me = conf.sufix_or(conf.name().unwrap());
        let dbg = format!("TaskConf({})", me);
        log::trace!("{}.new | conf: {:?}", dbg, conf);
        let self_name = Name::new(parent, me);
        log::trace!("{}.new | name: {:?}", dbg, self_name);
        let cycle = conf.get_duration("cycle").ok();
        log::trace!("{}.new | cycle: {:?}", dbg, cycle);
        let (rx, rx_max_length) = conf.get_in_queue().unwrap();
        log::trace!("{}.new | RX: {},\tmax-length: {:?}", dbg, rx, rx_max_length);
        let subscribe = conf.get("subscribe").unwrap_or(serde_yaml::Value::Null);
        let subscribe = ConfSubscribe::new(subscribe);
        log::trace!("{}.new | subscribe: {:#?}", dbg, subscribe);
        let mut node_index = 0;
        let mut nodes = IndexMap::new();
        for key in conf.keys(&["wait-started", "cycle", "subscribe", format!("in queue {}", rx).as_str()]) {
            let node_conf = conf.get(key).unwrap();
            log::trace!("{}.new | nodeConf: {:?}", dbg, node_conf);
            node_index += 1;
            let node_conf = FnConfig::new(&self_name.join(), &self_name, &node_conf, &mut vars);
            nodes.insert(
                format!("{}-{}", node_conf.name(), node_index),
                node_conf,
            );
        }
        TaskConf {
            name: self_name,
            cycle,
            rx,
            rx_max_length,
            subscribe,
            nodes,
            vars,
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent: impl Into<String>, value: &serde_yaml::Value) -> TaskConf {
        match value.as_mapping().unwrap().into_iter().next() {
            Some((key, value)) => {
                Self::new(parent, ConfTree::new(key.as_str().unwrap(), value.clone()))
            }
            None => {
                panic!("TaskConf.from_yaml | Format error or empty conf: {:#?}", value)
            }
        }        
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent: impl Into<String>, path: &str) -> TaskConf {
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        TaskConf::from_yaml(parent, &config)
                    }
                    Err(err) => {
                        panic!("TaskConf.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("TaskConf.read | File {} reading error: {:?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConf> {
        self.nodes.iter().fold(vec![], |mut points, (_node_name,node_conf)| {
            points.extend(node_conf.points());
            points
        })
    }
}
