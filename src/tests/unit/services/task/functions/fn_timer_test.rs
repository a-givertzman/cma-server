#[cfg(test)]

mod fn_timer {
    use log::{debug, info};
    use std::{sync::Once, time::{Instant, Duration}, thread,rc::Rc, cell::RefCell};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig},
        core_::{aprox_eq::aprox_eq::AproxEq, point::point_type::ToPoint,
        types::fn_in_out_ref::FnInOutRef}, services::task::nested_function::{fn_::FnOut, fn_input::FnInput, fn_timer::FnTimer}
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
    /// Testing Task FnTimer measuring simple elapsed seconds
    #[test]
    fn elapsed_repeat_false() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_elapsed_repeat_false");
        let input = init_each("false", FnConfPointType::Bool);
        let mut fn_timer = FnTimer::new(
            "id",
            None,
            None,
            input.clone(),
            false,
        );
        let test_data = vec![
            (false, 0),
            (false, 0),
            (true, 1),
            (true, 1),
            (false, 1),
            (false, 1),
            (true, 2),
            (false, 2),
            (true, 3),
            (false, 3),
            (false, 3),
            (true, 4),
            (true, 4),
            (false, 4),
            (false, 4),
        ];
        let mut start: Option<Instant> = None;
        let mut target: f64;
        let mut elapsed: f64 = 0.0;
        let mut elapsed_total: f64 = 0.0;
        let mut done = false;
        for (value, _) in test_data {
            if !done {
                if value {
                    if start.is_none() {
                        start = Some(Instant::now());
                    } else {
                        elapsed = start.unwrap().elapsed().as_secs_f64();
                    }
                } else {
                    if start.is_some() {
                        elapsed = 0.0;
                        elapsed_total += start.unwrap().elapsed().as_secs_f64();
                        // start = None
                        done = true;
                    }
                }
            }
            target = elapsed_total + elapsed;
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let fn_timer_elapsed = fn_timer.out().unwrap().as_double().value;
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, fn_timer_elapsed);
            assert!(fn_timer_elapsed.aprox_eq(target, 2), "current '{}' != target '{}'", fn_timer_elapsed, target);
            thread::sleep(Duration::from_secs_f64(0.1));
        }
    }
    ///
    /// Testing Task FnTimer with 'repeat' option
    #[test]
    fn total_elapsed_repeat() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_total_elapsed_repeat");
        let input = init_each("false", FnConfPointType::Bool);
        let initial = init_each("0.0", FnConfPointType::Double);
        let mut fn_timer = FnTimer::new(
            "id",
            None,
            Some(initial),
            input.clone(),
            true,
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
            (true, 4),
            (false, 4),
            (false, 4),
        ];
        let mut start: Option<Instant> = None;
        let mut target: f64;
        let mut elapsed: f64 = 0.0;
        let mut elapsed_total: f64 = 0.0;
        for (value, _) in test_data {
            if value {
                if start.is_none() {
                    start = Some(Instant::now());
                } else {
                    elapsed = start.unwrap().elapsed().as_secs_f64();
                }
            } else {
                if start.is_some() {
                    elapsed = 0.0;
                    elapsed_total += start.unwrap().elapsed().as_secs_f64();
                    start = None
                }
            }
            target = elapsed_total + elapsed;
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let fn_timer_elapsed = fn_timer.out().unwrap().as_double().value;
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, fn_timer_elapsed);
            assert!(fn_timer_elapsed.aprox_eq(target, 2), "current '{}' != target '{}'", fn_timer_elapsed, target);
            thread::sleep(Duration::from_secs_f64(0.1));
        }
    }
    ///
    /// Testing Task FnTimer with 'repeat' option, useing reset
    #[test]
    fn total_elapsed_repeat_reset() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_total_elapsed_repeat_reset");
        let input = init_each("false", FnConfPointType::Bool);
        let initial = init_each("0.0", FnConfPointType::Double);
        let mut fn_timer = FnTimer::new(
            "id",
            None,
            Some(initial),
            input.clone(),
            true,
        );
        let test_data = vec![
            (false, 0, false),
            (false, 0, false),
            (true, 1, false),
            (false, 1, false),
            (false, 1, false),
            (true, 2, false),
            (false, 2, false),
            (true, 3, false),
            (false, 3, false),
            (false, 3, false),
            (true, 4, false),
            (true, 4, true),
            (true, 4, false),
            (false, 4, false),
            (false, 4, false),
        ];
        let mut start: Option<Instant> = None;
        let mut elapsed_total: f64 = 0.0;
        let mut elapsed_session: f64 = 0.0;
        let mut target;
        for (value, _, reset) in test_data {
            if reset {
                start = None;
                elapsed_session = 0.0;
                elapsed_total = 0.0;
                fn_timer.reset();
            }
            if value {
                if start.is_none() {
                    start = Some(Instant::now());
                } else {
                    elapsed_session = start.unwrap().elapsed().as_secs_f64();
                }
            } else {
                if start.is_some() {
                    elapsed_session = 0.0;
                    elapsed_total += start.unwrap().elapsed().as_secs_f64();
                    start = None;
                }
            }
            target = elapsed_total + elapsed_session;
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let fn_timer_elapsed = fn_timer.out().unwrap().as_double().value;
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}   |   target {}{}", value, fn_timer_elapsed, target, if reset {"\t<-- reset"} else {""});
            assert!(fn_timer_elapsed.aprox_eq(target, 2), "current '{}' != target '{}'", fn_timer_elapsed, target);
            thread::sleep(Duration::from_secs_f64(0.1));
        }
    }
    ///
    /// Testing Task FnTimer with initial value and 'repeat' option
    #[test]
    fn initial_repeat() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        info!("test_initial_repeat");
        let initial = 123.1234f64;
        let input = init_each("false", FnConfPointType::Bool);
        let initial_input = init_each(initial.to_string().as_str(), FnConfPointType::Double);
        let mut fn_timer = FnTimer::new(
            "id",
            None,
            Some(initial_input),
            input.clone(),
            true,
        );
        let test_data = vec![
            (00, false),
            (01, false),
            (02, true),
            (03, false),
            (04, false),
            (05, true),
            (06, false),
            (07, true),
            (08, false),
            (09, false),
            (10, true),
            (11, true),
            (12, false),
            (13, false),
        ];
        let mut start: Option<Instant> = None;
        let mut target: f64;
        let mut elapsed: f64 = 0.0;
        let mut elapsed_total: f64 = initial;
        for (step, value) in test_data {
            if value {
                if start.is_none() {
                    start = Some(Instant::now());
                } else {
                    elapsed = start.unwrap().elapsed().as_secs_f64();
                }
            } else {
                if start.is_some() {
                    elapsed = 0.0;
                    elapsed_total += start.unwrap().elapsed().as_secs_f64();
                    start = None
                }
            }
            target = elapsed_total + elapsed;
            let point = value.to_point(0, "test");
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let fn_timer_elapsed = fn_timer.out().unwrap().as_double().value;
            // debug!("input: {:?}", &mut input);
            debug!("value: {:?}   |   state: {:?}", value, fn_timer_elapsed);
            assert!(fn_timer_elapsed.aprox_eq(target, 2), "step: {} | current '{}' != target '{}'", step, fn_timer_elapsed, target);
            thread::sleep(Duration::from_secs_f64(0.3));
        }
    }
}
