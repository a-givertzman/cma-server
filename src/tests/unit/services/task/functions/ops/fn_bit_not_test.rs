#[cfg(test)]
mod fn_bit_not {
    use log::{debug, info};
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::{point::point::ToPoint, types::fn_in_out_ref::FnInOutRef}, 
        services::task::nested_function::{fn_::FnOut, fn_input::FnInput, ops::fn_bit_not::FnBitNot}
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
    /// Testing Task FnNot Bool's
    #[test]
    fn test_bool() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "test_bool";
        info!("{}", self_id);
        let mut target: bool;
        let input = init_each("false", FnConfPointType::Bool);
        let mut fn_bit_not = FnBitNot::new(
            self_id,
            input.clone(),
        );
        let test_data = vec![
            (00, false),
            (01, false),
            (02, true),
            (03, true),
        ];
        for (step, value) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            let result = fn_bit_not.out().unwrap().as_bool().value.0;
            debug!("step {}  |  ! value: {:?} | result: {:?}", step, value, result);
            target = ! value;
            assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
    }
    ///
    /// Testing Task FnNot Int's
    #[test]
    fn test_int() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "test_int";
        info!("{}", self_id);
        let mut target: i64;
        let input = init_each("0", FnConfPointType::Int);
        let mut fn_bit_not = FnBitNot::new(
            self_id,
            input.clone(),
        );
        let test_data = vec![
            (00, 1),
            (01, 5),
            (02, 3),
            (03, -1),
            (04, -5),
            (05, -4),
            (06, 4),
            (07, 0),
            (08, 0),
            (09, -4),
            (10, 0),
        ];
        for (step, value) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            let result = fn_bit_not.out().unwrap().as_int().value;
            debug!("step {}  |  ! value1: {:?} | result: {:?}", step, value, result);
            target = ! value;
            assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
    }
}
