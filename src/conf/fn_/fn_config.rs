use indexmap::IndexMap;
use log::trace;
use std::{fs, str::FromStr};
use crate::conf::{
    conf_tree::ConfTree,
    fn_::{
        fn_conf_keywd::{FnConfKeywd, FnConfKindName, FnConfPointType}, fn_conf_kind::FnConfKind,
        fn_point_config::FnPointConfig, fn_conf_options::FnConfOptions,
    },
    point_config::{name::Name, point_config::PointConfig},
};
///
/// creates config read from yaml file of following format:
/// ```yaml
/// let VarName2:
///     input fn functionName:
///         initial: VarName1
///         input fn functionName:
///             input1: const someValue
///             input2: point '/path/Point.Name/'
///             input fn functionName:
///                 input: point '/path/Point.Name/'```
#[derive(Debug, PartialEq, Clone)]
pub struct FnConfig {
    pub name: String,
    pub inputs: IndexMap<String, FnConfKind>,
    pub type_: FnConfPointType,
    pub options: FnConfOptions,
}
//
//
impl Default for FnConfig {
    fn default() -> Self {
        Self { name: Default::default(), inputs: Default::default(), type_: Default::default(), options: Default::default() }
    }
}
//
// 
impl FnConfig {
    ///
    /// creates config from serde_yaml::Value of following format:
    /// ```yaml
    /// let VarName2:
    ///     input fn functionName:
    ///         initial: VarName1
    ///         input fn functionName:
    ///             input1: const someValue
    ///             input2: point '/path/Point.Name/'
    ///             input fn functionName:
    ///                 input: point '/path/Point.Name/'```
    pub fn new(parent_id: &str, parent_name: &Name, conf_tree: &ConfTree, vars: &mut Vec<String>) -> FnConfKind {
        trace!("FnConfig.new | confTree: {:?}", conf_tree);
        // self conf from first sub node
        //  - if additional sub nodes presents hit warning, FnConf must have single item
        // if confTree.count() > 1 {
        //     error!("FnConfig.new | FnConf must have single item, additional items was ignored: {:?}", confTree)
        // };
        let cfg = if conf_tree.is_mapping() {
            trace!("FnConfig.new | MAPPING VALUE: \t{:#?}", conf_tree);
            match FnConfKeywd::from_str(conf_tree.key.as_str()) {
                Ok(self_keyword) => {
                    trace!("FnConfig.new | selfKeyword parsed: {:?}", self_keyword);
                    // parse sub nodes
                    // let mut inputs = IndexMap::new();
                    trace!("FnConfig.new | build inputs...");
                    match self_keyword {
                        FnConfKeywd::Const(value) => {
                            let fn_name = if value.data.is_empty() {
                                conf_tree.conf.as_str().unwrap().to_string()
                            } else {
                                value.data
                            };
                            FnConfKind::Const(
                                FnConfig {
                                    name: fn_name,
                                    inputs: IndexMap::new(),
                                    type_: value.type_,
                                    options: FnConfOptions::default(),
                                }        
                            )
                        }
                        FnConfKeywd::Var(value) => {
                            vars.push(value.data.clone());
                            FnConfKind::Var(
                                FnConfig {
                                    name: value.data,
                                    inputs: Self::build_inputs(parent_id, parent_name, conf_tree, vars),
                                    type_: value.type_,
                                    options: FnConfOptions::default(),
                                }
                            )        
                        }
                        FnConfKeywd::Fn(value) => {
                            FnConfKind::Fn(
                                FnConfig {
                                    name: value.data,
                                    inputs: Self::build_inputs(parent_id, parent_name, conf_tree, vars),
                                    type_: value.type_,
                                    options: FnConfOptions::default(),
                                }
                            )
                        }
                        FnConfKeywd::Point(value) => {
                            trace!("FnConfig.new | Point: {:?}", value);
                            let result = Self::get_param_by_keyword(conf_tree, "enable", FnConfKindName::Const | FnConfKindName::Fn | FnConfKindName::Var | FnConfKindName::Point);
                            trace!("FnConfig.new | Point 'enable': {:?}", result);
                            let enable = match result {
                                Ok(conf) => {
                                    // debug!("FnConfig.new | Point 'enable' keyword: {:?}", keyword);
                                    Some(Box::new(FnConfig::new(parent_id, parent_name, &conf, vars)))
                                }
                                Err(_) => None,
                            };
                            let result = Self::get_param_by_keyword(conf_tree, "input", FnConfKindName::Const | FnConfKindName::Fn | FnConfKindName::Var | FnConfKindName::Point);
                            trace!("FnConfig.new | Point 'input': {:?}", result);
                            let input = match result {
                                Ok(conf) => {
                                    // debug!("FnConfig.new | Point 'input' keyword: {:?}", keyword);
                                    Some(Box::new(FnConfig::new(parent_id, parent_name, &conf, vars)))
                                }
                                Err(_) => None,
                            };
                            let result = Self::get_param_by_keyword(conf_tree, "changes-only", FnConfKindName::Const | FnConfKindName::Fn | FnConfKindName::Var | FnConfKindName::Point);
                            trace!("FnConfig.new | Point 'changes-only': {:?}", result);
                            let changes_only = match result {
                                Ok(conf) => {
                                    // debug!("FnConfig.new | Point 'changes-only' keyword: {:?}", keyword);
                                    Some(Box::new(FnConfig::new(parent_id, parent_name, &conf, vars)))
                                }
                                Err(_) => None,
                            };
                            FnConfKind::PointConf(
                                FnPointConfig {
                                    conf: PointConfig::new(parent_name, conf_tree),
                                    send_to: conf_tree.as_str("send-to").map_or(None, |v| Some(v.to_owned())),
                                    enable,
                                    input,
                                    changes_only,
                                }
                            )
                        }
                    }
                }
                // no keyword 
                //  - current node just an input name
                //      - take input Value / Fn from first sub node,
                //          if additional sub nodes prtesent, hit warning: "input must have single Value/Fn"
                Err(_) => {
                    trace!("FnConfig.new | Custom parameter '{}' declared: {:?}", conf_tree.key, conf_tree.conf);
                    FnConfKind::Param(conf_tree.to_owned())
                }
            }
        } else {
            trace!("FnConfig.new | SINGLE VALUE: \t{:#?}", &conf_tree.conf);
            if conf_tree.conf.is_string() {
                match FnConfKeywd::from_str(conf_tree.conf.as_str().unwrap()) {
                    // keyword parsed successefully
                    //  - take input name and input Value / Fn from the keyword
                    Ok(fn_keyword) => {
                        match fn_keyword {
                            FnConfKeywd::Const(value) => {
                                FnConfKind::Const(
                                    FnConfig {
                                        name: value.data,
                                        inputs: IndexMap::new(),
                                        type_: value.type_,
                                        options: FnConfOptions::default(),
                                    }
                                )
                            }
                            FnConfKeywd::Point(value) => {
                                FnConfKind::Point(
                                    FnConfig {
                                        name: value.data,
                                        inputs: IndexMap::new(),
                                        type_: value.type_,
                                        options: value.options,
                                    }
                                )
                            }
                            _ => {
                                panic!("FnConfig.new | Unknown keyword: {:?}", conf_tree.conf)
                            }
                        }
                    }
                    // no keyword 
                    //  - current node just an varible name
                    //  - or custom parameter
                    Err(_) => {
                        let var_name = conf_tree.conf.as_str().unwrap().to_string();
                        trace!("FnConfig.new | trying to find Variable: {:?} in vars: \n\t{:?}", &var_name, &vars);
                        if vars.contains(&var_name) {
                            trace!("FnConfig.new | Variable declared - ok: {:?}", conf_tree.conf);
                            FnConfKind::Var(
                                FnConfig { 
                                    name: var_name, 
                                    inputs: IndexMap::new(),
                                    type_: FnConfPointType::Unknown,
                                    options: FnConfOptions::default(),
                                }
                            )
                        } else {
                            trace!("FnConfig.new | Custom parameter '{}' declared: {:#?}", conf_tree.key, conf_tree.conf);
                            FnConfKind::Param(conf_tree.to_owned())
                        }
                    }
                }
            } else if conf_tree.conf.is_bool() {
                trace!("FnConfig.new | Custom parameter '{}' declared: {:?}", conf_tree.key, conf_tree.conf);
                FnConfKind::Param(conf_tree.to_owned())
            } else if conf_tree.conf.is_i64() {
                trace!("FnConfig.new | Custom parameter '{}' declared: {:?}", conf_tree.key, conf_tree.conf);
                FnConfKind::Param(conf_tree.to_owned())
            } else if conf_tree.conf.is_f64() {
                trace!("FnConfig.new | Custom parameter '{}' declared: {:?}", conf_tree.key, conf_tree.conf);
                FnConfKind::Param(conf_tree.to_owned())
            } else {
                panic!("FnConfig.new | Custom parameter '{}/{}' of unknown type declared, but : {:?}", parent_id, conf_tree.key, conf_tree.conf);
            }
        };
        trace!("FnConfig.new | Config created: {:#?}", cfg);
        cfg
    }
    ///
    /// Returns input ronfigurations in IndexMap
    fn build_inputs(parent_id: &str, parent_name: &Name, conf_tree: &ConfTree, vars: &mut Vec<String>) -> IndexMap<String, FnConfKind> {
        let mut inputs = IndexMap::new();
        match conf_tree.sub_nodes() {
            // has inputs in mapping
            Some(sub_nodes) => {
                trace!("FnConfig.buildInputs | sub nodes - found");
                for sub_node in sub_nodes {
                    trace!("FnConfig.buildInputs | sub node: {:?}", sub_node);
                    match FnConfKeywd::from_str(sub_node.key.as_str()) {
                        Ok(keyword) => {
                            trace!("FnConfig.buildInputs | sub node KEYWORD parsed: {:?}", keyword);
                            if !keyword.input().is_empty() {
                                inputs.insert(
                                    keyword.input(),
                                    FnConfig::new(parent_id, parent_name, &sub_node, vars),
                                );
                            }
                        }
                        Err(_) => {
                            trace!("FnConfig.buildInputs | sub node NO KEYWORD");
                            inputs.insert(
                                (sub_node).key.clone(), 
                                FnConfig::new(parent_id, parent_name, &sub_node, vars),
                            );
                        }
                    };
                }
            }
            None => {
                trace!("FnConfig.buildInputs | sub node not found, possible Const or Var");
                inputs.insert(
                    (conf_tree).key.clone(), 
                    FnConfig::new(parent_id, parent_name, conf_tree, vars),
                );
            }
        }
        inputs
    }
    ///
    /// creates config from serde_yaml::Value of following format:
    pub fn from_yaml(parent_id: &str, parent_name: &Name, value: &serde_yaml::Value, vars: &mut Vec<String>) -> FnConfKind {
        Self::new(parent_id, parent_name, &ConfTree::new_root(value.clone()).next().unwrap(), vars)
    }
    ///
    /// reads yaml config from path
    /// ```yaml
    /// let VarName2:
    ///     input fn functionName:
    ///         initial: VarName1
    ///         input fn functionName:
    ///             input1: const someValue
    ///             input2: point '/path/Point.Name/'
    ///             input fn functionName:
    ///                 input: point '/path/Point.Name/'```
    #[allow(dead_code)]
    pub fn read(parent_id: &str, parent_name: &Name, path: &str) -> FnConfKind {
        let mut vars = vec![];
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        FnConfig::from_yaml(parent_id, parent_name, &config, &mut vars)
                    }
                    Err(err) => {
                        panic!("FnConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("FnConfig.read | File {} reading error: {:?}", path, err)
            }
        }
    }
    ///
    /// Returns input config by it's name
    pub fn input_conf<'a>(&'a mut self, input_name: &str) -> Result<&'a mut FnConfKind, String> {
        match self.inputs.get_mut(input_name) {
            Some(conf) => Ok(conf),
            None => Err(format!("FnConfig.inputConf | function {:?} must have {:?}", self.name, input_name)),
        }
    }
    ///
    /// Returns custom parameter by it's name if exists, else none
    pub fn param(&self, name: &str) -> Result<&FnConfKind, String> {
        match self.inputs.get(name) {
            Some(param) => Ok(param),
            None => Err(format!("FnConfig.param | parameter {:?} not fount in the {:?}", name, self.name)),
        }
    }
    ///
    /// Returns ConfTree by keyword or Err
    fn get_param_by_keyword(conf: &ConfTree, input: &str, kind: u8) -> Result<ConfTree, String> {
        trace!("FnConfig.getParamByKeyword | conf: {:?}", conf);
        for node in conf.sub_nodes().unwrap() {
            trace!("FnConfig.getParamByKeyword | node: {:?}", node);
            match FnConfKeywd::from_str(&node.key) {
                Ok(keyword) => {
                    trace!("FnConfig.getParamByKeyword | keyword: {:?}, kind: {:?}", keyword, keyword.kind());
                    trace!("FnConfig.getParamByKeyword | keyword.kind({}) & kind({}): {:?}", (keyword.kind() as u8), kind, (keyword.kind() as u8) & kind);
                    if ((keyword.kind() as u8) & kind) > 0 && keyword.input() == input {
                        return Ok(node)
                    }
                }
                Err(_) => {
                    if node.key == input {
                        return Ok(node)
                    }
                }
            };
        };
        // Err(format!("{}.getParamByKeyword | keyword '{} {:?}' - not found", self.id, keywordPrefix, keywordKind))
        Err(format!("FnConfig.getParamByKeyword | keyword '{}' kind: {:?} - not found", input, kind))
    }
    ///
    /// Returns list of configurations of the defined points
    pub fn points(&self) -> Vec<PointConfig> {
        let mut points = vec![];
        trace!("FnConfig.points | requesting points...");
        for (input_name, input_kind) in &self.inputs {
            trace!("FnConfig({}).points | requesting points from {}: {:#?}...", self.name, input_name, input_kind.name());
            trace!("FnConfig({}).points | requesting points from {}: {:#?}...", self.name, input_name, input_kind);
            let mut input_points = match input_kind {
                FnConfKind::Fn(config) => {
                    config.points()
                }
                FnConfKind::Var(config) => {
                    config.points()
                }
                FnConfKind::Const(config) => {
                    config.points()
                }
                FnConfKind::Point(config) => {
                    config.points()
                }
                FnConfKind::PointConf(config) => {
                    config.points()
                }
                FnConfKind::Param(_) => {
                    vec![]
                }
            };
            points.append(&mut input_points);
        }
        points
    }
}
