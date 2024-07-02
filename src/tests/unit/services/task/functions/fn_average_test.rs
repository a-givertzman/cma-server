#[cfg(test)]
mod fn_average {
    use log::{debug, info};
    use testing::entities::test_value::Value;
    use std::{cell::RefCell, rc::Rc, sync::Once};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::{
            aprox_eq::aprox_eq::AproxEq, point::point_type::ToPoint, types::fn_in_out_ref::FnInOutRef
        },
        services::task::nested_function::{
            fn_::FnOut, fn_average::FnAverage, fn_input::FnInput
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
        let enable = init_each(&self_id, Value::Bool(false));
        let input = init_each(&self_id, Value::Bool(false));
        let mut fn_average = FnAverage::new(
            self_id,
            Some(enable.clone()),
            input.clone(),
        );
        let test_data = vec![
            (00,    true,  false,     0.000),
            (01,    true,  false,     0.000),
            (02,    true,   true,      0.333),
            (03,    true,  false,     0.250),
            (04,    true,  false,     0.200),
            (05,    true,   true,      0.333),
            (06,    true,  false,     0.285),
            (07,    true,   true,      0.375),
            (08,    true,  false,     0.333),
            (09,    true,  false,     0.300),
            (10,    true,   true,      0.363),
            (11,    true,   true,      0.416),
            (12,    true,   true,      0.461),
            (13,    true,  false,     0.428),
            (14,    true,  false,     0.400),
        ];
        for (step, en, value, target) in test_data {
            let en = en.to_point(0, "enable");
            let point = value.to_point(0, "input");
            enable.borrow_mut().add(en);
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let result = fn_average.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_double().value.trunc_eq(target, 3), "\nresult: {:?}\ntarget: {:?}", result, target);
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
        let enable = init_each(&self_id, Value::Bool(false));
        let input = init_each(&self_id, Value::Int(0));
        let mut fn_average = FnAverage::new(
            self_id,
            Some(enable.clone()),
            input.clone(),
        );
        let test_data = vec![
            (00,    true,  0,     0.0),
            (01,    true,  0,     0.0),
            (02,    true,  3,     1.0),
            (03,    true,  0,     0.75),
            (04,    true,  0,     0.6),
            (05,    true,  1,     0.666666666666667),
            (06,    true,  0,     0.571428571428571),
            (07,    true,  7,     1.375),
            (08,    true,  0,     1.22222222222222),
            (09,    true,  0,     1.1),
            (10,    true,  2,     1.18181818181818),
            (11,    true,  8,     1.75),
            (12,    true,  1,     1.69230769230769),
            (13,    true,  0,     1.57142857142857),
            (14,    true,  0,     1.46666666666667),
        ];
        for (step, en, value, target) in test_data {
            let en = en.to_point(0, "enable");
            let point = value.to_point(0, "input");
            enable.borrow_mut().add(en);
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let result = fn_average.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_double().value.aprox_eq(target, 3), "\nresult: {:?}\ntarget: {:?}", result, target);
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
        let enable = init_each(&self_id, Value::Bool(false));
        let input = init_each(&self_id, Value::Real(0.0));
        let mut fn_average = FnAverage::new(
            self_id,
            Some(enable.clone()),
            input.clone(),
        );
        let test_data = vec![
            (00,    true,  0.0,     0.0),
            (01,    true,  0.0,     0.0),
            (02,    true,  3.3,     1.09999),
            (03,    true,  0.1,     0.84999),
            (04,    true,  0.0,     0.67999),
            (05,    true,  1.6,     0.83333),
            (06,    true,  0.0,     0.71428),
            (07,    true,  7.2,     1.52499),
            (08,    true,  0.0,     1.35555),
            (09,    true,  0.3,     1.24999),
            (10,    true,  2.2,     1.33636),
            (11,    true,  8.1,     1.9),
            (12,    true,  1.9,     1.9),
            (13,    true,  0.1,     1.77142),
            (14,    true,  0.0,     1.65333),
        ];
        for (step, en, value, target) in test_data {
            let en = en.to_point(0, "enable");
            let point = value.to_point(0, "input");
            enable.borrow_mut().add(en);
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let result = fn_average.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_double().value.aprox_eq(target, 3), "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
    ///
    /// Real points on input, enable - is variable during the test
    #[test]
    fn test_real_enable() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "test_real_enable";
        info!("{}", self_id);
        let enable = init_each(&self_id, Value::Bool(false));
        let input = init_each(&self_id, Value::Real(0.0));
        let mut fn_average = FnAverage::new(
            self_id,
            Some(enable.clone()),
            input.clone(),
        );
        let test_data = vec![
            (00,    false,  0.0,     0.0),
            (01,    false,  0.0,     0.0),
            (02,    false,  3.3,     0.0),
            (03,    true,  0.1,     0.1),
            (04,    true,  0.0,     0.05),
            (05,    true,  1.6,     0.566666666666667),
            (06,    true,  0.0,     0.425),
            (07,    true,  7.2,     1.77999),
            (08,    true,  0.0,     1.48333333333333),
            (09,    true,  0.3,     1.31428571428571),
            (10,    true,  2.2,     1.424999),
            (11,    false,  8.1,     0.0),
            (12,    false,  1.9,     0.0),
            (13,    false,  0.1,     0.0),
            (14,    false,  0.0,     0.0),
            (15,    true,  0.1,     0.1),
            (16,    true,  0.0,     0.05),
            (17,    true,  1.6,     0.566666666666667),
            (18,    true,  0.0,     0.425),
            (19,    true,  7.2,     1.77999),
            (20,    true,  0.0,     1.48333333333333),
            (21,    true,  0.3,     1.31428571428571),
            (22,    true,  2.2,     1.424999),
            (23,    false,  0.0,     0.0),
            (24,    false,  0.0,     0.0),
        ];
        for (step, en, value, target) in test_data {
            let en = en.to_point(0, "enable");
            let point = value.to_point(0, "input");
            enable.borrow_mut().add(en);
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let result = fn_average.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_double().value.aprox_eq(target, 3), "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
    ///
    /// Double points on input, enable - is variable during the test
    #[test]
    fn test_double_enable() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        let self_id = "test_double_enable";
        info!("{}", self_id);
        let enable = init_each(&self_id, Value::Bool(false));
        let input = init_each(&self_id, Value::Double(0.0));
        let mut fn_average = FnAverage::new(
            self_id,
            Some(enable.clone()),
            input.clone(),
        );
        let test_data = vec![
            (00,    false,  0.0,     0.0),
            (01,    false,  0.0,     0.0),
            (02,    false,  3.3,     0.0),
            (03,    true,  0.1,     0.1),
            (04,    true,  0.0,     0.05),
            (05,    true,  1.6,     0.566666666666667),
            (06,    true,  0.0,     0.425),
            (07,    true,  7.2,     1.78),
            (08,    true,  0.0,     1.48333333333333),
            (09,    true,  0.3,     1.31428571428571),
            (10,    true,  2.2,     1.425),
            (11,    false,  8.1,     0.0),
            (12,    false,  1.9,     0.0),
            (13,    false,  0.1,     0.0),
            (14,    false,  0.0,     0.0),
            (15,    true,  0.1,     0.1),
            (16,    true,  0.0,     0.05),
            (17,    true,  1.6,     0.566666666666667),
            (18,    true,  0.0,     0.425),
            (19,    true,  7.2,     1.78),
            (20,    true,  0.0,     1.48333333333333),
            (21,    true,  0.3,     1.31428571428571),
            (22,    true,  2.2,     1.425),
            (23,    false,  0.0,     0.0),
            (24,    false,  0.0,     0.0),
        ];
        for (step, en, value, target) in test_data {
            let en = en.to_point(0, "enable");
            let point = value.to_point(0, "input");
            enable.borrow_mut().add(en);
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let result = fn_average.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_double().value.aprox_eq(target, 3), "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
}
