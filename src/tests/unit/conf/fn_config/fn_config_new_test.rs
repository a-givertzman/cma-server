#[cfg(test)]

mod tests {
    use log::debug;
    use std::sync::Once;
    use indexmap::IndexMap;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::conf::{conf_tree::ConfTree, fn_::{fn_conf_keywd::FnConfPointType, fn_conf_kind::FnConfKind, fn_conf_options::FnConfOptions, fn_config::FnConfig}, point_config::name::Name};
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
    fn test_fn_config_new_valid() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test FnConfig | new valid";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_data = [
            (
                r#"let newVar:
                    input: const '13.55'
                "#,
                FnConfKind::Var( FnConfig { name: "newVar".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                    ("input".to_string(), FnConfKind::Const( FnConfig { name: "13.55".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), })),
                ]), options: FnConfOptions::default() })
            ),
            (
                r#"let newVar:
                    input fn Count:
                        inputConst1: const '13.3'
                        inputConst2: const '13.7'
                "#,
                FnConfKind::Var( FnConfig { name: "newVar".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                    ("input".to_string(), FnConfKind::Fn( FnConfig { name: "Count".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                        ("inputConst1".to_string(), FnConfKind::Const( FnConfig { name: "13.3".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                        ("inputConst2".to_string(), FnConfKind::Const( FnConfig { name: "13.7".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                    ]), options: FnConfOptions::default(), } )),
                ]), options: FnConfOptions::default(), } )
            ),
            (
                r#"let newVar:
                    input1 fn Count:
                        inputConst1: const '11.3'
                        inputConst2: const '12.7'"
                    input2 fn Count:
                        inputConst1: const real '13.3'
                        inputConst2: const int '147'
                "#,
                FnConfKind::Var( FnConfig { name: "newVar".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                    ("input1".to_string(), FnConfKind::Fn( FnConfig { name: "Count".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                        ("inputConst1".to_string(), FnConfKind::Const( FnConfig { name: "11.3".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                        ("inputConst2".to_string(), FnConfKind::Const( FnConfig { name: "12.7".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                    ]), options: FnConfOptions::default(), } )),
                    ("input2".to_string(), FnConfKind::Fn( FnConfig { name: "Count".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                        ("inputConst1".to_string(), FnConfKind::Const( FnConfig { name: "13.3".to_string(), type_: FnConfPointType::Real, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                        ("inputConst2".to_string(), FnConfKind::Const( FnConfig { name: "147".to_string(), type_: FnConfPointType::Int, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                    ]), options: FnConfOptions::default(), } )),
                ]), options: FnConfOptions::default(), } )
            ),
            (
                r#"let VarName2:
                    param: "string param"
                    input fn functionName1:
                        initial: VarName2
                        input fn functionName2:
                            input1: const someValue
                            input2: point int '/path/Point.Name/'
                            input3 fn functionName3:
                                    input: point bool '/path/Point.Name/'
                "#,
                FnConfKind::Var( FnConfig { name: "VarName2".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                    ("param".to_string(), FnConfKind::Param( ConfTree::new("param", serde_yaml::from_str("string param").unwrap()) )),
                    ("input".to_string(), FnConfKind::Fn( FnConfig { name: "functionName1".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                        ("initial".to_string(), FnConfKind::Var( FnConfig { name: "VarName2".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                        ("input".to_string(), FnConfKind::Fn( FnConfig { name: "functionName2".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                            ("input1".to_string(), FnConfKind::Const( FnConfig { name: "someValue".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                            ("input2".to_string(), FnConfKind::Point( FnConfig { name: "/path/Point.Name/".to_string(), type_: FnConfPointType::Int, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                            ("input3".to_string(), FnConfKind::Fn( FnConfig { name: "functionName3".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                ("input".to_string(), FnConfKind::Point( FnConfig { name: "/path/Point.Name/".to_string(), type_: FnConfPointType::Bool, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                            ]), options: FnConfOptions::default(), } )),
                        ]), options: FnConfOptions::default(), } )),
                    ]), options: FnConfOptions::default(), } )),
                ]), options: FnConfOptions::default(), } )
            ),
            (
                r#"fn metricName1:
                    initial: 0.123
                    table: SelectMetric_test_table_name
                    sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
                    input fn functionName1:
                        initial: const int 1234567
                        input fn functionName2:
                            input1: const someValue
                            input2: point int '/path/Point.Name/'
                            input3 fn functionName3:
                                    input: point bool '/path/Point.Name/'
                "#,
                FnConfKind::Fn( FnConfig { name: "metricName1".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                    ("initial".to_string(), FnConfKind::Param( ConfTree::new("initial", serde_yaml::from_str("0.123").unwrap()) )),
                    ("table".to_string(), FnConfKind::Param( ConfTree::new("table", serde_yaml::from_str("SelectMetric_test_table_name").unwrap()) )),
                    ("sql".to_string(), FnConfKind::Param( ConfTree::new("sql", serde_yaml::from_str("UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';").unwrap()) )),
                    ("input".to_string(), FnConfKind::Fn( FnConfig { name: "functionName1".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                        ("initial".to_string(), FnConfKind::Const( FnConfig { name: "1234567".to_string(), type_: FnConfPointType::Int, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                        ("input".to_string(), FnConfKind::Fn( FnConfig { name: "functionName2".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                            ("input1".to_string(), FnConfKind::Const( FnConfig { name: "someValue".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                            ("input2".to_string(), FnConfKind::Point( FnConfig { name: "/path/Point.Name/".to_string(), type_: FnConfPointType::Int, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                            ("input3".to_string(), FnConfKind::Fn( FnConfig { name: "functionName3".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                ("input".to_string(), FnConfKind::Point( FnConfig { name: "/path/Point.Name/".to_string(), type_: FnConfPointType::Bool, inputs: IndexMap::new(), options: FnConfOptions::default(), } )),
                            ]), options: FnConfOptions::default(), } )),
                        ]), options: FnConfOptions::default() } )),
                    ]), options: FnConfOptions::default(), } )),
                ]), options: FnConfOptions::default(), } )
            ),
        ];
        for (value, target) in test_data {
            debug!("test value: {:?}", value);
            let conf: serde_yaml::Value = serde_yaml::from_str(value).unwrap();
            debug!("value: {:?}   |   conf: {:?}   |   target: {:?}", "_", conf, target);
            let mut vars = vec![];
            let fn_config = FnConfig::from_yaml(self_id, &self_name, &conf, &mut vars);
            debug!("\tfnConfig: {:?}", fn_config);
            assert_eq!(fn_config, target);
        }
    }
}
