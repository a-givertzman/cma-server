#[cfg(test)]

mod task_config_new {
    use indexmap::IndexMap;
    use log::{info, debug};
    use std::{sync::Once, time::Duration};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::conf::{
        conf_subscribe::ConfSubscribe, conf_tree::ConfTree, fn_::{fn_conf_keywd::FnConfPointType, fn_conf_kind::FnConfKind, fn_conf_options::FnConfOptions, fn_config::FnConfig}, point_config::name::Name, task_config::TaskConfig
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
    ///  - ...
    fn init_each() -> () {}
    ///
    ///
    #[test]
    fn valid() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        let self_id = "task_config_new_test";
        let self_name = Name::new("", self_id);
        info!("{}", self_id);
        // let (initial, switches) = init_each();
        let test_data = [
            // (
            //     r#"metric SqlMetric:
            //         table: "table_name"
            //         sql: "select * from {table}"
            //         inputs:
            //             input: const '13.55'
            //     "#,
            //     FnConfig { fnType: FnConfigType::Var, name: "newVar".to_string(), inputs: IndexMap::from([
            //         ("input".to_string(), FnConfig { fnType: FnConfigType::Const, name: "13.55".to_string(), inputs: IndexMap::new() }),
            //     ]) }
            // ),
            (
                r#"service Task Task0:
                    cycle: 100 ms
                    in queue recv-queue:
                        max-length: 10000
                    fn SqlMetric:
                        initial: 123.123      # начальное значение
                        table: table_name
                        sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
                        input1: point any every
                        input2 fn PointId:
                            input: point any every
                        input3 fn PointId:
                            input: point int every
                        input4 fn PointId:
                            input: point real every
                "#,
                TaskConfig {
                    name: Name::new(&self_name, "Task0"),
                    cycle: Some(Duration::from_millis(100)),
                    rx: format!("recv-queue"),
                    rx_max_length: 10000,
                    subscribe: ConfSubscribe::new(serde_yaml::Value::Null),
                    vars: vec![],
                    nodes: IndexMap::from([
                        (format!("SqlMetric-1"), FnConfKind::Fn( FnConfig {
                                type_: FnConfPointType::Unknown,
                                name: format!("SqlMetric"),
                                inputs: IndexMap::from([
                                    (format!("initial"), FnConfKind::Param( ConfTree::new("initial", serde_yaml::from_str("123.123").unwrap()) )),
                                    (format!("table"), FnConfKind::Param( ConfTree::new("table", serde_yaml::from_str("table_name").unwrap()) )),
                                    (format!("sql"), FnConfKind::Param( ConfTree::new("sql", serde_yaml::from_str("UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';").unwrap()) )),
                                    (format!("input1"), FnConfKind::Point( FnConfig { name: format!("every"), type_: FnConfPointType::Any, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                    (format!("input2"), FnConfKind::Fn( FnConfig {
                                        name: format!("PointId"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                            (format!("input"), FnConfKind::Point( FnConfig { name: format!("every"), type_: FnConfPointType::Any, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                        ]),
                                        options: FnConfOptions::default(),
                                    } )),
                                    (format!("input3"), FnConfKind::Fn( FnConfig {
                                        name: format!("PointId"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                            (format!("input"), FnConfKind::Point( FnConfig { name: format!("every"), type_: FnConfPointType::Int, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                        ]),
                                        options: FnConfOptions::default(),
                                    } )),
                                    (format!("input4"), FnConfKind::Fn( FnConfig {
                                        name: format!("PointId"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                            (format!("input"), FnConfKind::Point( FnConfig { name: format!("every"), type_: FnConfPointType::Real, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                        ]),
                                        options: FnConfOptions::default(),
                                    } )),
                                ]),
                                options: FnConfOptions::default(),
                            } ),
                        ),
                    ]),
                },
            ),
            (
                r#"service Task Task1:
                    cycle: 100 ms
                    in queue recv-queue:
                        max-length: 10000
                    fn SqlMetric:
                        initial: 0.123      # начальное значение
                        table: table_name
                        sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
                        input1 let VarName2:
                            input fn functionName:
                                initial: VarName2
                                input fn functionName:
                                    input1: const someValue
                                    input2: point real '/path/Point.Name'
                                    input fn functionName:
                                        input: point bool '/path/Point.Name'
                        input2:
                            const int 1
                "#,
                TaskConfig {
                    name: Name::new(&self_name, "Task1"),
                    cycle: Some(Duration::from_millis(100)),
                    rx: format!("recv-queue"),
                    rx_max_length: 10000,
                    subscribe: ConfSubscribe::new(serde_yaml::Value::Null),
                    vars: vec![format!("VarName2")],
                    nodes: IndexMap::from([
                        (format!("SqlMetric-1"), FnConfKind::Fn( FnConfig {
                                type_: FnConfPointType::Unknown,
                                name: format!("SqlMetric"),
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
                                                            (format!("input2"), FnConfKind::Point( FnConfig { name: format!("/path/Point.Name"), type_: FnConfPointType::Real, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
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
                                            } )),
                                        ]),
                                        options: FnConfOptions::default(),
                                    } )),
                                    (format!("input2"), FnConfKind::Const( FnConfig { name: format!("1"), type_: FnConfPointType::Int, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                ]),
                                options: FnConfOptions::default(),
                            } ),
                        ),
                    ]),
                },
            ),
            (
                r#"service Task Task2:
                    cycle: 100 ms
                    in queue recv-queue:
                        max-length: 10000
                    let VarName2:
                        input fn functionName:
                            initial: VarName2
                            input fn functionName:
                                input1: const someValue
                                input2: point real '/path/Point.Name'
                                input fn functionName:
                                    input: point bool '/path/Point.Name'
                "#,
                TaskConfig {
                    name: Name::new(&self_name, "Task2"),
                    cycle: Some(Duration::from_millis(100)),
                    rx: format!("recv-queue"),
                    rx_max_length: 10000,
                    subscribe: ConfSubscribe::new(serde_yaml::Value::Null),
                    vars: vec![format!("VarName2")],
                    nodes: IndexMap::from([
                        (format!("VarName2-1"), FnConfKind::Var( FnConfig {
                            name: format!("VarName2"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                (format!("input"), FnConfKind::Fn( FnConfig {
                                    name: format!("functionName"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                        (format!("initial"), FnConfKind::Var( FnConfig { name: format!("VarName2"), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                                        (format!("input"), FnConfKind::Fn( FnConfig {
                                            name: format!("functionName"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                                (format!("input1"), FnConfKind::Const( FnConfig { name: format!("someValue"), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                                (format!("input2"), FnConfKind::Point( FnConfig { name: format!("/path/Point.Name"), type_: FnConfPointType::Real, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
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
                    ])
                }
            ),
        ];
        for (value, target) in test_data {
            debug!("test value: {:?}", value);
            let conf: serde_yaml::Value = serde_yaml::from_str(value).unwrap();
            debug!("value: {:?}   |   conf: {:?}   |   target: {:?}", "_", conf, target);
            // let fnKeyword = FnConfigKeyword::from_str(conf.as_str().unwrap()).unwrap();
            // debug!("\tfnKeyword: {:?}", fnKeyword);
            // let mut vars = vec![];
            let fn_config = TaskConfig::from_yaml(&self_name, &conf);
            debug!("\tfnConfig: {:?}", fn_config);
            assert_eq!(fn_config, target, "\n result: {:#?}\n target: {:#?}", fn_config, target);
        }
    }
}