#![allow(non_snake_case)]

use log::{trace, debug, error};
use std::{fs, collections::HashMap, str::FromStr};

use crate::core_::conf::{metric_config::MetricConfig, fn_config::FnConfig, conf_tree::ConfTree, conf_keywd::ConfKeywd};


#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TaskConfNode {
    Fn(FnConfig),
    Metric(MetricConfig)
}

impl TaskConfNode {
    pub fn name(&self) -> String {
        match self {
            TaskConfNode::Fn(conf) => conf.name.clone(),
            TaskConfNode::Metric(conf) => conf.name.clone(),
        }
    }
}

///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// task operatingMetric:
///     cycle: 100 ms
///     metrics:
///         metric sqlUpdateMetric:
///             table: "TableName"
///             sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
///             initial: 123.456
///             inputs:
///                 input1:
///                     fn functionName:
///                         ...
///                 input2:
///                     metric sqlSelectMetric:
///                         ...
#[derive(Debug, PartialEq, Clone)]
pub struct TaskConfig {
    pub(crate) name: String,
    pub(crate) cycle: u64,
    pub(crate) apiQueue: String,
    pub(crate) nodes: HashMap<String, FnConfig>,
    pub(crate) vars: Vec<String>,
}
///
/// 
impl TaskConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// task taskName:
    ///     cycle: 100  // ms
    ///     metric sqlUpdateMetric:
    ///         table: "TableName"
    ///         sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
    ///         initial: 123.456
    ///         inputs:
    ///             input1:
    ///                 fn functionName:
    ///                     ...
    ///             input2:
    ///                 metric sqlSelectMetric:
    ///                     ...
    pub fn new(confTree: &mut ConfTree) -> TaskConfig {
        println!("\n");
        trace!("TaskConfig.new | confTree: {:?}", confTree);
        // self conf from first sub node
        //  - if additional sub nodes presents hit warning, FnConf must have single item
        if confTree.count() > 1 {
            error!("TaskConfig.new | FnConf must have single item, additional items was ignored")
        };
        let mut vars = vec![];
        match confTree.next() {
            Some(mut selfConf) => {
                debug!("TaskConfig.new | MAPPING VALUE");
                trace!("TaskConfig.new | selfConf: {:?}", selfConf);
                
                let selfName = match ConfKeywd::from_str(&selfConf.key) {
                    Ok(selfKeyword) => selfKeyword.data(),
                    Err(err) => panic!("TaskConfig.new | Unknown metric name in {:?}\n\tdetales: {:?}", &selfConf.key, err),
                };
                trace!("TaskConfig.new | selfName: {:?}", selfName);
                let selfCycle = (&mut selfConf).remove("cycle").unwrap().as_u64().unwrap();
                trace!("TaskConfig.new | selfCycle: {:?}", selfCycle);
                let selfApiQueue = (&mut selfConf).remove("api-queue").unwrap().as_str().unwrap().to_string();
                trace!("TaskConfig.new | selfApiQueue: {:?}", selfApiQueue);

                let mut nodeIndex = 0;
                let mut selfNodes = HashMap::new();
                for selfNodeConf in selfConf.subNodes().unwrap() {
                    trace!("TaskConfig.new | selfNodeConf: {:?}", selfNodeConf);
                    nodeIndex += 1;
                    let nodeConf = FnConfig::new(&selfNodeConf, &mut vars);
                    selfNodes.insert(
                        format!("{}-{}", nodeConf.name, nodeIndex),
                        nodeConf,
                    );
                }
                TaskConfig {
                    name: selfName,
                    cycle: selfCycle,
                    apiQueue: selfApiQueue,
                    nodes: selfNodes,
                    vars: vars,
                }
            },
            None => {
                panic!("TaskConfig.new | Configuration is empty")
            },
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn fromYamlValue(value: &serde_yaml::Value) -> TaskConfig {
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
                        TaskConfig::fromYamlValue(&config)
                    },
                    Err(err) => {
                        panic!("Error in config: {:?}\n\terror: {:?}", yamlString, err)
                    },
                }
            },
            Err(err) => {
                panic!("File {} reading error: {:?}", path, err)
            },
        }
    }

}
