use std::{cell::{Cell, RefCell}, rc::Rc, sync::Arc};
use indexmap::IndexMap;
use sal_core::error::Error;
use sal_sync::services::{entity::{Name, Point, PointTxId}, Services, task::functions::FnConfKind};
use crate::{
    domain::{FnInOutRef, FnOutRef}, 
    services::task::{EvalCycle, EvalCycleRef, FnEnableMode, FnEvalOnce, TaskRetain, functions::{FnBuilder, FnKind}, task_conf::TaskConf},
};
use super::{task_node_vars::TaskNodeVars, task_eval_node::TaskEvalNode};
///
/// ### TaskNodes - holds the IndexMap<String, TaskNode> in the following structure:
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
/// **every - Input wildcard**. Подстановка любого сигнала
/// - **Назначение**: `every` работает как wildcard (подстановочный знак) или глобальный триггер. Если узел подписан на `point type every` (например, `point any every` или `point int every`), то любое входящее событие (`Point`), переданное в `Task`, должно вызвать перерасчёт этого узла.
/// - **Типизация**: Дополнительный фильтр по типу (например, `point int every`) означает, что узел среагирует на любое событие, только если его значение имеет тип `int`. `point any every` реагирует вообще на всё.
/// - **Ограничение**: Все вычисления, зависящие от `every`, должны сводиться к одному корню (или одному выходному узлу).
#[derive(Debug)]
pub struct TaskNodes {
    txid: usize,
    retain: Arc<TaskRetain>,
    nodes: IndexMap<String, Rc<RefCell<TaskEvalNode>>>,
    vars: IndexMap<String, FnOutRef>,
    new_node_vars: Option<TaskNodeVars>,
    /// Текущий номер вычислительного цикла, инкремнтируется с каждым входом в `self.eval`
    cycle: EvalCycleRef,
    /// Enable Strategy: Cold Standby / Warm Standby (TODO: read from config)
    enable_mode: FnEnableMode,
    dbg: String,
}
//
// 
impl TaskNodes {
    ///
    /// Returns `TaskNodes` new instance 
    pub fn new(parent: impl Into<String>, txid: usize, retain: Arc<TaskRetain>,) ->Self {
        Self {
            txid,
            retain,
            nodes: IndexMap::new(),
            vars: IndexMap::new(),
            new_node_vars: None,
            cycle: Rc::new(EvalCycle::new()),
            enable_mode: FnEnableMode::Cold,
            dbg: format!("{}/TaskNodes", parent.into()),
        }
    }
    ///
    /// Returns `TaskNodes` new instance 
    pub fn without_retain(parent: impl Into<String>, txid: usize) ->Self {
        let dbg = format!("{}/TaskNodes", parent.into());
        Self {
            txid,
            retain: TaskRetain::mock(&dbg, []).into(),
            nodes: IndexMap::new(),
            vars: IndexMap::new(),
            new_node_vars: None,
            cycle: Rc::new(EvalCycle::new()),
            enable_mode: FnEnableMode::Cold,
            dbg,
        }
    }
    ///
    /// Returns `txid` of the parent `Task`
    pub fn txid(&self) -> usize {
        self.txid
    }
    ///
    /// Returns `retain` service of the parent `Task`
    pub fn retain(&self) -> Arc<TaskRetain> {
        self.retain.clone()
    }
    ///
    /// Returns Enable Strategy: Cold Standby / Warm Standby
    pub fn enable_mode(&self) -> FnEnableMode {
        self.enable_mode
    }
    ///
    /// ### Shared calculation cycle
    /// 
    /// Возвращает ссылку на текущий номер вычислительного цикла
    pub fn cycle(&self) -> EvalCycleRef {
        self.cycle.clone()
    }
    ///
    /// Returns all configured inputs
    pub fn get_inputs(&self) -> Vec<String> {
        self.nodes.keys().map(|k| k.to_string()).collect()
    }
    ///
    /// Returns input by it's name
    pub fn get_eval_node(&self, name: &str) -> Option<Rc<RefCell<TaskEvalNode>>> {
        self.nodes.get(name).map(|node| node.clone())
    }
    // ///
    // /// Returns input by it's name
    // pub fn get_input(&self, name: &str) -> Option<FnInOutRef> {
    //     self.inputs.get(name).map(|node| node.get_input())
    // }
    ///
    /// Returns variable by it's name
    pub fn get_var(&self, name: &str) -> Option<&FnOutRef> {
        log::trace!("{}.getVar | trying to find variable {:?} in {:?}", self.dbg, &name, self.vars);
        self.vars.get(name)
    }
    ///
    /// Adding new input reference
    pub fn add_input(&mut self, name: impl Into<String>, input: FnInOutRef) -> Result<FnOutRef, Error> {
        let name = name.into();
        match self.new_node_vars {
            Some(_) => {
                match self.nodes.get_mut(&name) {
                    // Same name - adding to the existing node, if input has different 'options hash'
                    Some(node) => {
                        log::trace!("{}.add_input | input {:?}:{} - adding to the existing node if has different 'options hash'", self.dbg, name, input.borrow().hash());
                        Ok(node.borrow_mut().add_input(input))
                    }
                    // New name - adding new TaskEvalNode
                    None => {
                        log::trace!("{}.add_input | adding input {:?}:{}", self.dbg, name, input.borrow().hash());
                        log::trace!("{}.add_input | adding input {:?}:{}: {:?}", self.dbg, name, input.borrow().hash(), input);
                        self.nodes.insert(
                            name.clone(), 
                            Rc::new(RefCell::new(TaskEvalNode::new(&self.dbg, name, vec![input.clone()]))),
                        );
                        Ok(input)
                    }
                }
            }
            None => Err(Error::new(&self.dbg, "add_input").err("Call begin_new_node first, then you can add inputs"))
        }
    }
    ///
    /// Adding new variable refeerence
    pub fn add_var(&mut self, name: impl Into<String>, var: FnOutRef) -> Result<(), Error> {
        let name = name.into();
        // assert!(!self.vars.contains_key(name.as_str()), "Dublicated variable name: {:?}", name);
        assert!(!name.is_empty(), "Variable name can't be emty");
        match self.new_node_vars.as_mut() {
            Some(new_node_vars) => {
                if self.vars.contains_key(&name) {
                    return Err(Error::new(&self.dbg, "add_var").err(format!("Dublicated variable name: {:?}", name)));
                } else {
                    log::trace!("{}.add_var | adding variable {:?}", self.dbg, &name);
                    log::trace!("{}.add_var | adding variable {:?}: {:?}", &name, self.dbg, &var);
                    self.vars.insert(
                        name.clone(),
                        var,
                    );
                }
                new_node_vars.add_var(name)
            }
            None => Err(Error::new(&self.dbg, "add_var").err(format!("Error: call beginNewNode first, then you can add inputs"))),
        }
    }
    ///
    /// Adding already declared variable as out to the newNodeStuff
    pub fn add_var_out(&mut self, name: impl Into<String>) -> Result<(), Error> {
        let name = name.into();
        assert!(!name.is_empty(), "Variable name can't be emty");
        match self.new_node_vars.as_mut() {
            Some(new_node_vars) => {
                new_node_vars.add_var(name).map_err(|err| Error::new(&self.dbg, "add_var_out").pass(err))
            }
            None => Err(Error::new(&self.dbg, "add_var_out").err("Call beginNewNode first, then you can add inputs")),
        }
    }    
    ///
    /// Call this method to finish configuration of jast created task node
    fn finish_new_node(&mut self, out: FnOutRef) -> Result<(), Error> {
        match self.new_node_vars.as_mut() {
            Some(new_node_vars) => {
                let mut vars: Vec<FnOutRef> = vec![];
                for var_name in new_node_vars.get_vars() {
                    match self.vars.get(&var_name) {
                        Some(var) => {
                            vars.push(
                                var.clone()
                            );
                        }
                        None => {
                            return Err(Error::new(&self.dbg, "finish_new_node").err(&format!("{}.finish_new_node | Variable {:?} - not found", self.dbg, var_name)))
                        }
                    };
                };
                let inputs = out.borrow().inputs();
                log::trace!("{}.finish_new_node | out {:#?} \n\tdipending on inputs:: {:#?}\n", self.dbg, &out, inputs);
                for input_name in inputs {
                    match self.nodes.get(&input_name) {
                        Some(eval_node) => {
                            log::trace!("{}.finish_new_node | updating input: {:?}", self.dbg, input_name);
                            let len = vars.len();
                            eval_node.borrow_mut().add_vars(&vars.clone());
                            if out.borrow().kind() != FnKind::Var {
                                eval_node.borrow_mut().add_out(out.clone());
                            }
                            log::trace!("{}.finish_new_node | evalNode '{}' appended: {:?}", self.dbg, eval_node.borrow().name(), len);
                        }
                        None => {
                            return Err(Error::new(&self.dbg, "finish_new_node").err(&format!("{}.finish_new_node | Input {:?} - not found", self.dbg, input_name)))
                        }
                    };
                };
                self.new_node_vars = None;
                log::trace!("\n{}.finish_new_node | self.inputs: {:?}\n", self.dbg, self.nodes);
            }
            None => {
                return Err(Error::new(&self.dbg, "finish_new_node").err(&format!("{}.finish_new_node | Call beginNewNode first, then you can add inputs & vars, then finish node", self.dbg)))
            }
        }
        Ok(())
    }
    ///
    /// Creates all task nodes depending on it config
    ///  - if Task config contains 'point [type] every' then single evaluation node allowed only
    pub fn build_nodes(&mut self, parent: &Name, conf: &TaskConf, services: Arc<Services>) -> Result<(), Error>{
        // TODO: Добавить проверку на ацикличность направленного графа (DAG). Например, алгоритм поиска в глубину (DFS) по связям inputs, проверяющий, не возвращаемся ли мы в уже посещенный узел.
        let error = Error::new(&self.dbg, "build_nodes");
        let tx_id = PointTxId::from_str(&parent.join());
        let conf_nodes = conf.nodes.clone();
        for (idx, (_node_name, mut node_conf)) in conf_nodes.into_iter().enumerate() {
            let node_name = node_conf.name();
            log::trace!("{}.build_nodes | node[{}]: {:?}", self.dbg, idx, node_name);
            self.new_node_vars = Some(TaskNodeVars::new());
            let out = match node_conf {
                FnConfKind::Fn(_) => {
                    Rc::new(RefCell::new(FnEvalOnce::new(parent, self.cycle.clone(), 
                        FnBuilder::new(parent, &mut node_conf, self, services.clone())
                            .map_err(|err| error.pass_with(format!("Can't build eval node '{node_name}': {:?}", conf), err))?,
                    )))
                }
                FnConfKind::Var(_) => {
                    Rc::new(RefCell::new(FnEvalOnce::new(parent, self.cycle.clone(), 
                    FnBuilder::new(parent, &mut node_conf, self, services.clone())
                            .map_err(|err| error.pass_with(format!("Can't build eval node '{node_name}': {:?}", conf), err))?,
                    )))
                }
                FnConfKind::Const(conf) => {
                    return Err(error.err(format!("Const is not supported in the root of the Task, config: {:?}: {:?}", node_name, conf)));
                }
                FnConfKind::Point(conf) => {
                    return Err(error.err(format!("Point is not supported in the root of the Task, config: {:?}: {:?}", node_name, conf)));
                }
                FnConfKind::PointConf(conf) => {
                    return Err(error.err(format!("PointConf is not supported in the root of the Task, config: {:?}: {:?}", node_name, conf)));
                }
                FnConfKind::Param(conf) => {
                    return Err(error.err(format!("Param (custom parameter) is not supported in the root of the Task, config: {:?}: {:?} - ", node_name, conf)));
                }
            };
            self.finish_new_node(out)
                .map_err(|err| error.pass_with(format!("Can't finish node {node_name}"), err))?;
        }
        // if let Some(eval_node) = self.get_eval_node("every") {
        //     let eval_node_name = eval_node.name();
        //     for (_name, input) in &self.nodes {
        //         let len = input.get_outs().len();
        //         if len > 1 {
        //             return Err(error.err(format!("evalNode '{}' - contains {} Out's, but single Out allowed when 'point [type] every' was used", eval_node_name, len)));
        //         }
        //     }
        // }
        Ok(())
    }
    ///
    /// Evaluates all containing node:
    ///  - adding new point
    ///  - evaluating each node
    pub fn eval(&self, point: Point) {
        let dbg = self.dbg.clone();
        self.cycle.increment();
        let point_name = point.name();
        let node_every = self.get_eval_node("every").map(|eval_node_every| {
            log::trace!("{dbg}.eval | evalNode '{}' - adding point...", &eval_node_every.borrow().name());
            eval_node_every.borrow().add(&point);
            eval_node_every
        });
        let node_spec = self.get_eval_node(&point_name).map(|eval_node| {
            log::trace!("{dbg}.eval | evalNode '{}' - adding point...", eval_node.borrow().name());
            eval_node.borrow().add(&point);
            eval_node
            // Some(eval_node) => {
            // }
            // None => {}
                // log::warn!("{dbg}.eval | evalNode '{}' - not fount, input point ignored", point_name);
        });
        if let Some(node) = node_every {
            log::trace!("{dbg}.eval | evalNode '{}' - evaluating...", node.borrow().name());
            node.borrow_mut().eval();
        }
        if let Some(node) = node_spec {
            log::trace!("{dbg}.eval | evalNode '{}' - evaluating...", node.borrow().name());
            node.borrow_mut().eval();
        }
    }
}