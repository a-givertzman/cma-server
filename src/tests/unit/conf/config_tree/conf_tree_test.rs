#[cfg(test)]
mod config_tree {
    use log::{debug, info};
    use std::sync::Once;
    use indexmap::IndexMap;
    use testing::entities::test_value::Value;
    use debugging ::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::conf::conf_tree::ConfTree;
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
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Node {
        Map(IndexMap<String, Node>),
        End(ConfTree),
    }
    impl Node {
        fn as_map(&self) ->&IndexMap<String, Node> {
            match self {
                Node::Map(map) => map,
                Node::End(_) => panic!("is not a map"),
            }
        }
    }
    ///
    ///
    #[test]
    fn valid() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test_config_tree_valid");
        // let (initial, switches) = init_each();
        let test_data: Vec<(&str, Node)> = vec![
            // (
            //     r#"let newVar:
            //         input const '13.55'
            //     "#,
            //     None
            // ),
            // (
            //     r#"let newVar:
            //         input fn Count:
            //             inputConst1 const '13.3'
            //             inputConst2 const '13.7'
            //     "#,
            //     None
            // ),
            // (
            //     r#"let newVar:
            //         input1 fn Count:
            //             inputConst1 const '11.3'
            //             inputConst2 const '12.7'"
            //         input2 fn Count:
            //             inputConst1 const '13.3'
            //             inputConst2 const '14.7'
            //     "#,
            //     None
            // ),
            (
                r#"
                    let newVar2:
                        input: const 2.2
                    let newVar3:
                        input: const 3.3
                    let newVar1:
                        input: const 1.1
                "#,
                Node::Map(IndexMap::from([
                    (format!("let newVar2"), Node::Map(IndexMap::from([
                        (format!("input"), Node::End(ConfTree { key: format!("input"), conf: serde_yaml::from_str("const 2.2").unwrap() })),
                    ]))),
                    (format!("let newVar3"), Node::Map(IndexMap::from([
                        (format!("input"), Node::End(ConfTree { key: format!("input"), conf: serde_yaml::from_str("const 3.3").unwrap() })),
                    ]))),
                    (format!("let newVar1"), Node::Map(IndexMap::from([
                        (format!("input"), Node::End(ConfTree { key: format!("input"), conf: serde_yaml::from_str("const 1.1").unwrap() })),
                    ]))),
                ]))
            ),
            (
                r#"
                    let newVar1:
                        input1: const 177.3
                        input2: point '/Path/Point.Name/'
                        input3:
                            fn Count:
                                inputConst1: const '13.5'
                                inputConst2: newVar1
                "#,
                Node::Map(IndexMap::from([
                    (format!("let newVar1"), Node::Map(IndexMap::from([
                        (format!("input2"), Node::End(ConfTree { key: format!("input2"), conf: serde_yaml::from_str("point '/Path/Point.Name/'").unwrap() })),
                        (format!("input3"), Node::Map(IndexMap::from([
                            (format!("fn Count"), Node::Map(IndexMap::from([
                                (format!("inputConst1"), Node::End(ConfTree { key: format!("inputConst1"), conf: serde_yaml::from_str("const '13.5'").unwrap() })),
                                (format!("inputConst2"), Node::End(ConfTree { key: format!("inputConst2"), conf: serde_yaml::from_str("newVar1").unwrap() }))
                            ])))
                        ]))),
                        (format!("input1"), Node::End(ConfTree { key: format!("input1"), conf: serde_yaml::from_str("const 177.3").unwrap() }))
                    ]))),
                ]))
            ),
            // (
            //     r#"
            //         fn SqlMetric:
            //             initial: const 0
            //             sql: "UPDATE {table} SET kind = '{input1}' WHERE id = '{input2}';"
            //             inputs:
            //                 input1:
            //                     let VarName2:
            //                         input:
            //                             fn functionName:
            //                                 initial: VarName2
            //                                 input:
            //                                     fn functionName:
            //                                         input1: const someValue
            //                                         input2: point '/path/Point.Name/'
            //                                         input:
            //                                             fn functionName:
            //                                                 input: point '/path/Point.Name/'
            //     "#,
            //     None
            // ),
            (
                r#"
                    serviceCMA:
                        nodeType: API Client
                        address: 127.0.0.1:8899
                        cycle: 1000
                    serviceAPI:
                        nodeType: API Client
                        address: 127.0.0.1:8899
                        cycle: 2000
                    serviceTask:
                        cycle: 200
                "#,
                Node::Map(IndexMap::from([
                    (format!("serviceCMA"), Node::Map(IndexMap::from([
                        (format!("nodeType"), Node::End(ConfTree { key: format!("nodeType"), conf: serde_yaml::from_str("API Client").unwrap() })),
                        (format!("address"), Node::End(ConfTree { key: format!("address"), conf: serde_yaml::from_str("127.0.0.1:8899").unwrap() })),
                        (format!("cycle"), Node::End(ConfTree { key: format!("cycle"), conf: serde_yaml::from_str("1000").unwrap() })),
                    ]))),
                    (format!("serviceAPI"), Node::Map(IndexMap::from([
                        (format!("nodeType"), Node::End(ConfTree { key: format!("nodeType"), conf: serde_yaml::from_str("API Client").unwrap() })),
                        (format!("address"), Node::End(ConfTree { key: format!("address"), conf: serde_yaml::from_str("127.0.0.1:8899").unwrap() })),
                        (format!("cycle"), Node::End(ConfTree { key: format!("cycle"), conf: serde_yaml::from_str("2000").unwrap() })),
                    ]))),
                    (format!("serviceTask"), Node::Map(IndexMap::from([
                        (format!("cycle"), Node::End(ConfTree { key: format!("cycle"), conf: serde_yaml::from_str("200").unwrap() }))
                    ]))),
                ]))
            ),
        ];
        for (value, target) in test_data {
            // debug!("test value: {:?}", value);
            let conf: serde_yaml::Value = serde_yaml::from_str(value).unwrap();
            debug!("test conf: {:?}", conf);
            // let conf = test_data.get("/").unwrap();
            let conf_tree = ConfTree::new_root(conf);
            debug!("confTree: {:?}", conf_tree);
            let res = inputs(&conf_tree);
            debug!("result: {:?}", res);
            println!();
            assert_eq!(res, target);
            let mut target = target.as_map().iter();
            for (_name, node) in res.as_map() {
                let (_, target_node) = target.next().unwrap();
                assert_eq!(node, target_node);
            }
        }
    }

    fn inputs(conf_tree: &ConfTree) -> Node {
        match conf_tree.sub_nodes() {
            Some(nodes) => {
                let mut res: IndexMap<String, Node> = IndexMap::new();
                for node in nodes {
                    debug!("key: {:?}\t|\tnode: {:?}", &node.key, &node.conf);
                    let sub_res = inputs(&node);
                    res.insert(node.key.clone(), sub_res);
                }
                return Node::Map(res)
            }
            None => {
                return Node::End(conf_tree.clone());
            }
        };
    }
    ///
    /// 
    #[test]
    fn as_type() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test_config_tree_valid");
        // let (initial, switches) = init_each();
        let test_data: Vec<(&str, IndexMap<&str, Value>)> = vec![
            (
                r#"
                    boolTrue: true
                    boolFalse: false
                    int1: 177
                    int2: -177
                    real1: 177.3
                    real2: -177.3
                    double1: 177.3
                    double2: -177.3
                    string1: /Path/Point.Name/
                    string2: '/Path/Point.Name/'
                    string3: "/Path/Point.Name/"
                "#,
                IndexMap::from([
                    ("boolTrue", Value::Bool(true)),
                    ("boolFalse", Value::Bool(false)),
                    ("int1", Value::Int(177)),
                    ("int2", Value::Int(-177)),
                    ("real1", Value::Real(177.3)),
                    ("real2", Value::Real(-177.3)),
                    ("double1", Value::Double(177.3)),
                    ("double2", Value::Double(-177.3)),
                    ("string1", Value::String("/Path/Point.Name/".to_string())),
                    ("string2", Value::String("/Path/Point.Name/".to_string())),
                    ("string3", Value::String("/Path/Point.Name/".to_string())),
                ])
            ),

        ];
        for (value, targets) in test_data {
            // debug!("test value: {:?}", value);
            let conf: serde_yaml::Value = serde_yaml::from_str(value).unwrap();
            debug!("test conf: {:?}", conf);
            let conf_tree = ConfTree::new_root(conf);
            for (key, target) in targets {
                match target {
                    Value::Bool(target_value) => assert_eq!(conf_tree.as_bool(key).unwrap(), target_value),
                    Value::Int(target_value) => assert_eq!(conf_tree.as_i64(key).unwrap(), target_value),
                    Value::Real(target_value) => assert_eq!(conf_tree.as_f32(key).unwrap(), target_value),
                    Value::Double(target_value) => assert_eq!(conf_tree.as_f64(key).unwrap(), target_value),
                    Value::String(target_value) => assert_eq!(conf_tree.as_str(key).unwrap(), target_value),
                }
            }
        }
    }
}
