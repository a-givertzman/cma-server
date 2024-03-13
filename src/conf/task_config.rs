#![allow(non_snake_case)]

use indexmap::IndexMap;
use log::{trace, debug, error};
use std::{fs, time::Duration};
use crate::conf::{
    fn_::{
        fn_config::FnConfig,
        fn_conf_kind::FnConfKind,
    },
    conf_tree::ConfTree, service_config::ServiceConfig,
    point_config::point_config::PointConfig,
};

///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// task operatingMetric:
///     cycle: 100 ms
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
pub struct TaskConfig {
    pub(crate) name: String,
    pub(crate) cycle: Option<Duration>,
    pub(crate) rx: String,
    pub(crate) rxMaxLength: i64,
    pub(crate) nodes: IndexMap<String, FnConfKind>,
    pub(crate) vars: Vec<String>,
}
///
/// 
impl TaskConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// task taskName:
    ///     cycle: 100 ms
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
    pub fn new(confTree: &mut ConfTree) -> TaskConfig {
        println!("\n");
        trace!("TaskConfig.new | confTree: {:?}", confTree);
        // self conf from first sub node
        //  - if additional sub nodes presents hit warning, FnConf must have single item
        if confTree.count() > 1 {
            error!("TaskConfig.new | FnConf must have single item, additional items was ignored: {:?}", confTree)
        };
        let mut vars = vec![];
        match confTree.next() {
            Some(selfConf) => {
                let self_id = format!("TaskConfig({})", selfConf.key);
                trace!("{}.new | MAPPING VALUE", self_id);
                let mut selfConf = ServiceConfig::new(&self_id, selfConf);
                trace!("{}.new | selfConf: {:?}", self_id, selfConf);
                let selfName = selfConf.name();
                debug!("{}.new | name: {:?}", self_id, selfName);
                let cycle = selfConf.getDuration("cycle");
                debug!("{}.new | cycle: {:?}", self_id, cycle);
                let (rx, rxMaxLength) = selfConf.getInQueue().unwrap();
                debug!("{}.new | RX: {},\tmax-length: {:?}", self_id, rx, rxMaxLength);
                let mut nodeIndex = 0;
                let mut nodes = IndexMap::new();
                for key in &selfConf.keys {
                    let nodeConf = selfConf.get(key).unwrap();
                    trace!("{}.new | nodeConf: {:?}", self_id, nodeConf);
                    nodeIndex += 1;
                    let nodeConf = FnConfig::new(&selfName, &nodeConf, &mut vars);
                    nodes.insert(
                        format!("{}-{}", nodeConf.name(), nodeIndex),
                        nodeConf,
                    );
                }
                TaskConfig {
                    name: selfName,
                    cycle,
                    rx,
                    rxMaxLength: rxMaxLength,
                    nodes,
                    vars,
                }
            },
            None => {
                panic!("TaskConfig.new | Configuration is empty")
            },
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(value: &serde_yaml::Value) -> TaskConfig {
        Self::new(&mut ConfTree::newRoot(value.clone()))
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(path: &str) -> TaskConfig {
        match fs::read_to_string(&path) {
            Ok(yamlString) => {
                match serde_yaml::from_str(&yamlString) {
                    Ok(config) => {
                        TaskConfig::from_yaml(&config)
                    },
                    Err(err) => {
                        panic!("TaskConfig.read | Error in config: {:?}\n\terror: {:?}", yamlString, err)
                    },
                }
            },
            Err(err) => {
                panic!("TaskConfig.read | File {} reading error: {:?}", path, err)
            },
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        self.nodes.iter().fold(vec![], |mut points, (_nodeName, nodeConf)| {
            points.extend(nodeConf.points());
            points
        })
    }
}
