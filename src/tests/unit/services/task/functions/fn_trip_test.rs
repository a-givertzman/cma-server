#[cfg(test)]

mod fn_trip {
    use log::{debug, info};
    use sal_sync::services::entity::point::point::ToPoint;
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig},
        core_::types::fn_in_out_ref::FnInOutRef,
        services::task::nested_function::{comp::fn_ge::FnGe, fn_::FnOut, fn_input::FnInput},
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
    fn single_int() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_single");
        // let (initial, switches) = init_each();
        let input1 = init_each("0", FnConfPointType::Int);
        let input2 = init_each("0", FnConfPointType::Int);
        let mut fn_trip = FnGe::new(
            "test",
            input1.clone(),
            input2.clone(),
        );
        let test_data = vec![
            (-1, 0, false),
            (0, 1, false),
            (-2, -1, false),
            (0, 1, false),
            (0, 0, true),
            (2, 1, true),
            (i64::MAX, 5, true),
            (3, 4, false),
            (2, 3, false),
            (1, 2, false),
            (0, 1, false),
            (-1, 0, false),
        ];
        for (value1, value2, target_state) in test_data {
            let point1 = value1.to_point(0, "point1");
            let point2 = value2.to_point(0, "point2");
            input1.borrow_mut().add(&point1);
            input2.borrow_mut().add(&point2);
            // debug!("input: {:?}", &input);
            let state = fn_trip.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value1: {:?}  >=  value2: {:?}  |   state: {:?}", value1, value2, state);
            assert_eq!(state.as_bool().value.0, target_state);
        }
    }
    ///
    ///
    #[test]
    fn multiple_int() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_single");

        // let (initial, switches) = init_each();
        let input1 = init_each("0", FnConfPointType::Int);
        let input2 = init_each("0", FnConfPointType::Int);
        let mut fn_trip = FnGe::new(
            "test",
            input1.clone(),
            input2.clone(),
        );
        let test_data = vec![
            (-1, 0, false),
            (0, 1, false),
            (1, 2, false),
            (3, 3, true),
            (2, 3, false),
            (5, 3, true),
            (6, 3, true),
            (2, 3, false),
            (1, 2, false),
            (2, 3, false),
            (4, 4, true),
            (5, 4, true),
            (3, 4, false),
            (2, 3, false),
            (1, 2, false),
            (0, 1, false),
        ];
        for (value1, value2, target_dtate) in test_data {
            let point1 = value1.to_point(0, "point1");
            let point2 = value2.to_point(0, "point2");
            input1.borrow_mut().add(&point1);
            input2.borrow_mut().add(&point2);
            // debug!("input: {:?}", &input);
            let state = fn_trip.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value1: {:?}  >=  value2: {:?}  |   state: {:?}", value1, value2, state);
            assert_eq!(state.as_bool().value.0, target_dtate);
        }
    }
    ///
    ///
    #[test]
    fn multiple_real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_single");
        // let (initial, switches) = init_each();
        let input1 = init_each("0.0", FnConfPointType::Real);
        let input2 = init_each("0.0", FnConfPointType::Real);
        let mut fn_trip = FnGe::new(
            "test",
            input1.clone(),
            input2.clone(),
        );
        let test_data = vec![
            (-0.1f32, 0.0, false),
            ( 1.0f32, 1.1, false),
            ( 2.0f32, 2.2, false),
            ( 5.0f32, 5.0, true),
            ( 3.0f32, 3.1, false),
            ( 5.0f32, 5.0, true),
            ( 5.1f32, 5.0, true),
            ( 4.9f32, 5.0, false),
            ( 4.8f32, 5.0, false),
            ( 4.7f32, 5.0, false),
            ( 5.1f32, 5.0, true),
            ( 5.2f32, 5.0, true),
            ( 2.0f32, 3.0, false),
            ( 1.0f32, 2.0, false),
            ( 0.0f32, 1.0, false),
            (-0.1f32, 0.0, false),
        ];
        for (value1, value2, target_state) in test_data {
            let point1 = value1.to_point(0, "point1");
            let point2 = value2.to_point(0, "point2");
            input1.borrow_mut().add(&point1);
            input2.borrow_mut().add(&point2);
            // debug!("input: {:?}", &input);
            let state = fn_trip.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value1: {:?}  >=  value2: {:?}  |   state: {:?}", value1, value2, state);
            assert_eq!(state.as_bool().value.0, target_state);
        }
    }
    ///
    ///
    #[test]
    fn multiple_double() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_single");
        // let (initial, switches) = init_each();
        let input1 = init_each("0.0", FnConfPointType::Real);
        let input2 = init_each("0.0", FnConfPointType::Real);
        let mut fn_trip = FnGe::new(
            "test",
            input1.clone(),
            input2.clone(),
        );
        let test_data = vec![
            (-0.1f64, 0.0, false),
            ( 1.0f64, 1.1, false),
            ( 2.0f64, 2.2, false),
            ( 5.0f64, 5.0, true),
            ( 3.0f64, 3.1, false),
            ( 5.0f64, 5.0, true),
            ( 5.1f64, 5.0, true),
            ( 4.9f64, 5.0, false),
            ( 4.8f64, 5.0, false),
            ( 4.7f64, 5.0, false),
            ( 5.1f64, 5.0, true),
            ( 5.2f64, 5.0, true),
            ( 2.0f64, 3.0, false),
            ( 1.0f64, 2.0, false),
            ( 0.0f64, 1.0, false),
            (-0.1f64, 0.0, false),
        ];
        for (value1, value2, target_state) in test_data {
            let point1 = value1.to_point(0, "point1");
            let point2 = value2.to_point(0, "point2");
            input1.borrow_mut().add(&point1);
            input2.borrow_mut().add(&point2);
            // debug!("input: {:?}", &input);
            let state = fn_trip.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value1: {:?}  >=  value2: {:?}  |   state: {:?}", value1, value2, state);
            assert_eq!(state.as_bool().value.0, target_state);
        }
    }
}