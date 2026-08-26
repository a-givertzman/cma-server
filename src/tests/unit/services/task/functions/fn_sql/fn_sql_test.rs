#[cfg(test)]

use regex::RegexBuilder;
use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::{Name, ToPoint}, Services}, thread_pool::ThreadPool};
use std::sync::{Once, Arc};
use debugging::session::{DebugSession, LogLevel};
use crate::services::task::{FlowContext, TaskConf, TaskNodes};
///
///
static INIT: Once = Once::new();
///
/// once called initialisation
fn init_once() {
    INIT.call_once(|| {
        // implement your initialisation code to be called only once for current test file
    })
}
///
/// returns:
///  - Rc<RefCell<Box<dyn FnInOut>>>...
// fn init_each() {
// }
///
///
#[test]
fn int() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    let dbg = "test_int";
    let self_name = Name::new("", dbg);
    log::debug!("\n{}", dbg);
    let conf = serde_yaml::from_str(r#"
        service Task task1:
            cycle: 1 us
            in queue api-link:
                max-length: 10000
            fn SqlMetric:
                sql: "UPDATE table_name SET kind = '{input1}' WHERE id = '{input2}';"
                input1 let VarName2:
                    input fn Add:
                        input1 fn Add:
                            input1: const int 1
                            input2: point int '/path/Point.Name'
                        input2: const int 1
                input2: const real 1.11
    "#).unwrap();
    let conf = TaskConf::from_yaml(&self_name, &conf).unwrap();
    log::trace!("conf: {:?}", conf);
    let mut nodes = TaskNodes::without_retain(dbg, 0);
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg,
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), Some(tp.scheduler())).unwrap());
    nodes.build_nodes(&self_name, &conf, services).unwrap();
    // log::debug!("taskNodes: {:?}", nodes);
    let test_data = vec![
        (01, 1, "/path/Point.Name", 3),
        (02, 1, "/path/Point.Name", 3),
        (03, 1, "/path/Point.Name", 3),
        (04, 1, "/path/Point.Name", 3),
        (05, 0, "/path/Point.Name", 2),
        (06, 1, "/path/Point.Name", 3),
        (07, 2, "/path/Point.Name", 4),
        (08, 3, "/path/Point.Name", 5),
        (09, 4, "/path/Point.Name", 6),
        (10, 5, "/path/Point.Name", 7),
        (11, 6, "/path/Point.Name", 8),
        (12, 7, "/path/Point.Name", 9),
        (13, 8, "/path/Point.Name", 10),
        (14, 9, "/path/Point.Name", 11),
    ];
    let flow = FlowContext::new();
    for (step, value, name, target_value) in test_data {
        let point = value.to_point(0, name);
        let input_name = point.name();
        nodes.eval(point);
        match nodes.get_eval_node(&input_name) {
            Some(eval_node) => {
                let eval_node_name = eval_node.borrow().name();
                for eval_node_out in eval_node.borrow().get_outs() {
                    log::trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node_name);
                    let out = flow.ignore(eval_node_out.borrow_mut().out());
                    match out {
                        Ok(Some(out)) => {
                            log::debug!("{dbg} | out: {:?}", out);
                            let out_value = out.to_string().as_string().value;
                            let target = format!("UPDATE table_name SET kind = '{}' WHERE id = '{}';", target_value, 1.11);
                            assert_eq!(out_value, target, "{dbg} | step {step} \nresult: {} \ntarget: {}", out_value, target);
                        }
                        Ok(None) => log::warn!("{dbg} | step {step}: evalNode '{}' out - '{}': None", eval_node_name, eval_node_out.borrow().id()),
                        Err(err) => log::warn!("{dbg} | step {step}: evalNode '{}' out - '{}' is Error: {:#?}", eval_node_name, eval_node_out.borrow().id(), err),
                    }
                }
            }
            None => {
                panic!("{dbg} | step {step}: input {:?} - not found in the current taskNodes", &input_name)
            }
        };
    }
}
///
///
#[test]
fn real() {
    DebugSession::new().filter(LogLevel::Info).init().unwrap();
    init_once();
    let dbg = "test_real";
    let self_name = Name::new("", dbg);
    log::debug!("\n{}", dbg);
    let conf = serde_yaml::from_str(r#"
        service Task task1:
            cycle: 1 us
            in queue api-link:
                max-length: 10000
            fn SqlMetric:
                sql: "UPDATE table_name SET kind = '{input1:.2}' WHERE id = '{input2:.2}';"
                input1 let VarName2:
                    input fn Add:
                        input1 fn Add:
                            input1: const real 1.1
                            input2: point real '/path/Point.Name'
                        input2: const real 1.1
                input2: const real 3.33
    "#).unwrap();
    let conf = TaskConf::from_yaml(dbg, &conf).unwrap();
    log::trace!("conf: {:?}", conf);
    let mut nodes = TaskNodes::without_retain(dbg, 0);
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg,
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), Some(tp.scheduler())).unwrap());
    nodes.build_nodes(&self_name, &conf, services).unwrap();
    // log::debug!("taskNodes: {:?}", nodes);
    let test_data = vec![
        (01, 1.1f32, "/path/Point.Name", 3.3f32),
        (02, 1.2f32, "/path/Point.Name", 3.4),
        (03, 1.3f32, "/path/Point.Name", 3.5),
        (04, 1.4f32, "/path/Point.Name", 3.6),
        (05, 0.1f32, "/path/Point.Name", 2.3),
        (06, 1.1f32, "/path/Point.Name", 3.3),
        (07, 2.2f32, "/path/Point.Name", 4.4),
        (08, 3.3f32, "/path/Point.Name", 5.5),
        (09, 4.4f32, "/path/Point.Name", 6.6),
        (10, 5.5f32, "/path/Point.Name", 7.7),
        (11, 6.6f32, "/path/Point.Name", 8.8),
        (12, 7.7f32, "/path/Point.Name", 9.9),
        (13, 8.8f32, "/path/Point.Name", 11.0),
        (14, 9.9f32, "/path/Point.Name", 12.1),
    ];
    let flow = FlowContext::new();
    for (step, value, name, target_value) in test_data {
        let point = value.to_point(0, name);
        let input_name = point.name();
        nodes.eval(point.clone());
        match nodes.get_eval_node(&input_name) {
            Some(eval_node) => {
                let eval_node_name = eval_node.borrow().name();
                for eval_node_out in eval_node.borrow().get_outs() {
                    log::trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node_name);
                    let out = flow.ignore(eval_node_out.borrow_mut().out());
                    match out {
                        Ok(Some(out)) => {
                            log::debug!("{dbg} | step {step}: out: {:?}", out);
                            let out_value = out.to_string().as_string().value;
                            let re = r"(UPDATE table_name SET kind = ')(\d+(?:\.\d+)*)(' WHERE id = '3.33';)";
                            log::trace!("re: {}", re);
                            let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
                            let digits: f64 = re.captures(&out_value).unwrap().get(2).unwrap().as_str().parse().unwrap();
                            let digits = format!("{:.1}", digits);
                            log::trace!("digits: {:?}", digits);
                            let out = re.replace(&out_value, "$1{!}$3");
                            let out = out.replace("{!}", &digits);
                            log::trace!("out: {}", out);
                            log::debug!("{dbg} | step {step}:   value: {:?}   |   state: {:?}", point.as_real().value, out_value);
                            let target = format!("UPDATE table_name SET kind = '{:.2}' WHERE id = '{:.2}';",target_value, 3.33);
                            assert_eq!(out_value, target, "{dbg} | step {step} \nresult: {} \ntarget: {}", out_value, target);
                        }
                        Ok(None) => log::warn!("{dbg} | step {step}: evalNode '{}' out - '{}': None", eval_node_name, eval_node_out.borrow().id()),
                        Err(err) => log::warn!("{dbg} | step {step}: evalNode '{}' out - '{}' is Error: {:#?}", eval_node_name, eval_node_out.borrow().id(), err),
                    }
                }
            }
            None => {
                panic!("{dbg} | step {step}: input {:?} - not found in the current taskNodes", &input_name)
            }
        };
    }
}
///
///
#[test]
fn double() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    let dbg = "test_real";
    let self_name = Name::new("", dbg);
    log::debug!("\n{}", dbg);
    let conf = serde_yaml::from_str(r#"
        service Task task1:
            cycle: 1 us
            in queue api-link:
                max-length: 10000
            fn SqlMetric:
                sql: "UPDATE table_name SET kind = '{input1:.2}' WHERE id = '{input2:.2}';"
                input1 let VarName2:
                    input fn Add:
                        input1 fn Add:
                            input1: const double 1.1
                            input2: point double '/path/Point.Name'
                        input2: const double 1.1
                input2: const double 3.33
    "#).unwrap();
    let conf = TaskConf::from_yaml(dbg, &conf).unwrap();
    log::trace!("conf: {:?}", conf);
    let mut nodes = TaskNodes::without_retain(dbg, 0);
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg,
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), None).unwrap());
    nodes.build_nodes(&self_name, &conf, services).unwrap();
    // log::trace!("taskNodes: {:?}", nodes);
    let test_data = vec![
        (01, 1.1f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 1.1f64),
        (02, 1.2f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 1.2f64),
        (03, 1.3f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 1.3f64),
        (04, 1.4f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 1.4f64),
        (05, 0.1f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 0.1f64),
        (06, 1.1f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 1.1f64),
        (07, 2.2f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 2.2f64),
        (08, 3.3f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 3.3f64),
        (09, 4.4f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 4.4f64),
        (10, 5.5f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 5.5f64),
        (11, 6.6f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 6.6f64),
        (12, 7.7f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 7.7f64),
        (13, 8.8f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 8.8f64),
        (14, 9.9f64, "/path/Point.Name", 1.1f64 + 1.1f64 + 9.9f64),
    ];
    let flow = FlowContext::new();
    for (step, value, name, target_value) in test_data {
        let point = value.to_point(0, name);
        let input_name = &point.name();
        nodes.eval(point.clone());
        match nodes.get_eval_node(&input_name) {
            Some(eval_node) => {
                let eval_node_name = eval_node.borrow().name();
                for eval_node_out in eval_node.borrow().get_outs() {
                    log::trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node_name);
                    let out = flow.ignore(eval_node_out.borrow_mut().out());
                    match out {
                        Ok(Some(out)) => {
                            log::debug!("{dbg} | out: {:?}", out);
                            let out_value = out.to_string().as_string().value;
                            let re = r"(UPDATE table_name SET kind = ')(\d+(?:\.\d+)*)(' WHERE id = '3.33';)";
                            log::trace!("re: {}", re);
                            let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
                            let digits: f64 = re.captures(&out_value).unwrap().get(2).unwrap().as_str().parse().unwrap();
                            let digits = format!("{:.1}", digits);
                            log::trace!("digits: {:?}", digits);
                            let out = re.replace(&out_value, "$1{!}$3");
                            let out = out.replace("{!}", &digits);
                            log::trace!("out: {}", out);
                            log::debug!("value: {:?}   |   state: {:?}", point.as_double().value, out_value);
                            let target = format!("UPDATE table_name SET kind = '{:.2}' WHERE id = '{:.2}';",target_value, 3.33f64);
                            assert_eq!(out_value, target, "{dbg} | step {step} \nresult: {} \ntarget: {}", out_value, target);
                        }
                        Ok(None) => log::warn!("{dbg} | step {step}: evalNode '{}' out - '{}': None", eval_node_name, eval_node_out.borrow().id()),
                        Err(err) => log::warn!("{dbg} | step {step}: evalNode '{}' out - '{}' is Error: {:#?}", eval_node_name, eval_node_out.borrow().id(), err),
                    };
                }
            }
            None => {
                panic!("{dbg} | step {step}: input {:?} - not found in the current taskNodes", &input_name)
            }
        };
    }
}
