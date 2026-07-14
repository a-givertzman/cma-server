use sal_core::dbg::Dbg;
use sal_sync::services::entity::Point;
use crate::domain::{FnInOutRef, FnOutRef};
///
/// Holds Task input and all dipendent variables & outputs
#[derive(Debug)]
pub struct TaskEvalNode {
    name: String,
    input: Vec<FnInOutRef>,
    vars: Vec<FnOutRef>,
    outs: Vec<FnOutRef>,
    dbg: Dbg,
}
//
// 
impl TaskEvalNode {
    ///
    /// Creates new instance from input name, input it self and dependent vars & outs
    pub fn new(parent: impl Into<String>, name: impl Into<String>, input: Vec<FnInOutRef>) -> Self {
        let name = name.into();
        let dbg = Dbg::new(parent, &name);
        TaskEvalNode { 
            name, 
            input, 
            vars:  vec![],
            outs: vec![],
            dbg,
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
        log::trace!("TaskEvalNode.add_input | eval_node '{}' - input '{}' added", self.dbg, input.borrow().hash());
        input
    }
    ///
    /// 
    fn contains_var(&self, var: &FnOutRef) -> bool {
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
    fn contains_out(&self, out: &FnOutRef) -> bool {
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
    pub fn add_vars(&mut self, vars: &Vec<FnOutRef>) {
        for var in vars {
            if !self.contains_var(var) {
                self.vars.push(var.clone());
            }
        }
    }
    ///
    /// 
    pub fn add_out(&mut self, out: FnOutRef) {
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
    pub fn get_vars(&self) -> &Vec<FnOutRef> {
        &self.vars
    }
    ///
    /// 
    #[allow(unused)]
    pub fn get_outs(&self) -> &Vec<FnOutRef> {
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
        // Commented by AL. Looks like it's olready done by the calculation branch
        // for eval_node_var in &self.vars {
        //     log::trace!("TaskEvalNode.eval | node '{}' - var '{}' evaluating...", self.dbg, eval_node_var.borrow_mut().id());
        //     _ = eval_node_var.borrow_mut().out();
        //     log::trace!("TaskEvalNode.eval | node '{}' - var '{}' evaluated", self.dbg, eval_node_var.borrow_mut().id());
        // };
        for eval_node_out in &self.outs {
            log::trace!("TaskEvalNode.eval | node '{}' out...", self.dbg);
            match eval_node_out.borrow_mut().out() {
                Ok(Some(_)) => {
                    // log::debug!("TaskEvalNode.eval | node '{}' out: {:?}", self.id, out);
                }
                Ok(None) => if log::max_level() >= log::LevelFilter::Trace {
                    log::warn!("TaskEvalNode.eval | node '{}' out: 'None'", self.dbg);
                },
                Err(err) => if log::max_level() >= log::LevelFilter::Trace {
                    log::warn!("TaskEvalNode.eval | node '{}' out: {}", self.dbg, err);
                },
            }
        };
    }
}
