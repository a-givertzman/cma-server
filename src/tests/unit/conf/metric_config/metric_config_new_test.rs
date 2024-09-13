#[cfg(test)]

mod tests {
    use log::debug;
    use sal_sync::services::{entity::name::Name, task::functions::conf::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions}};
    use std::sync::Once;
    use indexmap::IndexMap;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::conf::fn_::{fn_conf_kind::FnConfKind, fn_config::FnConfig, metric_config::MetricConfig};
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
    fn test_metric_config_new_valid() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test MetricConfig | new valid";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        // let (initial, switches) = init_each();
        let test_data = [
            // (
            //     r#"fn SqlMetric:
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
                r#"fn SqlMetric:
                    initial: 0.123      # начальное значение
                    table: table_name
                    sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
                    inputs:
                        input1:
                            let VarName2:
                                input fn functionName:
                                    initial: VarName2
                                    input11 fn functionName:
                                        input1: const someValue
                                        input2: point int '/path/Point.Name/'
                                        input fn functionName:
                                            input: point bool '/path/Point.Name/'
                        input2:
                            const 1
                "#,
                MetricConfig {
                    name: format!("SqlMetric"),
                    table: format!("table_name"),
                    sql: String::from("UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"),
                    initial: 0.123,
                    vars: vec![format!("VarName2")],
                    inputs: IndexMap::from([
                        (format!("input1"), FnConfKind::Var( FnConfig {
                            name: format!("VarName2"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                (format!("input"), FnConfKind::Fn( FnConfig {
                                    name: format!("functionName"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                        (format!("initial"), FnConfKind::Var( FnConfig { name: format!("VarName2"), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                        (format!("input11"), FnConfKind::Fn( FnConfig {
                                            name: format!("functionName"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                                (format!("input1"), FnConfKind::Const( FnConfig { name: format!("someValue"), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                                (format!("input2"), FnConfKind::Point( FnConfig { name: format!("/path/Point.Name/"), type_: FnConfPointType::Int, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                                                (format!("input"), FnConfKind::Fn( FnConfig {
                                                    name: format!("functionName"), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                                        (format!("input"), FnConfKind::Point( FnConfig { name: format!("/path/Point.Name/"), type_: FnConfPointType::Bool, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
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
                        (format!("input2"), FnConfKind::Const( FnConfig { name: format!("1"), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default() } )),
                    ]),
                },
            ),
        ];
        for (value, target) in test_data {
            debug!("test value: {:?}", value);
            let conf: serde_yaml::Value = serde_yaml::from_str(value).unwrap();

            debug!("value: {:?}   |   conf: {:?}   |   target: {:?}", "_", conf, target);
            // let fnKeyword = FnConfigKeyword::from_str(conf.as_str().unwrap()).unwrap();
            // debug!("\tfnKeyword: {:?}", fnKeyword);
            let mut vars = vec![];
            let fn_config = MetricConfig::from_yaml(self_id, &self_name, &conf, &mut vars);
            debug!("\tfnConfig: {:?}", fn_config);
            assert_eq!(fn_config, target);
        }
    }
}