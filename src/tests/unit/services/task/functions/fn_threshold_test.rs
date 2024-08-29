#[cfg(test)]
mod fn_threshold {
    use log::{debug, info};
    use sal_sync::services::entity::point::point::ToPoint;
    use testing::entities::test_value::Value;
    use std::{cell::RefCell, rc::Rc, sync::Once};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::types::fn_in_out_ref::FnInOutRef,
        services::task::nested_function::{
            filter::fn_threshold::FnThreshold, fn_::FnOut, fn_input::FnInput,
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
    /// Threshold Int's
    #[test]
    fn fn_threshold_int() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "fn_threshold_int";
        info!("{}", self_id);
        let threshold = init_each(&self_id, Value::Double(0.0));
        let input = init_each(&self_id, Value::Int(0));
        let mut fn_threshold = FnThreshold::new(
            self_id,
            None,
            threshold.clone(),
            None,
            input.clone(),
        );
        let test_data = vec![
            (00,    3.0,  0,     0),
            (01,    3.0,  1,     0),
            (02,    3.0,  2,     0),
            (03,    3.0,  3,     3),
            (04,    3.0,  4,     3),
            (05,    3.0,  5,     3),
            (06,    3.0,  4,     3),
            (07,    3.0,  3,     3),
            (08,    3.0,  2,     3),
            (09,    3.0,  1,     3),
            (10,    3.0,  0,     0),
            (11,    3.0,  0,     0),
            (12,    3.0,  0,     0),
            (13,    3.0,  0,     0),
            (14,    3.0,  0,     0),
        ];
        for (step, thrh, value, target) in test_data {
            let thrh = thrh.to_point(0, "threshold");
            let point = value.to_point(0, &format!("input step {}", step));
            threshold.borrow_mut().add(&thrh);
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_threshold.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_int().value == target, "step {}\nresult: {:?}\ntarget: {:?}", step, result, target);
        }
    }
    ///
    /// Threshold Reals's
    #[test]
    fn fn_threshold_real() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "fn_threshold_real";
        info!("{}", self_id);
        let threshold = init_each(&self_id, Value::Double(0.0));
        let input = init_each(&self_id, Value::Real(0.0));
        let mut fn_threshold = FnThreshold::new(
            self_id,
            None,
            threshold.clone(),
            None,
            input.clone(),
        );
        let test_data = vec![
            (00,    0.3,  0.0,     0.0),
            (01,    0.3,  0.1,     0.0),
            (02,    0.3,  0.2,     0.0),
            (03,    0.3,  0.3,     0.3),
            (04,    0.3,  0.4,     0.3),
            (05,    0.3,  0.5,     0.3),
            (06,    0.3,  0.4,     0.3),
            (07,    0.3,  0.3,     0.3),
            (08,    0.3,  0.2,     0.3),
            (09,    0.3,  0.1,     0.3),
            (10,    0.3,  0.0,     0.0),
            (11,    0.3,  0.0,     0.0),
            (12,    0.3,  0.0,     0.0),
            (13,    0.3,  0.0,     0.0),
            (14,    0.3,  0.0,     0.0),
        ];
        for (step, thrh, value, target) in test_data {
            let thrh = thrh.to_point(0, "threshold");
            let point = value.to_point(0, &format!("input step {}", step));
            threshold.borrow_mut().add(&thrh);
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_threshold.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_real().value == target, "step {}\nresult: {:?}\ntarget: {:?}", step, result, target);
        }
    }
    ///
    /// Threshold Double's
    #[test]
    fn fn_threshold_double() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "fn_threshold_double";
        info!("{}", self_id);
        let threshold = init_each(&self_id, Value::Double(0.0));
        let input = init_each(&self_id, Value::Double(0.0));
        let mut fn_threshold = FnThreshold::new(
            self_id,
            None,
            threshold.clone(),
            None,
            input.clone(),
        );
        let test_data = vec![
            (00,    0.3,  0.0,     0.0),
            (01,    0.3,  0.1,     0.0),
            (02,    0.3,  0.2,     0.0),
            (03,    0.3,  0.3,     0.3),
            (04,    0.3,  0.4,     0.3),
            (05,    0.3,  0.5,     0.3),
            (06,    0.3,  0.4,     0.3),
            (07,    0.3,  0.3,     0.3),
            (08,    0.3,  0.2,     0.3),
            (09,    0.3,  0.1,     0.3),
            (10,    0.3,  0.0,     0.0),
            (11,    0.3,  0.0,     0.0),
            (12,    0.3,  0.0,     0.0),
            (13,    0.3,  0.0,     0.0),
            (14,    0.3,  0.0,     0.0),
        ];
        for (step, thrh, value, target) in test_data {
            let thrh = thrh.to_point(0, "threshold");
            let point = value.to_point(0, &format!("input step {}", step));
            threshold.borrow_mut().add(&thrh);
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_threshold.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_double().value == target, "step {}\nresult: {:?}\ntarget: {:?}", step, result, target);
        }
    }
}
