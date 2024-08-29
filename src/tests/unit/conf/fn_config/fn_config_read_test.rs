#[cfg(test)]

mod tests {
    use log::trace;
    use sal_sync::services::entity::name::Name;
    use std::{sync::Once, env};
    use indexmap::IndexMap;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_kind::FnConfKind, fn_conf_options::FnConfOptions, fn_config::FnConfig};
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
    fn test_fn_config_read_valid() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test FnConfig | read valid";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let target = FnConfKind::Var( FnConfig {
            name: "VarName2".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                ("input".to_string(), FnConfKind::Fn( FnConfig {
                    name: "functionName".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                        ("initial".to_string(), FnConfKind::Var( FnConfig {
                            name: "VarName2".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(),
                            options: FnConfOptions::default(),
                        } )),
                        ("input".to_string(), FnConfKind::Fn( FnConfig {
                            name: "functionName".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                ("input".to_string(), FnConfKind::Fn( FnConfig {
                                    name: "functionName".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                                        ("input".to_string(), FnConfKind::Point( FnConfig {
                                            name: "/path/Point.Name/".to_string(), type_: FnConfPointType::Bool, inputs: IndexMap::from([]),
                                            options: FnConfOptions::default(),
                                        } )),
                                    ]),
                                    options: FnConfOptions::default(),
                                } )),
                                ("input2".to_string(), FnConfKind::Point( FnConfig {
                                    name: "/path/Point.Name/".to_string(), type_: FnConfPointType::Real, inputs: IndexMap::from([]),
                                    options: FnConfOptions::default(),
                                } )),
                                ("input1".to_string(), FnConfKind::Const( FnConfig {
                                    name: "someValue".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([]),
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
        } );

        // let (initial, switches) = init_each();
        trace!("dir: {:?}", env::current_dir());
        let path= "./src/tests/unit/conf/fn_config/fn_config_test.yaml";
        let fn_config = FnConfig::read(self_id, &self_name, path);
        trace!("fnConfig: {:?}", fn_config);
        assert_eq!(fn_config, target);
    }
}
