#[cfg(test)]

use indexmap::IndexMap;
use sal_sync::services::{conf::ConfTree, entity::Name, ConfSubscribe, task::functions::{FnConfKind, FnConfig, FnConfPointType, FnConfOptions}};
use std::{sync::Once, env, time::Duration};
use debugging::session::{DebugSession, LogLevel};
use crate::services::task::TaskConf;
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
///  - ...
fn init_each() -> () {}
///
///
#[test]
fn valid() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let self_id = "task_config_new_test";
    let self_name = Name::new("", self_id);
    log::info!("{}", self_id);
    let target = TaskConf {
        name: Name::new(&self_name, "Task1"),
        retain: Default::default(),
        cycle: Some(Duration::from_millis(100)),
        rx: format!("recv-queue"),
        rx_max_length: 10000,
        subscribe: ConfSubscribe::new(serde_yaml::Value::Null),
        vars: vec![format!("VarName2")],
        nodes: IndexMap::from([
            (format!("SqlMetric-1"), FnConfKind::Fn( FnConfig {
                    name: format!("SqlMetric"),
                    type_: FnConfPointType::Unknown,
                    // table: format!("table_name"),
                    // sql: format!("UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"),
                    // initial: 0.123,
                    // vars: vec![format!("VarName2")],
                    inputs: IndexMap::from([
                        (format!("initial"), FnConfKind::Param( ConfTree::new("initial", serde_yaml::from_str("0.123").unwrap()) )),
                        (format!("table"), FnConfKind::Param( ConfTree::new("table", serde_yaml::from_str("table_name").unwrap()) )),
                        (format!("sql"), FnConfKind::Param( ConfTree::new("sql", serde_yaml::from_str("UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';").unwrap()) )),
                        (format!("input1"), FnConfKind::Var( FnConfig {
                            name: format!("VarName2"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                (format!("input"), FnConfKind::Fn( FnConfig {
                                    name: format!("functionName"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                        (format!("initial"), FnConfKind::Var( FnConfig { name: format!("VarName2"), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                        (format!("input"), FnConfKind::Fn( FnConfig {
                                            name: format!("functionName"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                                (format!("input1"), FnConfKind::Const( FnConfig { name: format!("someValue"), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                                (format!("input2"), FnConfKind::Point( FnConfig { name: format!("/path/Point.Name"), type_: FnConfPointType::Real, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                                                (format!("input"), FnConfKind::Fn( FnConfig {
                                                    name: format!("functionName"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                                        (format!("input"), FnConfKind::Point( FnConfig { name: format!("/path/Point.Name"), type_: FnConfPointType::Bool, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                                    ]),
                                                    options: FnConfOptions::default(),
                                                } )),
                                            ]),
                                            options: FnConfOptions::default(),
                                        } )),
                                    ]),
                                    options: FnConfOptions::default(),
                                } ))
                            ]),
                            options: FnConfOptions::default(),
                        } )),
                        (format!("input2"), FnConfKind::Const( FnConfig { name: format!("1"), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                        (format!("input3"), FnConfKind::Point( FnConfig { name: format!("every"), type_: FnConfPointType::Any, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                        (format!("input4"), FnConfKind::Fn( FnConfig {
                            name: format!("PointId"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                (format!("input"), FnConfKind::Point( FnConfig { name: format!("every"), type_: FnConfPointType::Any, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                            ]),
                            options: FnConfOptions::default(),
                        } )),
                        (format!("input5"), FnConfKind::Fn( FnConfig {
                            name: format!("PointId"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                (format!("input"), FnConfKind::Point( FnConfig { name: format!("every"), type_: FnConfPointType::Int, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                            ]),
                            options: FnConfOptions::default(),
                        } )),
                        (format!("input6"), FnConfKind::Fn( FnConfig {
                            name: format!("PointId"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                (format!("input"), FnConfKind::Point( FnConfig { name: format!("every"), type_: FnConfPointType::Real, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                            ]),
                            options: FnConfOptions::default(),
                        } )),
                    ]),
                    options: FnConfOptions::default(),
                } )
            ),
        ])
    };
    log::trace!("dir: {:?}", env::current_dir());
    let path = "src/tests/unit/services/task/task_conf/task_config_test.yaml";
    let result = TaskConf::read(&self_name, path).unwrap();
    log::trace!("fnConfig: {:?}", result);
    assert_eq!(result, target, "\nresult: {:?}, \ntarget: {:?}", result, target);
}
