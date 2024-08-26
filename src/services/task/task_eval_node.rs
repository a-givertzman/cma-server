use log::trace;
use crate::core_::{types::fn_in_out_ref::FnInOutRef, point::point::Point};
///
/// Holds Task input and all dipendent variables & outputs
#[derive(Debug)]
pub struct TaskEvalNode {
    id: String,
    name: String,
    input: Vec<FnInOutRef>,
    vars: Vec<FnInOutRef>,
    outs: Vec<FnInOutRef>,
}
//
// 
impl TaskEvalNode {
    ///
    /// Creates new instance from input name, input it self and dependent vars & outs
    pub fn new(parent_name: impl Into<String>, name: impl Into<String>, input: Vec<FnInOutRef>) -> Self {
        let self_name = name.into();
        TaskEvalNode { 
            id: format!("{}/{}", parent_name.into(), &self_name), 
            name: self_name, 
            input, 
            vars:  vec![],
            outs: vec![],
        }
    }
    ///
    /// Adds input if it's has different 'Options hash'
    /// Returns added new or found existing input
    pub fn add_input(&mut self, input: FnInOutRef) -> FnInOutRef {
        let input_hash = input.borrow().hash();
        for input in &self.input {
            let hash = input.borrow().hash();
            if input_hash == hash {
                return input.clone();
            }
        }
        self.input.push(input.clone());
        trace!("TaskEvalNode.add_input | evalNode '{}' - input '{}' added", self.id, input.borrow().hash());
        input
    }
    ///
    /// 
    fn contains_var(&self, var: &FnInOutRef) -> bool {
        let var_id = var.borrow().id();
        for self_var in &self.vars {
            if self_var.borrow().id() == var_id {
                return true;
            }
        }
        false
    }
    ///
    /// 
    fn contains_out(&self, out: &FnInOutRef) -> bool {
        let out_id = out.borrow().id();
        for self_out in &self.outs {
            if self_out.borrow().id() == out_id {
                return true;
            }
        }
        false
    }
    ///
    /// 
    pub fn add_vars(&mut self, vars: &Vec<FnInOutRef>) {
        for var in vars {
            if !self.contains_var(var) {
                self.vars.push(var.clone());
            }
        }
    }
    ///
    /// 
    pub fn add_out(&mut self, out: FnInOutRef) {
        if !self.contains_out(&out) {
            self.outs.push(out);
        }
    }
    ///
    /// Returns self name
    pub fn name(&self) -> String {
        self.name.clone()
    }
    // ///
    // /// 
    // pub fn get_input(&self) -> Vec<FnInOutRef> {
    //     self.input.clone()
    // }
    ///
    ///
    #[allow(unused)]
    pub fn get_vars(&self) -> &Vec<FnInOutRef> {
        &self.vars
    }
    ///
    /// 
    pub fn get_outs(&self) -> &Vec<FnInOutRef> {
        &self.outs
    }
    ///
    /// Adds new point to the holding input reference
    pub fn add(&self, point: &Point) {
        for input in &self.input {
            input.borrow_mut().add(point);
        }
    }
    ///
    /// Evaluates node:
    ///  - eval all conaining vars
    ///  - eval all conaining outs
    pub fn eval(&mut self) {
        for eval_node_var in &self.vars {
            trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", self.id, eval_node_var.borrow_mut().id());
            eval_node_var.borrow_mut().eval();
            trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", self.id, eval_node_var.borrow_mut().id());
        };
        for eval_node_out in &self.outs {
            trace!("TaskEvalNode.eval | evalNode '{}' out...", self.id);
            let out = eval_node_out.borrow_mut().out();
            trace!("TaskEvalNode.eval | evalNode '{}' out: {:?}", self.id, out);
        };
    }
}