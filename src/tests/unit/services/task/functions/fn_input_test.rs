#[cfg(test)]

mod fn_input {
    use log::{debug, info};
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig},
        core_::{point::point_type::ToPoint, types::fn_in_out_ref::FnInOutRef},
        services::task::nested_function::fn_input::FnInput,
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
    ///
    ///
    #[test]
    fn int() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_int");
        let input = init_each("0", FnConfPointType::Int);
        let test_data = vec![
            0,
            1,
            2,
            3,
            4,
            5,
            6,
            5,
            4,
            3,
            2,
            1,
            0,
            0,
        ];
        for value in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let state = input.borrow_mut().out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_int().value, value);
        }
    }
    ///
    ///
    #[test]
    fn bool() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_bool");
        let input = init_each("false", FnConfPointType::Bool);
        let test_data = vec![
            false,
            false,
            false,
            true,
            false,
            true,
            true,
            false,
            false,
            false,
            true,
            true,
            false,
            false,
            false,
            false,
        ];
        for value in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let state = input.borrow_mut().out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_bool().value.0, value);
        }
    }
    ///
    ///
    #[test]
    fn real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_real");
        let input = init_each("0.0", FnConfPointType::Real);
        let test_data = vec![
            0.0f32,
            1.0f32,
            2.0f32,
            4.0f32,
            3.0f32,
            5.0f32,
            6.0f32,
            3.0f32,
            2.0f32,
            3.0f32,
            4.0f32,
            4.0f32,
            3.0f32,
            2.0f32,
            1.0f32,
            0.0f32,
        ];
        for value in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let state = input.borrow_mut().out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_real().value, value);
        }
    }
    ///
    ///
    #[test]
    fn double() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_real");
        let input = init_each("0.0", FnConfPointType::Double);
        let test_data = vec![
            0.0f64,
            1.0f64,
            2.0f64,
            4.0f64,
            3.0f64,
            5.0f64,
            6.0f64,
            3.0f64,
            2.0f64,
            3.0f64,
            4.0f64,
            4.0f64,
            3.0f64,
            2.0f64,
            1.0f64,
            0.0f64,
        ];
        for value in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let state = input.borrow_mut().out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_double().value, value);
        }
    }
    ///
    ///
    #[test]
    fn test_string() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_string");
        let input = init_each("0", FnConfPointType::String);
        let test_data = vec![
            "0",
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "5",
            "4",
            "3",
            "2",
            "1",
            "0",
            "0",
        ];
        for value in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(point);
            // debug!("input: {:?}", &input);
            let state = input.borrow_mut().out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_string().value, value);
        }
    }
}