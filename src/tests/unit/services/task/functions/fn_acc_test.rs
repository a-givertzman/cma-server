#[cfg(test)]
mod fn_acc {
    use log::{debug, info};
    use sal_sync::services::{entity::point::point::ToPoint, task::functions::conf::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions}};
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::fn_config::FnConfig, 
        core_::types::fn_in_out_ref::FnInOutRef, 
        services::task::nested_function::{fn_::FnOut, fn_acc::{self, FnAcc}, fn_input::FnInput, reset_counter::AtomicReset},
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
        fn_acc::COUNT.reset(0);
        Rc::new(RefCell::new(Box::new(
            FnInput::new("test", 0, &mut conf)
        )))
    }
    ///
    /// Testing accumulation of the BSool's
    #[test]
    fn acc_bool() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("acc_bool");
        let initial = Some(init_each("0", FnConfPointType::Int));
        let input = init_each("false", FnConfPointType::Bool);
        let mut fn_count = FnAcc::new(
            "test",
            initial,
            input.clone(),
        );
        let test_data = vec![
            (false, 0),
            (false, 0),
            (true, 1),
            (false, 1),
            (false, 1),
            (true, 2),
            (false, 2),
            (true, 3),
            (false, 3),
            (false, 3),
            (true, 4),
            (true, 5),
            (false, 5),
            (false, 5),
        ];
        for (value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_count.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_int().value, target, "\n result: {:?} \ntarget: {}", state, target);
        }
    }
    ///
    /// Testing accumulation of the Int's
    #[test]
    fn acc_int() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("acc_int");
        let initial = Some(init_each("0", FnConfPointType::Int));
        let input = init_each("0", FnConfPointType::Int);
        let mut fn_count = FnAcc::new(
            "test",
            initial,
            input.clone(),
        );
        let test_data = vec![
            (0, 0),
            (1, 1),
            (22, 23),
            (1457, 1480),
            (-10, 1470),
            (0, 1470),
            (99, 1569),
            (0, 1569),
            (0, 1569),
            (-2, 1567),
            (15, 1582),
            (0, 1582),
            (1, 1583),
            (0, 1583),
        ];
        for (value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_count.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_int().value, target, "\n   result: {:?} \ntarget: {}", state, target);
        }
    }
    ///
    /// Testing accumulation of the Int's using reset
    #[test]
    fn acc_int_reset() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("acc_int_reset");
        let initial = Some(init_each("0", FnConfPointType::Int));
        let input = init_each("0", FnConfPointType::Int);
        let mut fn_count = FnAcc::new(
            "test",
            initial,
            input.clone(),
        );
        let test_data = vec![
            (0, 0, false),
            (1, 1, false),
            (22, 23, false),
            (1457, 1480, false),
            (-10, 1470, false),
            (0, 1470, false),
            (99, 99, true),
            (0, 99, false),
            (0, 99, false),
            (-2, 97, false),
            (15, 112, false),
            (0, 112, false),
            (1, 1, true),
            (0, 1, false),
        ];
        for (value, target, reset) in test_data {
            if reset {
                fn_count.reset();
            }
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_count.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_int().value, target, "\n   result: {:?} \ntarget: {}", state, target);
        }
    }
    ///
    /// Testing accumulation of the Real's
    #[test]
    fn acc_real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("acc_real");
        let initial = Some(init_each("0.0", FnConfPointType::Real));
        let input = init_each("0.0", FnConfPointType::Real);
        let mut fn_count = FnAcc::new(
            "test",
            initial,
            input.clone(),
        );
        let test_data = vec![
            (0.0f32, 0.0),
            (1.0, 1.0),
            (22.0, 23.0),
            (1457.0, 1480.0),
            (-10.0, 1470.0),
            (0.0, 1470.0),
            (99.0, 1569.0),
            (0.0, 1569.0),
            (0.0, 1569.0),
            (-2.0, 1567.0),
            (15.0, 1582.0),
            (0.0, 1582.0),
            (1.0, 1583.0),
            (0.0, 1583.0),
        ];
        for (value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_count.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_real().value, target, "\n   result: {:?} \ntarget: {}", state, target);
        }
    }
    ///
    /// Testing accumulation of the Double's
    #[test]
    fn acc_double() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("acc_double");
        let initial = Some(init_each("0.0", FnConfPointType::Double));
        let input = init_each("0.0", FnConfPointType::Double);
        let mut fn_count = FnAcc::new(
            "test",
            initial,
            input.clone(),
        );
        let test_data = vec![
            (0.0f64, 0.0),
            (1.0, 1.0),
            (22.0, 23.0),
            (1457.0, 1480.0),
            (-10.0, 1470.0),
            (0.0, 1470.0),
            (99.0, 1569.0),
            (0.0, 1569.0),
            (0.0, 1569.0),
            (-2.0, 1567.0),
            (15.0, 1582.0),
            (0.0, 1582.0),
            (1.0, 1583.0),
            (0.0, 1583.0),
        ];
        for (value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_count.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_double().value, target, "\n   result: {:?} \ntarget: {}", state, target);
        }
    }
}
