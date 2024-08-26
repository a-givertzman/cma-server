#[cfg(test)]
mod fn_rising_edge {
    use log::{debug, info};
    use testing::entities::test_value::Value;
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::{
            point::point::ToPoint, types::fn_in_out_ref::FnInOutRef,
        },
        services::task::nested_function::{
            edge_detection::fn_rising_edge::FnRisingEdge, fn_::FnOut, fn_input::FnInput
        }
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
    fn init_each(parent: &str, initial: Value) -> FnInOutRef {
        let mut conf = FnConfig {
            name: "test".to_owned(),
            type_: match initial {
                Value::Bool(_) => FnConfPointType::Bool,
                Value::Int(_) => FnConfPointType::Int,
                Value::Real(_) => FnConfPointType::Real,
                Value::Double(_) => FnConfPointType::Double,
                Value::String(_) => FnConfPointType::String,
            },
            options: FnConfOptions {default: Some(match initial {
                Value::Bool(v) => v.to_string(),
                Value::Int(v) => v.to_string(),
                Value::Real(v) => v.to_string(),
                Value::Double(v) => v.to_string(),
                Value::String(v) => v.to_string(),
            }),
                ..Default::default()}, ..Default::default()
        };        
        Rc::new(RefCell::new(Box::new(
            FnInput::new(parent, 0, &mut conf)
        )))
    }
    ///
    ///
    #[test]
    fn test_bool() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "test_bool";
        info!("{}", self_id);
        let input = init_each(&self_id, Value::Bool(false));
        let mut fn_rising_edge = FnRisingEdge::new(
            self_id,
            input.clone(),
        );
        let test_data = vec![
            (00,    false,     false),
            (01,    false,     false),
            (02,    true,      true),
            (03,    false,     false),
            (04,    false,     false),
            (05,    true,      true),
            (06,    false,     false),
            (07,    true,      true),
            (08,    false,     false),
            (09,    false,     false),
            (10,    true,      true),
            (11,    true,      false),
            (12,    true,      false),
            (13,    false,     false),
            (14,    false,     false),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_rising_edge.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
        assert!(result.as_bool().value.0 == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
    ///
    ///
    #[test]
    fn test_int() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "test_int";
        info!("{}", self_id);
        let input = init_each(&self_id, Value::Int(0));
        let mut fn_rising_edge = FnRisingEdge::new(
            self_id,
            input.clone(),
        );
        let test_data = vec![
            (00,    0,      false),
            (01,    0,      false),
            (02,    1,      true),
            (03,    0,      false),
            (04,    0,      false),
            (05,    3,      true),
            (06,    0,      false),
            (07,    2,      true),
            (08,    0,      false),
            (09,    -1,     false),
            (10,    3,      true),
            (11,    77,     false),
            (12,    65,     false),
            (13,    0,      false),
            (14,    -10,    false),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_rising_edge.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
        assert!(result.as_bool().value.0 == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
    ///
    ///
    #[test]
    fn test_real() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "test_real";
        info!("{}", self_id);
        let input = init_each(&self_id, Value::Real(0.0));
        let mut fn_rising_edge = FnRisingEdge::new(
            self_id,
            input.clone(),
        );
        let test_data = vec![
            (00,    0.0,      false),
            (01,    0.0,      false),
            (02,    0.1,      true),
            (03,    0.0,      false),
            (04,    0.0,      false),
            (05,    3.0,      true),
            (06,    0.0,      false),
            (07,    2.0,      true),
            (08,    0.0,      false),
            (09,    -1.0,     false),
            (10,    3.0,      true),
            (11,    77.0,     false),
            (12,    65.0,     false),
            (13,    0.0,      false),
            (14,    -10.0,    false),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_rising_edge.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
        assert!(result.as_bool().value.0 == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
    ///
    ///
    #[test]
    fn test_double() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "test_real";
        info!("{}", self_id);
        let input = init_each(&self_id, Value::Double(0.0));
        let mut fn_rising_edge = FnRisingEdge::new(
            self_id,
            input.clone(),
        );
        let test_data = vec![
            (00,    0.0,      false),
            (01,    0.0,      false),
            (02,    0.1,      true),
            (03,    0.0,      false),
            (04,    0.0,      false),
            (05,    3.0,      true),
            (06,    0.0,      false),
            (07,    2.0,      true),
            (08,    0.0,      false),
            (09,    -1.0,     false),
            (10,    3.0,      true),
            (11,    77.0,     false),
            (12,    65.0,     false),
            (13,    0.0,      false),
            (14,    -10.0,    false),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_rising_edge.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
        assert!(result.as_bool().value.0 == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }     
}
