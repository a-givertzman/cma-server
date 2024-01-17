#![allow(non_snake_case)]
#[cfg(test)]
use log::trace;
use log::{debug, info};
use regex::RegexBuilder;
use std::sync::{Once, Arc, Mutex};

use crate::{
    conf::{fn_config::FnConfig, task_config::TaskConfig}, 
    core_::{
        debug::debug_session::{DebugSession, LogLevel, Backtrace}, 
        point::point_type::{ToPoint, PointType}, 
    }, 
    services::{
        task::{task_nodes::TaskNodes, nested_function::{nested_fn, sql_metric::SqlMetric}}, services::Services,
        // queues::queues::Queues,
    },
};

// Note this useful idiom: importing names from outer (for mod tests) scope.
// use super::*;

static INIT: Once = Once::new();

///
/// once called initialisation
fn initOnce() {
    INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        }
    )
}


///
/// returns:
///  - Rc<RefCell<Box<dyn FnInOut>>>...
// fn initEach() {
// }


#[test]
fn test_int() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    initOnce();
    println!("");
    let selfId = "test_int";
    println!("{}", selfId);
    let path = "./src/tests/unit/services/task/metric/sql_metric_int_test.yaml";
    let conf = TaskConfig::read(path);
    debug!("conf: {:?}", conf);
    let mut nodes = TaskNodes::new(selfId);
    let services = Arc::new(Mutex::new(Services::new(selfId)));
    nodes.buildNodes("test", conf, services);
    debug!("taskNodes: {:?}", nodes);
    let testData = vec![
        (1, "/path/Point.Name", 3),
        (1, "/path/Point.Name", 3),
        (1, "/path/Point.Name", 3),
        (1, "/path/Point.Name", 3),
        (0, "/path/Point.Name", 2),
        (1, "/path/Point.Name", 3),
        (2, "/path/Point.Name", 4),
        (3, "/path/Point.Name", 5),
        (4, "/path/Point.Name", 6),
        (5, "/path/Point.Name", 7),
        (6, "/path/Point.Name", 8),
        (7, "/path/Point.Name", 9),
        (8, "/path/Point.Name", 10),
        (9, "/path/Point.Name", 11),
    ];
    for (value, name, targetValue) in testData {
        let point = value.toPoint(0, name);
        let inputName = &point.name();
        match &nodes.getEvalNode(&inputName) {
            Some(evalNode) => {
                let input = evalNode.getInput();
                input.borrow_mut().add(point.clone());
                for evalNodeVar in evalNode.getVars() {
                    trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", evalNode.name(), evalNodeVar.borrow().id());
                    evalNodeVar.borrow_mut().eval();
                    debug!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", evalNode.name(), evalNodeVar.borrow().id());
                };
                for evalNodeOut in evalNode.getOuts() {
                    trace!("TaskEvalNode.eval | evalNode '{}' out...", evalNode.name());
                    let out = evalNodeOut.borrow_mut().out();
                    let outValue = match &out {
                        PointType::Bool(point) => point.value.to_string(),
                        PointType::Int(point) => point.value.to_string(),
                        PointType::Float(point) => point.value.to_string(),
                        PointType::String(point) => point.value.clone(),
                    };
                    debug!("TaskEvalNode.eval | evalNode '{}' out - '{}': {:?}", evalNode.name(), evalNodeOut.borrow().id(), out);
                    assert_eq!(
                        outValue, 
                        format!("UPDATE SelectMetric_test_table_name SET kind = '{:.1}' WHERE id = '{}';",targetValue, 1.11),
                        // format!("insert into SelectMetric_test_table_name values(id, value, timestamp) (SqlMetric,{:.3},{})", targetValue, point.timestamp())
                    );

                    // match out {
                    //     TaskNodeType::Var(var) => {
                    //         var.borrow_mut().eval();
                    //         debug!("var evalueted: {:?}", var.borrow_mut().out());
                    //     },
                    //     TaskNodeType::Metric(metric) => {
                    //         // debug!("input: {:?}", &input);
                    //         let state = metric.borrow_mut().out();
                    //         let out = state.asString().value;
                    //         trace!("out: {}", out);                    
                    //         debug!("value: {:?}   |   state: {:?}", point.asInt().value, state.asString().value);
                    //         assert_eq!(
                    //             out, 
                    //             format!("UPDATE SelectMetric_test_table_name SET kind = '{:.1}' WHERE id = '{}';",targetValue, 1.11),
                    //             // format!("insert into SelectMetric_test_table_name values(id, value, timestamp) (SqlMetric,{:.3},{})", targetValue, point.timestamp())
                    //         );
                    //     },
                    // }
                }
            },
            None => {
                panic!("input {:?} - not found in the current taskNodes", &inputName)
            },
        };
    }        
}


