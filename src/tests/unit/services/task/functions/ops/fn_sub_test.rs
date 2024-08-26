#[cfg(test)]
mod fn_sub {
    use log::{debug, info};
    use sal_sync::services::entity::point::point::{Point, ToPoint};
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::types::fn_in_out_ref::FnInOutRef, 
        services::task::nested_function::{fn_::FnOut, fn_input::FnInput, ops::fn_sub::FnSub}
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
    /// Testing Task Sub Bool's
    #[ignore = "Task FnSub ignored for Bool's - not implemented, under discussion"]
    #[test]
    fn test_bool() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        info!("test_bool");
        let mut value1_stored;
        let mut value2_stored = false.to_point(0, "bool");
        let mut target: Point;
        let input1 = init_each("false", FnConfPointType::Bool);
        let input2 = init_each("false", FnConfPointType::Bool);
        let mut fn_sub = FnSub::new(
            "test",
            input1.clone(),
            input2.clone(),
        );
        let test_data = vec![
            (false, false),
            (false, true),
            (false, false),
            (true, false),
            (false, false),
            (true, true),
            (false, false),
        ];
        for (value1, value2) in test_data {
            let point1 = value1.to_point(0, "test");
            let point2 = value2.to_point(0, "test");
            input1.borrow_mut().add(&point1);
            let state = fn_sub.out().unwrap();
            debug!("value1: {:?}   |   state: {:?}", value1, state);
            value1_stored = point1.clone();
            target = Point::Bool(value1_stored.as_bool() + value2_stored.as_bool());
            assert_eq!(state, target);
            input2.borrow_mut().add(&point2);
            let state = fn_sub.out().unwrap();
            debug!("value2: {:?}   |   state: {:?}", value2, state);
            value2_stored = point2.clone();
            target = Point::Bool(value1_stored.as_bool() + value2_stored.as_bool());
            assert_eq!(state, target);
            println!();
        }
    }
    ///
    /// Testing Task Sub Int's
    #[test]
    fn test_int() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_int");
        let mut value1_stored;
        let mut value2_stored = 0.to_point(0, "int");
        let mut target: Point;
        let input1 = init_each("0", FnConfPointType::Int);
        let input2 = init_each("0", FnConfPointType::Int);
        let mut fn_sub = FnSub::new(
            "test",
            input1.clone(),
            input2.clone(),
        );
        let test_data = vec![
            (1, 1),
            (2, 2),
            (5, 5),
            (-1, 1),
            (-5, 1),
            (1, -1),
            (1, -5),
            (0, 0),
            (i64::MIN, 0),
            (0, i64::MIN),
            (i64::MAX, 0),
            (0, i64::MAX),
        ];
        for (value1, value2) in test_data {
            let point1 = value1.to_point(0, "test");
            let point2 = value2.to_point(0, "test");
            input1.borrow_mut().add(&point1);
            let state = fn_sub.out().unwrap();
            debug!("value1: {:?}   |   state: {:?}", value1, state);
            value1_stored = point1.clone();
            target = Point::Int(value1_stored.as_int() - value2_stored.as_int());
            assert_eq!(state, target);
            input2.borrow_mut().add(&point2);
            let state = fn_sub.out().unwrap();
            debug!("value2: {:?}   |   state: {:?}", value2, state);
            value2_stored = point2.clone();
            target = Point::Int(value1_stored.as_int() - value2_stored.as_int());
            assert_eq!(state, target);
            println!();
        }
    }
    ///
    /// Testing Task Sub Real's
    #[test]
    fn real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("fn_sub_real");
        let mut value1_stored;
        let mut value2_stored = 0.0f32.to_point(0, "real");
        let mut target: f32;
        let input1 = init_each("0.0", FnConfPointType::Real);
        let input2 = init_each("0.0", FnConfPointType::Real);
        let mut fn_sub = FnSub::new(
            "test",
            input1.clone(),
            input2.clone(),
        );
        let test_data = vec![
            (01, 0.1, 0.1),
            (02, 0.2, 0.2),
            (03, 0.5, 0.5),
            (04, -0.1, 0.1),
            (05, -0.5, 0.1),
            (06, 0.1, -0.1),
            (07, 0.1, -0.5),
            (08, 0.0, 0.0),
            (09, f32::MIN, 0.0),
            (10, f32::MIN, 0.5),
            (11, f32::MIN, 1.0),
            (12, 0.0, f32::MIN),
            (13, 0.5, f32::MIN),
            (14, 1.0, f32::MIN),
            (15, f32::MAX, 0.0),
            (16, f32::MAX, 0.5),
            (17, f32::MAX, 1.0),
            (18, 0.0, f32::MAX),
            (19, 0.5, f32::MAX),
            (20, 1.0, f32::MAX),
        ];
        for (step, value1, value2) in test_data {
            let point1 = value1.to_point(0, "test");
            let point2 = value2.to_point(0, "test");
            input1.borrow_mut().add(&point1);
            let state = fn_sub.out().unwrap();
            debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
            value1_stored = point1.clone();
            target = value1_stored.as_real().value - value2_stored.as_real().value;
            let result = state.as_real().value;
            assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
            input2.borrow_mut().add(&point2);
            let state = fn_sub.out().unwrap();
            debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
            value2_stored = point2.clone();
            target = value1_stored.as_real().value - value2_stored.as_real().value;
            let result = state.as_real().value;
            assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
            println!();
        }
    }
    ///
    /// Testing Task Sub Double's
    #[test]
    fn double() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("fn_sub_double");
        let mut value1_stored;
        let mut value2_stored = 0.0f64.to_point(0, "double");
        let mut target: f64;
        let input1 = init_each("0.0", FnConfPointType::Double);
        let input2 = init_each("0.0", FnConfPointType::Double);
        let mut fn_sub = FnSub::new(
            "test",
            input1.clone(),
            input2.clone(),
        );
        let test_data = vec![
            (01, 0.1, 0.1),
            (02, 0.2, 0.2),
            (03, 0.5, 0.5),
            (04, -0.1, 0.1),
            (05, -0.5, 0.1),
            (06, 0.1, -0.1),
            (07, 0.1, -0.5),
            (08, 0.0, 0.0),
            (09, f64::MIN, 0.0),
            (10, f64::MIN, 0.5),
            (11, f64::MIN, 1.0),
            (12, 0.0, f64::MIN),
            (13, 0.5, f64::MIN),
            (14, 1.0, f64::MIN),
            (15, f64::MAX, 0.0),
            (16, f64::MAX, 0.5),
            (17, f64::MAX, 1.0),
            (18, 0.0, f64::MAX),
            (19, 0.5, f64::MAX),
            (20, 1.0, f64::MAX),
        ];
        for (step, value1, value2) in test_data {
            let point1 = value1.to_point(0, "test");
            let point2 = value2.to_point(0, "test");
            input1.borrow_mut().add(&point1);
            let state = fn_sub.out().unwrap();
            debug!("step: {}  |  value1: {:?}   |   state: {:?}", step, value1, state);
            value1_stored = point1.clone();
            target = value1_stored.as_double().value - value2_stored.as_double().value;
            let result = state.as_double().value;
            assert_eq!(result, target, "\n result: {} \n target: {}", result, target);
            input2.borrow_mut().add(&point2);
            let state = fn_sub.out().unwrap();
            debug!("step: {}  |  value2: {:?}   |   state: {:?}", step, value2, state);
            value2_stored = point2.clone();
            target = value1_stored.as_double().value - value2_stored.as_double().value;
            let result = state.as_double().value;
            assert_eq!(result, target, "step {} \n result: {} \n target: {}", step, result, target);
            println!();
        }
    }
}
