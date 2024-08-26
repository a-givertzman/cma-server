#[cfg(test)]

mod sql_metric {
    use log::trace;
    use log::debug;
    use log::warn;
    use regex::RegexBuilder;
    use std::sync::RwLock;
    use std::sync::{Once, Arc};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::conf::point_config::name::Name;
    use crate::services::task::nested_function::fn_result::FnResult;
    use crate::{
        conf::task_config::TaskConfig,
        core_::point::point::{ToPoint, Point},
        services::{
            task::task_nodes::TaskNodes, services::Services,
            // queues::queues::Queues,
        },
    };
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
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "test_int";
        let self_name = Name::new("", self_id);
        debug!("\n{}", self_id);
        let path = "./src/tests/unit/services/task/sql_metric/sql_metric_int_test.yaml";
        let conf = TaskConfig::read(&self_name, path);
        debug!("conf: {:?}", conf);
        let mut nodes = TaskNodes::new(self_id);
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        nodes.build_nodes(&self_name, conf, services);
        debug!("taskNodes: {:?}", nodes);
        let test_data = vec![
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
        for (value, name, target_value) in test_data {
            let point = value.to_point(0, name);
            let input_name = &point.name();
            match nodes.get_eval_node(&input_name) {
                Some(eval_node) => {
                    eval_node.add(&point);
                    for eval_node_var in eval_node.get_vars() {
                        trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", eval_node.name(), eval_node_var.borrow().id());
                        eval_node_var.borrow_mut().eval();
                        debug!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", eval_node.name(), eval_node_var.borrow().id());
                    };
                    for eval_node_out in eval_node.get_outs() {
                        trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node.name());
                        let out = eval_node_out.borrow_mut().out();
                        match out {
                            FnResult::Ok(out) => {
                                let out_value = match &out {
                                    Point::Bool(point) => point.value.to_string(),
                                    Point::Int(point) => point.value.to_string(),
                                    Point::Real(point) => point.value.to_string(),
                                    Point::Double(point) => point.value.to_string(),
                                    Point::String(point) => point.value.clone(),
                                };
                                debug!("TaskEvalNode.eval | evalNode '{}' out - '{}': {:?}", eval_node.name(), eval_node_out.borrow().id(), out);
                                assert_eq!(
                                    out_value,
                                    format!("UPDATE SelectMetric_test_table_name SET kind = '{:.1}' WHERE id = '{}';",target_value, 1.11),
                                    // format!("insert into SelectMetric_test_table_name values(id, value, timestamp) (SqlMetric,{:.3},{})", targetValue, point.timestamp())
                                );
                            }
                            FnResult::None => warn!("TaskEvalNode.eval | evalNode '{}' out - '{}': None", eval_node.name(), eval_node_out.borrow().id()),
                            FnResult::Err(err) => warn!("TaskEvalNode.eval | evalNode '{}' out - '{}' is Error: {:#?}", eval_node.name(), eval_node_out.borrow().id(), err),
                        } 
                    }
                }
                None => {
                    panic!("input {:?} - not found in the current taskNodes", &input_name)
                }
            };
        }
    }
    ///
    ///
    #[test]
    fn real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "test_real";
        let self_name = Name::new("", self_id);
        debug!("\n{}", self_id);
        let path = "./src/tests/unit/services/task/sql_metric/sql_metric_real_test.yaml";
        let conf = TaskConfig::read(&self_name, path);
        debug!("conf: {:?}", conf);
        let mut nodes = TaskNodes::new(self_id);
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        nodes.build_nodes(&self_name, conf, services);
        debug!("taskNodes: {:?}", nodes);
        let test_data = vec![
            (1.1f32, "/path/Point.Name", 3.3f32),
            (1.2f32, "/path/Point.Name", 3.4),
            (1.3f32, "/path/Point.Name", 3.5),
            (1.4f32, "/path/Point.Name", 3.6),
            (0.1f32, "/path/Point.Name", 2.3),
            (1.1f32, "/path/Point.Name", 3.3),
            (2.2f32, "/path/Point.Name", 4.4),
            (3.3f32, "/path/Point.Name", 5.5),
            (4.4f32, "/path/Point.Name", 6.6),
            (5.5f32, "/path/Point.Name", 7.7),
            (6.6f32, "/path/Point.Name", 8.8),
            (7.7f32, "/path/Point.Name", 9.9),
            (8.8f32, "/path/Point.Name", 11.0),
            (9.9f32, "/path/Point.Name", 12.1),
        ];
        for (value, name, target_value) in test_data {
            let point = value.to_point(0, name);
            let input_name = &point.name();
            match nodes.get_eval_node(&input_name) {
                Some(eval_node) => {
                    eval_node.add(&point);
                    for eval_node_var in eval_node.get_vars() {
                        trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", eval_node.name(), eval_node_var.borrow().id());
                        eval_node_var.borrow_mut().eval();
                        debug!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", eval_node.name(), eval_node_var.borrow().id());
                    };
                    for eval_node_out in eval_node.get_outs() {
                        trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node.name());
                        let out = eval_node_out.borrow_mut().out();
                        match out {
                            FnResult::Ok(out) => {
                                let out_value = match &out {
                                    Point::Bool(point) => point.value.to_string(),
                                    Point::Int(point) => point.value.to_string(),
                                    Point::Real(point) => point.value.to_string(),
                                    Point::Double(point) => point.value.to_string(),
                                    Point::String(point) => point.value.clone(),
                                };
                                debug!("TaskEvalNode.eval | evalNode '{}' out - '{}': {:?}", eval_node.name(), eval_node_out.borrow().id(), out);
                                let re = r"(UPDATE SelectMetric_test_table_name SET kind = ')(\d+(?:\.\d+)*)(' WHERE id = '3.33';)";
                                trace!("re: {}", re);
                                let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
                                let digits: f64 = re.captures(&out_value).unwrap().get(2).unwrap().as_str().parse().unwrap();
                                let digits = format!("{:.1}", digits);
                                trace!("digits: {:?}", digits);
                                let out = re.replace(&out_value, "$1{!}$3");
                                let out = out.replace("{!}", &digits);
                                trace!("out: {}", out);
                                debug!("value: {:?}   |   state: {:?}", point.as_real().value, out_value);
                                assert_eq!(
                                    out,
                                    format!("UPDATE SelectMetric_test_table_name SET kind = '{:.1}' WHERE id = '{}';",target_value, 3.33),
                                    // format!("insert into SelectMetric_test_table_name values(id, value, timestamp) (SqlMetric,{:.3},{})", targetValue, point.timestamp())
                                );
                            }
                            FnResult::None => warn!("TaskEvalNode.eval | evalNode '{}' out - '{}': None", eval_node.name(), eval_node_out.borrow().id()),
                            FnResult::Err(err) => warn!("TaskEvalNode.eval | evalNode '{}' out - '{}' is Error: {:#?}", eval_node.name(), eval_node_out.borrow().id(), err),
                        }
                    }
                }
                None => {
                    panic!("input {:?} - not found in the current taskNodes", &input_name)
                }
            };
        }
    }
    ///
    ///
    #[test]
    fn double() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "test_real";
        let self_name = Name::new("", self_id);
        debug!("\n{}", self_id);
        let path = "./src/tests/unit/services/task/sql_metric/sql_metric_double_test.yaml";
        let conf = TaskConfig::read(&self_name, path);
        debug!("conf: {:?}", conf);
        let mut nodes = TaskNodes::new(self_id);
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        nodes.build_nodes(&self_name, conf, services);
        debug!("taskNodes: {:?}", nodes);
        let test_data = vec![
            (1.1f64, "/path/Point.Name", 3.3),
            (1.2f64, "/path/Point.Name", 3.4),
            (1.3f64, "/path/Point.Name", 3.5),
            (1.4f64, "/path/Point.Name", 3.6),
            (0.1f64, "/path/Point.Name", 2.3),
            (1.1f64, "/path/Point.Name", 3.3),
            (2.2f64, "/path/Point.Name", 4.4),
            (3.3f64, "/path/Point.Name", 5.5),
            (4.4f64, "/path/Point.Name", 6.6),
            (5.5f64, "/path/Point.Name", 7.7),
            (6.6f64, "/path/Point.Name", 8.8),
            (7.7f64, "/path/Point.Name", 9.9),
            (8.8f64, "/path/Point.Name", 11.0),
            (9.9f64, "/path/Point.Name", 12.1),
        ];
        for (value, name, target_value) in test_data {
            let point = value.to_point(0, name);
            let input_name = &point.name();
            match nodes.get_eval_node(&input_name) {
                Some(eval_node) => {
                    eval_node.add(&point);
                    for eval_node_var in eval_node.get_vars() {
                        trace!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluating...", eval_node.name(), eval_node_var.borrow().id());
                        eval_node_var.borrow_mut().eval();
                        debug!("TaskEvalNode.eval | evalNode '{}' - var '{}' evaluated", eval_node.name(), eval_node_var.borrow().id());
                    };
                    for eval_node_out in eval_node.get_outs() {
                        trace!("TaskEvalNode.eval | evalNode '{}' out...", eval_node.name());
                        let out = eval_node_out.borrow_mut().out();
                        match out {
                            FnResult::Ok(out) => {
                                let out_value = match &out {
                                    Point::Bool(point) => point.value.to_string(),
                                    Point::Int(point) => point.value.to_string(),
                                    Point::Real(point) => point.value.to_string(),
                                    Point::Double(point) => point.value.to_string(),
                                    Point::String(point) => point.value.clone(),
                                };
                                debug!("TaskEvalNode.eval | evalNode '{}' out - '{}': {:?}", eval_node.name(), eval_node_out.borrow().id(), out);
                                let re = r"(UPDATE SelectMetric_test_table_name SET kind = ')(\d+(?:\.\d+)*)(' WHERE id = '3.33';)";
                                trace!("re: {}", re);
                                let re = RegexBuilder::new(&re).multi_line(false).build().unwrap();
                                let digits: f64 = re.captures(&out_value).unwrap().get(2).unwrap().as_str().parse().unwrap();
                                let digits = format!("{:.1}", digits);
                                trace!("digits: {:?}", digits);
                                let out = re.replace(&out_value, "$1{!}$3");
                                let out = out.replace("{!}", &digits);
                                trace!("out: {}", out);
                                debug!("value: {:?}   |   state: {:?}", point.as_double().value, out_value);
                                assert_eq!(
                                    out,
                                    format!("UPDATE SelectMetric_test_table_name SET kind = '{:.1}' WHERE id = '{}';",target_value, 3.33),
                                    // format!("insert into SelectMetric_test_table_name values(id, value, timestamp) (SqlMetric,{:.3},{})", targetValue, point.timestamp())
                                );
                            }
                            FnResult::None => warn!("TaskEvalNode.eval | evalNode '{}' out - '{}': None", eval_node.name(), eval_node_out.borrow().id()),
                            FnResult::Err(err) => warn!("TaskEvalNode.eval | evalNode '{}' out - '{}' is Error: {:#?}", eval_node.name(), eval_node_out.borrow().id(), err),
                        };
                    }
                }
                None => {
                    panic!("input {:?} - not found in the current taskNodes", &input_name)
                }
            };
        }
    }
}
