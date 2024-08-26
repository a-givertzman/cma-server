#[cfg(test)]
mod fn_to_bool {
    use log::{debug, info};
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::{point::point::ToPoint, types::fn_in_out_ref::FnInOutRef}, 
        services::task::nested_function::{fn_::FnOut, fn_input::FnInput, fn_to_bool::{self, FnToBool}, reset_counter::AtomicReset}
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
        fn_to_bool::COUNT.reset(0);
        Rc::new(RefCell::new(Box::new(
            FnInput::new("test", 0, &mut conf)
        )))
    }
    ///
    /// Testing Task Add Bool's
    #[test]
    fn test_bool() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_bool");
        let input = init_each("false", FnConfPointType::Bool);
        let mut fn_to_bool = FnToBool::new(
            "test",
            input.clone(),
        );
        let test_data = vec![
            (00, false, false),
            (02, true, true),
        ];
        for (step , value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            let state = fn_to_bool.out().unwrap();
            debug!("{}   |   value: {:?}   |   state: {:?}", step, value, state);
            assert!(state.as_bool().value.0 == target, "step {} \n result: {:?} \ntarget: {}", step, state.as_bool().value.0, target);
        }
    }
    ///
    /// Testing Task Add Int's
    #[test]
    fn test_int() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_int");
        let input = init_each("0", FnConfPointType::Int);
        let mut fn_to_bool = FnToBool::new(
            "test",
            input.clone(),
        );
        let test_data = vec![
            (00, 1, true),
            (01, 2, true),
            (02, 5, true),
            (03, -1, false),
            (04, -5, false),
            (05, 0, false),
            (06, i64::MIN, false),
            (07, i64::MAX, true),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            let state = fn_to_bool.out().unwrap();
            debug!("{}   |   value: {:?}   |   state: {:?}", step, value, state);
            assert!(state.as_bool().value.0 == target, "step {} \n result: {:?} \ntarget: {}", step, state.as_bool().value.0, target);
        }
    }
    ///
    /// Testing ToBool Real's
    #[test]
    fn real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("fn_to_bool_real");
        let input = init_each("0.0", FnConfPointType::Real);
        let mut fn_to_bool = FnToBool::new(
            "test",
            input.clone(),
        );
        let test_data = vec![
            (01, 0.1, true),
            (03, 0.5, true),
            (04, 1.0, true),
            (05, 0.0, false),
            (06, -0.1, false),
            (07, -0.5, false),
            (08, f32::MIN, false),
            (09, f32::MAX, true),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            let state = fn_to_bool.out().unwrap();
            debug!("{}   |   value: {:?}   |   state: {:?}", step, value, state);
            assert!(state.as_bool().value.0 == target, "step {} \n result: {:?} \ntarget: {}", step, state.as_bool().value.0, target);
        }
    }
    ///
    /// Testing ToBool Double's
    #[test]
    fn double() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("fn_to_bool_double");
        let input = init_each("0.0", FnConfPointType::Double);
        let mut fn_to_bool = FnToBool::new(
            "test",
            input.clone(),
        );
        let test_data = vec![
            (01, 0.1, true),
            (03, 0.5, true),
            (04, 1.0, true),
            (05, 0.0, false),
            (06, -0.1, false),
            (07, -0.5, false),
            (08, f32::MIN, false),
            (09, f32::MAX, true),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            let state = fn_to_bool.out().unwrap();
            debug!("{}   |   value: {:?}   |   state: {:?}", step, value, state);
            assert!(state.as_bool().value.0 == target, "step {} \n result: {:?} \ntarget: {}", step, state.as_bool().value.0, target);
        }
    }
}
