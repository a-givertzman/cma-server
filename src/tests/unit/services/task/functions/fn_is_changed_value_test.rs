#[cfg(test)]
mod fn_is_changed_value {
    use log::{debug, info};
    use sal_sync::services::entity::point::point::ToPoint;
    use testing::entities::test_value::Value;
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::types::fn_in_out_ref::FnInOutRef, 
        services::task::nested_function::{fn_::FnOut, fn_input::FnInput, fn_is_changed_value::FnIsChangedValue}
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
    fn init_each(default: &str, name: impl Into<String>, type_: FnConfPointType) -> FnInOutRef {
        let mut conf = FnConfig { name: name.into(), type_, options: FnConfOptions {default: Some(default.into()), ..Default::default()}, ..Default::default()};
        Rc::new(RefCell::new(Box::new(
            FnInput::new("test", 0, &mut conf)
        )))
    }
    ///
    /// Testing accumulation of the Bool's
    #[test]
    fn is_changed_bool() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "is_changed_bool";
        info!("{}", self_id);
        let input1 = init_each("false", format!("/{}/Bool", self_id), FnConfPointType::Bool);
        let input2 = init_each("0", format!("/{}/Int", self_id), FnConfPointType::Int);
        let input3 = init_each("0.0", format!("/{}/Real", self_id), FnConfPointType::Real);
        let input4 = init_each("0.0", format!("/{}/Double", self_id), FnConfPointType::Double);
        let input5 = init_each("test", format!("/{}/String", self_id), FnConfPointType::String);
        let mut fn_is_changed = FnIsChangedValue::new(
            "test",
            vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
                input5.clone(),
            ]
        );
        let test_data = vec![
            (00, format!("/{}/Bool", self_id),      Value::Bool(false),     1),
            (01, format!("/{}/Bool", self_id),      Value::Bool(false),     0),
            (02, format!("/{}/Bool", self_id),      Value::Bool(true),      1),
            (03, format!("/{}/Bool", self_id),      Value::Bool(true),      0),
            (04, format!("/{}/Int", self_id),       Value::Int(0),          0),
            (05, format!("/{}/Int", self_id),       Value::Int(0),          0),
            (06, format!("/{}/Real", self_id),      Value::Real(0.0),       0),
            (07, format!("/{}/Int", self_id),       Value::Int(0),          0),
            (08, format!("/{}/Real", self_id),      Value::Real(0.1),       1),
            (09, format!("/{}/Double", self_id),    Value::Double(0.1),     1),
            (10, format!("/{}/Bool", self_id),      Value::Bool(true),      0),
            (11, format!("/{}/Double", self_id),    Value::Double(0.1),     0),
            (12, format!("/{}/Real", self_id),      Value::Real(0.1),       0),
            (13, format!("/{}/Bool", self_id),      Value::Bool(true),      0),
            (13, format!("/{}/String", self_id),    Value::String("..".into()),      1),
            (14, format!("/{}/Bool", self_id),      Value::Bool(false),     1),
            (15, format!("/{}/Bool", self_id),      Value::Bool(false),     0),
            (16, format!("/{}/Double", self_id),    Value::Double(0.0),     1),
            (17, format!("/{}/Real", self_id),      Value::Real(0.1),       0),
            (18, format!("/{}/Double", self_id),    Value::Double(0.0),     0),
            (19, format!("/{}/Bool", self_id),      Value::Bool(false),     0),
        ];
        for (step, name, value, target) in test_data {
            match &value {
                Value::Bool(value) => {
                    input1.borrow_mut().add(&value.to_point(0, &name))
                }
                Value::Int(value) => {
                    input2.borrow_mut().add(&value.to_point(0, &name))
                }
                Value::Real(value) => {
                    input3.borrow_mut().add(&value.to_point(0, &name))
                }
                Value::Double(value) => {
                    input4.borrow_mut().add(&value.to_point(0, &name))
                }
                Value::String(value) => {
                    input5.borrow_mut().add(&value.to_point(0, &name))
                }
            };
            // debug!("input: {:?}", &input);
            let state = fn_is_changed.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {}   |   value: {:?}   |   state: {:?}", step, value, state);
            assert!(state.as_bool().value.0 == (target > 0), "step {} \n result: {:?} \ntarget: {}", step, state.as_bool().value.0, target > 0);
        }
    }
}