#[test]
fn test_float() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    initOnce();
    println!("");
    let selfId = "test_float";
    println!("{}", selfId);
    let path = "./src/tests/unit/services/task/metric/sql_metric_float_test.yaml";
    let conf = TaskConfig::read(path);
    debug!("conf: {:?}", conf);
    let mut nodes = TaskNodes::new(selfId);
    let services = Arc::new(Mutex::new(Services::new(selfId)));
    nodes.buildNodes("test", conf, services);
    debug!("taskNodes: {:?}", nodes);
    let testData = vec![
        (1.1, "/path/Point.Name", 3.3),
        (1.2, "/path/Point.Name", 3.4),
        (1.3, "/path/Point.Name", 3.5),
        (1.4, "/path/Point.Name", 3.6),
        (0.1, "/path/Point.Name", 2.3),
        (1.1, "/path/Point.Name", 3.3),
        (2.2, "/path/Point.Name", 4.4),
        (3.3, "/path/Point.Name", 5.5),
        (4.4, "/path/Point.Name", 6.6),
        (5.5, "/path/Point.Name", 7.7),
        (6.6, "/path/Point.Name", 8.8),
        (7.7, "/path/Point.Name", 9.9),
        (8.8, "/path/Point.Name", 11.0),
        (9.9, "/path/Point.Name", 12.1),
    ];
    for (value, name, targetValue) in testData {
        let point = value.toPoint(0, name);
        let inputName = &point.name();
        match nodes.getEvalNode(&inputName) {
            Some(evalNode) => {
                let input = evalNode.getInput();
                input.borrow_mut().add(point.clone());
                for evalNodeVar in evalNode.getVars() {
                    trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", evalNode.name(), evalNodeVar.borrow().id());
                    evalNodeVar.borrow_mut().eval();
                    debug!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", evalNode.name(), evalNodeVar.borrow().id());
                };
                for evalNodeOut in evalNode.getOuts() {
                    trace!("TaskEvalNode.eval | evalNode '{}' out...", evalNode.name());
                    let out = evalNodeOut.borrow_mut().out();
                    let outValue = match &out {
                        PointType::Bool(point) => point.value.to_string(),
                        PointType::Int(point) => point.value.to_string(),
                        PointType::Float(point) => point.value.to_string(),
                        PointType::String(point) => point.value.clone(),
                    };
                    debug!("TaskEvalNode.eval | evalNode '{}' out - '{}': {:?}", evalNode.name(), evalNodeOut.borrow().id(), out);

                    let re = r"(UPDATE SelectMetric_test_table_name SET kind = ')(\d+(?:\.\d+)*)(' WHERE id = '3.33';)";
                    trace!("re: {}", re);
                    let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
                    let digits: f64 = re.captures(&outValue).unwrap().get(2).unwrap().as_str().parse().unwrap();
                    let digits = format!("{:.1}", digits);
                    trace!("digits: {:?}", digits);
                    let out = re.replace(&outValue, "$1{!}$3");
                    let out = out.replace("{!}", &digits);
                    trace!("out: {}", out);
            
                    debug!("value: {:?}   |   state: {:?}", point.asFloat().value, outValue);
                    assert_eq!(
                        out, 
                        format!("UPDATE SelectMetric_test_table_name SET kind = '{:.1}' WHERE id = '{}';",targetValue, 3.33),
                        // format!("insert into SelectMetric_test_table_name values(id, value, timestamp) (SqlMetric,{:.3},{})", targetValue, point.timestamp())
                    );

                    // match out {
                    //     TaskNodeType::Var(var) => {
                    //         var.borrow_mut().eval();
                    //         debug!("var evalueted: {:?}", var.borrow_mut().out());
                    //     },
                    //     TaskNodeType::Metric(metric) => {
                    //         // debug!("input: {:?}", &input);
                    //         let state = metric.borrow_mut().out();
                    //         let out = state.asString().value;
                    //         let re = r"(UPDATE SelectMetric_test_table_name SET kind = ')(\d+(?:\.\d+)*)(' WHERE id = '3.33';)";
                    //         trace!("re: {}", re);
                    //         let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
                    //         let digits: f64 = re.captures(&out).unwrap().get(2).unwrap().as_str().parse().unwrap();
                    //         let digits = format!("{:.1}", digits);
                    //         trace!("digits: {:?}", digits);
                    //         let out = re.replace(&out, "$1{!}$3");
                    //         let out = out.replace("{!}", &digits);
                    //         trace!("out: {}", out);
                    
                    //         debug!("value: {:?}   |   state: {:?}", point.asFloat().value, state.asString().value);
                    //         assert_eq!(
                    //             out, 
                    //             format!("UPDATE SelectMetric_test_table_name SET kind = '{:.1}' WHERE id = '{}';",targetValue, 3.33),
                    //             // format!("insert into SelectMetric_test_table_name values(id, value, timestamp) (SqlMetric,{:.3},{})", targetValue, point.timestamp())
                    //         );
                    //     },
                    // }
                }
            },
            None => {
                panic!("input {:?} - not found in the current taskNodes", &inputName)
            },
        };
    }        
}
