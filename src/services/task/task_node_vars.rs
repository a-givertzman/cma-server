#![allow(non_snake_case)]

use std::collections::HashMap;

use log::trace;

use crate::core_::types::fn_in_out_ref::FnInOutRef;


///
/// A container for storing variable names 
/// during configuring single TaskEvalNode only
#[derive(Debug)]
pub struct TaskNodeVars {
    vars: Vec<String>,
}
impl TaskNodeVars {
    ///
    /// Creates new container for storing variable & input names
    /// during configuring single TaskEvalNode only
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
        }
    }
    ///
    /// Adding new variable name
    pub fn addVar(&mut self, name: impl Into<String> + Clone) {
        // assert!(!self.vars.contains(name.clone().into().as_str()), "Dublicated variable name: {:?}", name.clone().into());
        assert!(!name.clone().into().is_empty(), "Variable name can't be emty");
        trace!("TaskNodeStuff.addVar | adding variable {:?}", name.clone().into());
        self.vars.push(name.into());
    }
    ///
    /// 
    fn names(collection: &HashMap<String, FnInOutRef>) -> Vec<String> {
        collection.keys().map(|v| v.clone()).collect()
    }
    ///
    /// Returns all collected var names
    pub fn getVars(&self) -> Vec<String> {
        self.vars.clone()
    }
}
