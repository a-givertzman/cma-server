#[cfg(test)]

mod fn_point_id {
    use log::debug;
    use sal_sync::services::{entity::{name::Name, point::{point::ToPoint, point_config::PointConfig}}, task::functions::conf::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions}};
    use testing::entities::test_value::Value;
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::fn_config::FnConfig,
        core_::types::fn_in_out_ref::FnInOutRef,
        services::task::nested_function::{fn_::FnOut, fn_input::FnInput, fn_point_id::FnPointId},
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
    fn init_each(default: &str, type_: FnConfPointType) -> FnInOutRef {
        let mut conf = FnConfig { name: "test".to_owned(), type_, options: FnConfOptions {default: Some(default.into()), ..Default::default()}, ..Default::default()};
        Rc::new(RefCell::new(Box::new(
            FnInput::new("test", 0, &mut conf)
        )))
    }
    const POINTS: &[(usize, &str)] = &[
                    (0, r#"PointName0:
                        type: bool      # Bool / Int / Real / String / Json
                        comment: Test Point Bool"#),
                    (1, r#"PointName1:
                        type: bool      # Bool / Int / Real / String / Json
                        comment: Test Point Bool"#),
                    (2, r#"PointName2:
                        type: bool      # Bool / Int / Real / String / Json
                        comment: Test Point Bool"#),
                    (3, r#"PointName3:
                        type: bool      # Bool / Int / Real / String / Json
                        comment: Test Point Bool"#),
                    (4, r#"PointName4:
                        type: bool      # Bool / Int / Real / String / Json
                        comment: Test Point Bool"#),
    ];
    ///
    ///
    #[test]
    fn basic() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "fn_point_id_test";
        println!("{}", self_id);
        let input = init_each("0", FnConfPointType::Any);
        let points = POINTS.into_iter().map(|(id, conf)| {
            let mut point = PointConfig::from_yaml(&Name::new(self_id, ""), &serde_yaml::from_str(conf).unwrap());
            point.id = *id;
            point
        }).collect();
        println!("{} | Configured points: {:#?}", self_id, points);
        let mut fn_point_id = FnPointId::new(self_id, input.clone(), points);
        let test_data = vec![
            (Value::Bool(false),    Name::new(self_id, "PointName0").join(), 0),
            (Value::Bool(true),     Name::new(self_id, "PointName0").join(), 0),
            (Value::Int(0),         Name::new(self_id, "PointName1").join(), 1),
            (Value::Int(1),         Name::new(self_id, "PointName2").join(), 2),
            (Value::Int(2),         Name::new(self_id, "PointName2").join(), 2),
            (Value::Int(3),         Name::new(self_id, "PointName3").join(), 3),
            (Value::Int(3),         Name::new(self_id, "PointName3").join(), 3),
            (Value::Int(-1),        Name::new(self_id, "PointName2").join(), 2),
            (Value::Int(-2),        Name::new(self_id, "PointName2").join(), 2),
            (Value::Int(-3),        Name::new(self_id, "PointName3").join(), 3),
            (Value::Int(-4),        Name::new(self_id, "PointName4").join(), 4),
            (Value::Real(5.0),      Name::new(self_id, "PointName0").join(), 0),
            (Value::Real(6.0),      Name::new(self_id, "PointName1").join(), 1),
            (Value::Real(5.0),      Name::new(self_id, "PointName2").join(), 2),
            (Value::Real(4.0),      Name::new(self_id, "PointName3").join(), 3),
            (Value::Real(-3.0),     Name::new(self_id, "PointName4").join(), 4),
            (Value::Double(2.1),    Name::new(self_id, "PointName0").join(), 0),
            (Value::Double(1.1),    Name::new(self_id, "PointName1").join(), 1),
            (Value::Double(0.1),    Name::new(self_id, "PointName2").join(), 2),
            (Value::Double(-0.1),   Name::new(self_id, "PointName3").join(), 3),
            (Value::Double(-0.1),   Name::new(self_id, "PointName4").join(), 4),
            (Value::String("2.1".to_owned()),    Name::new(self_id, "PointName0").join(), 0),
            (Value::String("1.1".to_owned()),    Name::new(self_id, "PointName1").join(), 1),
            (Value::String("0.1".to_owned()),    Name::new(self_id, "PointName2").join(), 2),
            (Value::String("-0.1".to_owned()),   Name::new(self_id, "PointName3").join(), 3),
            (Value::String("-0.1".to_owned()),   Name::new(self_id, "PointName4").join(), 4),
        ];
        for (value, name, target_id) in test_data {
            let point = value.to_point(0, &name);
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_point_id.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_int().value, target_id);
        }
    }
}