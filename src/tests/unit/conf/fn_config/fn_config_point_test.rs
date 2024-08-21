#[cfg(test)]

mod tests {
    use log::debug;
    use std::sync::Once;
    use indexmap::IndexMap;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::conf::{
        fn_::{fn_conf_keywd::FnConfPointType, fn_conf_kind::FnConfKind, fn_conf_options::FnConfOptions, fn_config::FnConfig, fn_point_config::FnPointConfig}, point_config::{name::Name, point_config::PointConfig, point_config_history::PointConfigHistory, point_config_type::PointConfigType}
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
    fn test_fn_config_point() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test FnConfig | point";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_point1 = PointConfig {
            id: 0,
            name: format!("/{}/CraneMovement.BoomUp", self_id),
            type_: PointConfigType::Int,
            history: PointConfigHistory::None,
            alarm: None,
            address: None,
            filters: None,
            comment: Some("Some indication".to_string()),
        };
        let test_point2 = PointConfig {
            id: 0,
            name: format!("/{}/CraneMovement.BoomDown", self_id),
            type_: PointConfigType::Real,
            history: PointConfigHistory::Read,
            alarm: None,
            address: None,
            filters: None,
            comment: Some("Some indication".to_string()),
        };
        let test_data = [
            (
                vec![],
                r#"let newVar:
                    input: const '13.55'
                "#,
                FnConfKind::Var(
                    FnConfig { name: "newVar".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                        ("input".to_string(), FnConfKind::Const( FnConfig { name: "13.55".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::new(), options: FnConfOptions::default(), })),
                    ]), options: FnConfOptions::default(), }
                ),
            ),
            (
                vec![test_point1.clone(), test_point2.clone()],
                r#"
                    fn ToMultiQueue:
                        in1 point CraneMovement.BoomUp:
                            type: 'Int'
                            comment: 'Some indication'
                            input:
                                const real 0.05
                        in2 point CraneMovement.BoomDown:
                            type: 'Real'
                            history: r
                            comment: 'Some indication'
                            input:
                                const real 0.07
                "#,
                FnConfKind::Fn( FnConfig { name: "ToMultiQueue".to_string(), type_: FnConfPointType::Unknown, inputs: IndexMap::from([
                    ("in1".to_string(), FnConfKind::PointConf( FnPointConfig {
                        conf: test_point1,
                        send_to: None,
                        enable: None,
                        input: Some(Box::new(FnConfKind::Const( FnConfig { name: "0.05".to_string(), type_: FnConfPointType::Real, inputs: IndexMap::new(), options: FnConfOptions::default() } ))),
                        changes_only: None,
                    })),
                    ("in2".to_string(), FnConfKind::PointConf( FnPointConfig {
                        conf: test_point2,
                        send_to: None,
                        enable: None,
                        input: Some(Box::new(FnConfKind::Const( FnConfig { name: "0.07".to_string(), type_: FnConfPointType::Real, inputs: IndexMap::new(), options: FnConfOptions::default() } ))),
                        changes_only: None,
                    })),
                ]), options: FnConfOptions::default(), } ),
            ),
        ];
        for (points_target, value, target) in test_data {
            debug!("test value: {:?}", value);
            let conf: serde_yaml::Value = serde_yaml::from_str(value).unwrap();
            debug!("value: {:?}   |   conf: {:?}   |   target: {:?}", "_", conf, target);
            let mut vars = vec![];
            let fn_config = FnConfig::from_yaml(self_id, &self_name, &conf, &mut vars);
            let points = fn_config.points();
            debug!("\tfnConfig: {:?}", fn_config);
            debug!("\tpoints: {:?}", points);
            assert_eq!(fn_config, target);
            assert_eq!(points, points_target);
        }
    }
}
