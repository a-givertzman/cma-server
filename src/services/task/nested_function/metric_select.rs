#![allow(non_snake_case)]

use std::collections::HashMap;

use indexmap::IndexMap;
use log::{debug, error, trace, warn};

use crate::{
    core_::{conf::fn_config::FnConfig, point::{point_type::{PointType, ToPoint}, point::Point}, format::format::Format, types::fn_in_out_ref::FnInOutRef}, 
    services::{task::task_nodes::TaskNodes, queues::queues::Queues},
};

use super::{fn_::{FnInOut, FnOut, FnIn}, nested_fn::NestedFn};


///
/// Counts number of raised fronts of boolean input
#[derive(Debug)]
pub struct MetricSelect {
    id: String,
    inputs: IndexMap<String, FnInOutRef>,
    initial: f64,
    table: String,
    sql: Format,
    sqlNames: HashMap<String, (String, Option<String>)>,
}
///
/// 
impl MetricSelect {
    //
    //
    pub fn new(conf: &mut FnConfig, taskNodes: &mut TaskNodes, queues: &mut Queues) -> MetricSelect {
        let mut inputs = IndexMap::new();
        let inputConfs = conf.inputs.clone();
        let inputConfNames = inputConfs.keys().filter(|v| {
            let delete = match v.as_str() {
                "initial" => true,
                "table" => true,
                "sql" => true,
                _ => false
            };
            !delete
        });
        // let v: Vec<&String> = inputConfNames.collect();
        // inputConfs.remove("initial");
        // inputConfs.remove("table");
        // inputConfs.remove("sql");
        for name in inputConfNames {
            debug!("MetricSelect.new | input name: {:?}", name);
            let inputConf = conf.inputConf(&name);
            inputs.insert(
                name.to_string(), 
                NestedFn::new(inputConf, taskNodes, queues),
            );
        }
        let id = conf.name.clone();
        let initial = conf.param("initial").name.parse().unwrap();
        let table = conf.param("table").name.clone();
        let mut sql = Format::new(&conf.param("sql").name);
        sql.insert("id", id.clone().toPoint(""));
        sql.insert("table", table.clone().toPoint(""));
        sql.prepare();
        let mut sqlNames = sql.names();
        sqlNames.remove("initial");
        sqlNames.remove("table");
        sqlNames.remove("sql");
        sqlNames.remove("id");
        MetricSelect {
            id: id,
            inputs: inputs,
            initial: initial,
            table: table,
            sql,
            sqlNames: sqlNames,
        }
    }
}
///
/// 
impl FnIn for MetricSelect {
    fn add(&mut self, point: PointType) {
        panic!("MetricSelect.add | method is not used")
    }
}
///
/// 
impl FnOut for MetricSelect {
    //
    fn out(&mut self) -> PointType {
        for (fullName, (name, sufix)) in &self.sqlNames {
            trace!("MetricSelect.out | name: {:?}, sufix: {:?}", &name, &sufix);
            match self.inputs.get(name) {
                Some(input) => {
                    trace!("MetricSelect.out | input: {:?} - found", &name);
                    let point = input.borrow_mut().out();
                    self.sql.insert(&fullName, point);
                },
                None => {
                    panic!("MetricSelect.out | input: {:?} - not found", &name);
                },
            };
        }
        debug!("MetricSelect.out | sql: {:?}", self.sql.out());
        PointType::String(Point::newString(
            "MetricSelect.out", 
            self.sql.out(),
        ))
    }
    //
    fn reset(&mut self) {
        todo!()
    }
}
///
/// 
impl FnInOut for MetricSelect {}
