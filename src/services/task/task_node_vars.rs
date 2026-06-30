use sal_core::error::Error;

// use std::collections::HashMap;
// use crate::core_::FnInOutRef;
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
    pub fn add_var(&mut self, name: impl Into<String> + Clone) -> Result<(), Error> {
        let name = name.into();
        // assert!(!self.vars.contains(&name), "Dublicated variable name: {:?}", name);
        if name.is_empty() {
            return Err(Error::new("TaskNodeVars", "add_var").err("Variable name can't be emty"));
        }
        log::trace!("TaskNodeStuff.addVar | adding variable {:?}", name);
        self.vars.push(name);
        Ok(())
    }
    // ///
    // /// 
    // fn names(collection: &HashMap<String, FnInOutRef>) -> Vec<String> {
    //     collection.keys().cloned().collect()
    // }
    ///
    /// Returns all collected var names
    pub fn get_vars(&self) -> Vec<String> {
        self.vars.clone()
    }
    ///
    /// Returns len of the collection
    pub fn len(&self) -> usize {
        self.vars.len()
    }
}
