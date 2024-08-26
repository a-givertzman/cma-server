use std::sync::{Arc, RwLock};
use indexmap::IndexMap;
use log::{trace, warn};
use sal_sync::services::entity::{name::Name, point::{point::Point, point_tx_id::PointTxId}};
use crate::{
    conf::{fn_::fn_conf_kind::FnConfKind, task_config::TaskConfig}, 
    core_::types::fn_in_out_ref::FnInOutRef, 
    services::{services::Services, task::nested_function::{fn_kind::FnKind, nested_fn::NestedFn}},
};
use super::{task_node_vars::TaskNodeVars, task_eval_node::TaskEvalNode};
///
/// TaskNodes - holds the IndexMap<String, TaskNode> in the following structure:
///   ```
///   {
///       inputName1: TaskEvalNode {
///           input: FnInOutRef,
///           outs: [
///               var1
///               var2
///               var...
///               metric1
///               metric2
///               metric...
///           ]
///       },
///       inputName2: TaskEvalNode {
///           ...
///       },
///   }
///   ```
#[derive(Debug)]
pub struct TaskNodes {
    id: String,
    nodes: IndexMap<String, TaskEvalNode>,
    vars: IndexMap<String, FnInOutRef>,
    new_node_vars: Option<TaskNodeVars>,
}
//
// 
impl TaskNodes {
    ///
    /// Creates new empty instance 
    pub fn new(parent: impl Into<String>) ->Self {
        Self {
            id: format!("{}/TaskNodes", parent.into()),
            nodes: IndexMap::new(),
            vars: IndexMap::new(),
            new_node_vars: None,
        }
    }
    ///
    /// Returns input by it's name
    pub fn get_eval_node(&mut self, name: &str) -> Option<&mut TaskEvalNode> {
        self.nodes.get_mut(name)
    }
    // ///
    // /// Returns input by it's name
    // pub fn get_input(&self, name: &str) -> Option<FnInOutRef> {
    //     self.inputs.get(name).map(|node| node.get_input())
    // }
    ///
    /// Returns variable by it's name
    pub fn get_var(&self, name: &str) -> Option<&FnInOutRef> {
        trace!("{}.getVar | trying to find variable {:?} in {:?}", self.id, &name, self.vars);
        self.vars.get(name)
    }
    ///
    /// Adding new input reference
    pub fn add_input(&mut self, name: impl Into<String> + std::fmt::Debug + Clone, input: FnInOutRef) -> FnInOutRef {
        let name = name.into();
        match self.new_node_vars {
            Some(_) => {
                match self.nodes.get_mut(&name) {
                    // Same name - adding to the existing node, if input has different 'options hash'
                    Some(node) => {
                        trace!("{}.add_input | input {:?}:{} - adding to the existing node if has different 'options hash'", self.id, name, input.borrow().hash());
                        node.add_input(input)
                    }
                    // New name - adding new TaskEvalNode
                    None => {
                        trace!("{}.add_input | adding input {:?}:{}", self.id, name, input.borrow().hash());
                        trace!("{}.add_input | adding input {:?}:{}: {:?}", self.id, name, input.borrow().hash(), input);
                        self.nodes.insert(
                            name.clone(), 
                            TaskEvalNode::new(&self.id, name, vec![input.clone()]),
                        );
                        input
                    }
                }
            }
            None => {
                panic!("{}.add_input | Call beginNewNode first, then you can add inputs", self.id)
            }
        }
    }
    ///
    /// Adding new variable refeerence
    pub fn add_var(&mut self, name: impl Into<String> + Clone, var: FnInOutRef) {
        // assert!(!self.vars.contains_key(name.clone().into().as_str()), "Dublicated variable name: {:?}", name.clone().into());
        assert!(!name.clone().into().is_empty(), "Variable name can't be emty");
        match self.new_node_vars {
            Some(_) => {
                if self.vars.contains_key(&name.clone().into()) {
                    panic!("{}.addVar | Dublicated variable name: {:?}", self.id, &name.clone().into());
                } else {
                    trace!("{}.addVar | adding variable {:?}", self.id, &name.clone().into());
                    trace!("{}.addVar | adding variable {:?}: {:?}", &name.clone().into(), self.id, &var);
                    self.vars.insert(
                        name.clone().into(),
                        var,
                    );
                }
                self.new_node_vars.as_mut().unwrap().addVar(name.clone().into());
            }
            None => panic!("{}.addVar | Error: call beginNewNode first, then you can add inputs", self.id),
        }
    }
    ///
    /// Adding already declared variable as out to the newNodeStuff
    pub fn add_var_out(&mut self, name: impl Into<String> + Clone) {
        assert!(!name.clone().into().is_empty(), "Variable name can't be emty");
        match self.new_node_vars {
            Some(_) => {
                self.new_node_vars.as_mut().unwrap().addVar(name.clone().into());
            }
            None => panic!("{}.addVarOut | Error: call beginNewNode first, then you can add inputs", self.id),
        }
    }    
    ///
    /// Call this method to finish configuration of jast created task node
    fn finish_new_node(&mut self, out: FnInOutRef) {
        match self.new_node_vars {
            Some(_) => {
                let mut vars: Vec<FnInOutRef> = vec![];
                for var_name in self.new_node_vars.as_mut().unwrap().getVars() {
                    match self.vars.get(&var_name) {
                        Some(var) => {
                            vars.push(
                                var.clone()
                            );
                        }
                        None => panic!("{}.finishNewNode | Variable {:?} - not found", self.id, var_name),
                    };
                };
                let inputs = out.borrow().inputs();
                trace!("{}.finishNewNode | out {:#?} \n\tdipending on inputs:: {:#?}\n", self.id, &out, inputs);
                for input_name in inputs {
                    match self.nodes.get_mut(&input_name) {
                        Some(eval_node) => {
                            trace!("{}.finishNewNode | updating input: {:?}", self.id, input_name);
                            let len = vars.len();
                            eval_node.add_vars(&vars.clone());
                            if out.borrow().kind() != &FnKind::Var {
                                eval_node.add_out(out.clone());
                            }
                            trace!("{}.finishNewNode | evalNode '{}' appended: {:?}", self.id, eval_node.name(), len);
                        }
                        None => panic!("{}.finishNewNode | Input {:?} - not found", self.id, input_name),
                    };
                };
                self.new_node_vars = None;
                trace!("\n{}.finishNewNode | self.inputs: {:?}\n", self.id, self.nodes);
            }
            None => panic!("{}.finishNewNode | Call beginNewNode first, then you can add inputs & vars, then finish node", self.id),
        }
    }
    ///
    /// Creates all task nodes depending on it config
    ///  - if Task config contains 'point [type] every' then single evaluation node allowed only
    pub fn build_nodes(&mut self, parent: &Name, conf: TaskConfig, services: Arc<RwLock<Services>>) {
        let tx_id = PointTxId::from_str(&parent.join());
        for (idx, (_node_name, mut node_conf)) in conf.nodes.into_iter().enumerate() {
            let node_name = node_conf.name();
            trace!("{}.build_nodes | node[{}]: {:?}", self.id, idx, node_name);
            self.new_node_vars = Some(TaskNodeVars::new());
            let out = match node_conf {
                FnConfKind::Fn(_) => {
                    NestedFn::new(parent, tx_id, &mut node_conf, self, services.clone())
                }
                FnConfKind::Var(_) => {
                    NestedFn::new(parent, tx_id, &mut node_conf, self, services.clone())
                }
                FnConfKind::Const(conf) => {
                    panic!("{}.build_nodes | Const is not supported in the root of the Task, config: {:?}: {:?}", self.id, node_name, conf);
                }
                FnConfKind::Point(conf) => {
                    panic!("{}.build_nodes | Point is not supported in the root of the Task, config: {:?}: {:?}", self.id, node_name, conf);
                }
                FnConfKind::PointConf(conf) => {
                    panic!("{}.build_nodes | PointConf is not supported in the root of the Task, config: {:?}: {:?}", self.id, node_name, conf);
                }
                FnConfKind::Param(conf) => {
                    panic!("{}.build_nodes | Param (custom parameter) is not supported in the root of the Task, config: {:?}: {:?} - ", self.id, node_name, conf);
                }
            };
            self.finish_new_node(out);
        }
        if let Some(eval_node) = self.get_eval_node("every") {
            let eval_node_name = eval_node.name();
            for (_name, input) in &self.nodes {
                let len = input.get_outs().len();
                if len > 1 {
                    panic!("{}.build_nodes | evalNode '{}' - contains {} Out's, but single Out allowed when 'point [type] every' was used", self.id, eval_node_name, len);
                }
            }
        }
    }
    ///
    /// Evaluates all containing node:
    ///  - adding new point
    ///  - evaluating each node
    pub fn eval(&mut self, point: Point) {
        let self_id = self.id.clone();
        let point_name = point.name();
        if let Some(eval_node) = self.get_eval_node("every") {
            trace!("{}.eval | evalNode '{}' - adding point...", self_id, &eval_node.name());
            eval_node.add(&point);
        };
        match self.get_eval_node(&point_name) {
            Some(eval_node) => {
                trace!("{}.eval | evalNode '{}' - adding point...", self_id, &eval_node.name());
                eval_node.add(&point);
                trace!("{}.eval | evalNode '{}' - evaluating...", self_id, &eval_node.name());
                eval_node.eval();
            }
            None => {
                if let Some(eval_node) = self.get_eval_node("every") {
                    trace!("{}.eval | evalNode '{}' - evaluating...", self_id, &eval_node.name());
                    eval_node.eval()
                } else {
                    warn!("{}.eval | evalNode '{}' - not fount, input point ignored", self.id, &point_name);
                }
            }
        };
    }
}