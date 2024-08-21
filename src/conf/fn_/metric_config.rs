use indexmap::IndexMap;
use log::{trace, debug};
use std::{fs, str::FromStr};
use crate::conf::{conf_tree::ConfTree, fn_::{fn_conf_keywd::FnConfKeywd, fn_conf_kind::FnConfKind, fn_config::FnConfig}, point_config::{name::Name, point_config::PointConfig}};

///
/// creates config from serde_yaml::Value of following format:
/// ```yaml
/// fn sqlUpdateMetric:
///     table: "TableName"
///     sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
///     initial: 123.456
///     inputs:
///         input1: 
///             fn functionName:
///                 ...
///         input2:
///             fn SqlMetric:
///                 ...
#[derive(Debug, Clone, PartialEq)]
pub struct MetricConfig {
    pub(crate) name: String,
    pub(crate) table: String,
    pub(crate) sql: String,
    pub(crate) initial: f64,
    pub(crate) inputs: IndexMap<String, FnConfKind>,
    pub(crate) vars: Vec<String>,
}
//
// 
impl MetricConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// fn sqlUpdateMetric:
    ///     table: "TableName"
    ///     sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
    ///     initial: 123.456
    ///     inputs:
    ///         input1: 
    ///             fn functionName:
    ///                 ...
    ///         input2:
    ///             fn SqlMetric:
    ///                 ...
    pub fn new(parent_id: &str, parent_name: &Name, conf_tree: &ConfTree, vars: &mut Vec<String>) -> MetricConfig {
        println!();
        trace!("MetricConfig.new | confTree: {:?}", conf_tree);
        // self conf from first sub node
        //  - if additional sub nodes presents hit warning, FnConf must have single item
        if conf_tree.isMapping() {
                debug!("MetricConfig.new | MAPPING VALUE");
                trace!("MetricConfig.new | confTree: {:?}", conf_tree);
                let self_name = match FnConfKeywd::from_str(&conf_tree.key) {
                    Ok(self_keyword) => {
                        self_keyword.data()
                    }
                    Err(err) => {
                        panic!("MetricConfig.new | Unknown metric name in {:?}\n\tdetales: {:?}", &conf_tree.key, err)
                    }
                };
                let mut inputs = IndexMap::new();
                match conf_tree.get("inputs") {
                    Some(inputs_node) => {
                        for input_conf in inputs_node.subNodes().unwrap() {
                            trace!("MetricConfig.new | input conf: {:?}\t|\t{:?}", input_conf.key, input_conf.conf);
                            if input_conf.isMapping() {
                                inputs.insert(
                                    (input_conf).key.to_string(), 
                                    FnConfig::new(parent_id, parent_name, &input_conf.next().unwrap(), vars),
                                );
                            } else {
                                inputs.insert(
                                    (input_conf).key.to_string(), 
                                    FnConfig::new(parent_id, parent_name, &input_conf, vars),
                                );
                            };
                        }
                    }
                    None => {
                        panic!("MetricConfig.new | Metric '{:?}' 'inputs' not found", &conf_tree.key)
                    }
                }
                MetricConfig {
                    name: self_name,
                    table: (conf_tree).asStr("table").unwrap().to_string(),
                    sql: (conf_tree).asStr("sql").unwrap().to_string(),
                    initial: (conf_tree).asF64("initial").unwrap(),
                    inputs,
                    vars: vars.clone(),
                }
        } else {
            panic!("MetricConfig.new | Configuration is empty")
        }
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub(crate) fn from_yaml(parent_id: &str, parent_name: &Name, value: &serde_yaml::Value, vars: &mut Vec<String>) -> MetricConfig {
        Self::new(parent_id, parent_name, &ConfTree::newRoot(value.clone()).next().unwrap(), vars)
    }
    ///
    /// reads config from path
    #[allow(dead_code)]
    pub fn read(parent_id: &str, parent_name: &Name, path: &str) -> MetricConfig {
        let mut vars = vec![];
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        MetricConfig::from_yaml(parent_id, parent_name, &config, &mut vars)
                    }
                    Err(err) => {
                        panic!("MetricConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("MetricConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        vec![]
    }
}
