#[cfg(test)]

mod task_config_read {
    use indexmap::IndexMap;
    use log::{trace, info};
    use std::{sync::Once, env, time::Duration};
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
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let self_id = "task_config_new_test";
        let self_name = Name::new("", self_id);
        info!("{}", self_id);
        let target = TaskConfig {
            name: Name::new(&self_name, "Task1"),
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
        trace!("dir: {:?}", env::current_dir());
        let path = "./src/tests/unit/conf/task_config/task_config_test.yaml";
        let metric_config = TaskConfig::read(&self_name, path);
        trace!("fnConfig: {:?}", metric_config);
        assert_eq!(metric_config, target);
    }
}

