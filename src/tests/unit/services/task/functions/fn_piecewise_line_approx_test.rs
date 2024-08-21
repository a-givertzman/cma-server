#[cfg(test)]
mod fn_piecewise_line_approx {
    use log::{debug, info};
    use std::{sync::Once, rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::{aprox_eq::aprox_eq::AproxEq, point::point_type::ToPoint, types::fn_in_out_ref::FnInOutRef}, 
        services::task::nested_function::{fn_::FnOut, fn_input::FnInput, fn_piecewise_line_approx::FnPiecewiseLineApprox},
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
    /// Testing FnPiecewiseLineApprox with Int's
    #[test]
    fn line_approx_int() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        info!("line_approx_int");
        let input = init_each("0", FnConfPointType::Int);
        let mut fn_line_approx = FnPiecewiseLineApprox::new(
            "test",
            input.clone(),
            serde_yaml::from_str("
                0: 0
                5: 0
                10: 3
            ").unwrap(),
        );
        info!("fn: {:#?}", fn_line_approx);
        let test_data = vec![
            (00, -1, 0),
            (01, 0, 0),
            (02, 1, 0),
            (03, 2, 0),
            (04, 3, 0),
            (05, 4, 0),
            (06, 5, 0),
            (07, 6, 1),
            (08, 7, 1),
            (09, 8, 2),
            (10, 9, 2),
            (11, 10, 3),
            (12, 11, 3),
            (13, 12, 3),
            (14, 13, 3),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_line_approx.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_int().value, target, "step: {}\n result: {:?} \ntarget: {}", step, state.as_int().value, target);
        }
    }
    ///
    /// Testing FnPiecewiseLineApprox with Real's
    #[test]
    fn line_approx_real() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        info!("line_approx_real");
        let input = init_each("0.0", FnConfPointType::Real);
        let mut fn_line_approx = FnPiecewiseLineApprox::new(
            "test",
            input.clone(),
            serde_yaml::from_str("
                0: 0
                5: 0
                10: 3
                20: 1
            ").unwrap(),
        );
        info!("fn: {:#?}", fn_line_approx);
        let test_data = vec![
            (00, -1.0, 0.0),
            (01, 0.0, 0.0),
            (02, 1.0, 0.0),
            (03, 2.0, 0.0),
            (04, 3.0, 0.0),
            (05, 4.0, 0.0),
            (06, 5.0, 0.0),
            (07, 6.0, 0.6),
            (08, 7.0, 1.2),
            (09, 8.0, 1.8),
            (10, 9.0, 2.4),
            (11, 10.0, 3.0),
            (12, 11.0, 2.8),
            (13, 12.0, 2.6),
            (14, 13.0, 2.4),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_line_approx.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert_eq!(state.as_real().value, target, "step: {}\n result: {:?} \ntarget: {}", step, state.as_real().value, target);
        }
    }
    ///
    /// Testing FnPiecewiseLineApprox with Double's
    #[test]
    fn line_approx_double() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        info!("line_approx_double");
        let input = init_each("0.0", FnConfPointType::Double);
        let mut fn_line_approx = FnPiecewiseLineApprox::new(
            "test",
            input.clone(),
            serde_yaml::from_str("
                0: 0
                5: 0
                10: 3
                20: 1
            ").unwrap(),
        );
        info!("fn: {:#?}", fn_line_approx);
        let test_data = vec![
            (00, -1.0, 0.0),
            (01, 0.0, 0.0),
            (02, 1.0, 0.0),
            (03, 2.0, 0.0),
            (04, 3.0, 0.0),
            (05, 4.0, 0.0),
            (06, 5.0, 0.0),
            (07, 6.0, 0.6),
            (08, 7.0, 1.2),
            (09, 8.0, 1.8),
            (10, 9.0, 2.4),
            (11, 10.0, 3.0),
            (12, 11.0, 2.8),
            (13, 12.0, 2.6),
            (14, 13.0, 2.4),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let state = fn_line_approx.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, state);
            assert!(state.as_double().value.aprox_eq(target, 4), "step: {}\n result: {:?} \ntarget: {}", step, state.as_double().value, target);
        }
    }
}
