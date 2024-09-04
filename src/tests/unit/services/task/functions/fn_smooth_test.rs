#[cfg(test)]
mod fn_smooth {
    use log::{debug, info};
    use sal_sync::services::entity::point::point::ToPoint;
    use testing::entities::test_value::Value;
    use std::{cell::RefCell, rc::Rc, sync::Once};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{
        conf::fn_::{fn_conf_keywd::FnConfPointType, fn_conf_options::FnConfOptions, fn_config::FnConfig}, 
        core_::{
            aprox_eq::aprox_eq::AproxEq, types::fn_in_out_ref::FnInOutRef
        },
        services::task::nested_function::{
            filter::fn_smooth::FnSmooth, fn_::FnOut, fn_input::FnInput
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
    /// Threshold Reals's
    #[test]
    fn fn_smooth_real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "fn_smooth_real";
        info!("{}", self_id);
        let factor = init_each(&self_id, Value::Double(0.125));
        let input = init_each(&self_id, Value::Real(0.0));
        let mut fn_smooth = FnSmooth::new(
            self_id,
            factor,
            input.clone(),
        );
        let test_data = vec![
        //  step    input  target
            (0,    0.00,        0.0),
            (1,    0.00,        0.0),
            (2,    3.30,        0.4125),
            (3,    0.10,        0.3734375),
            (4,    0.00,        0.3267578125),
            (5,    1.60,        0.4859130859375),
            (6,    0.00,        0.425173950195313),
            (7,    7.20,        1.2720272064209),
            (8,    0.00,        1.11302380561829),
            (9,    0.30,        1.011395829916),
            (10,    2.20,        1.1599713511765),
            (11,    8.10,        2.02747493227944),
            (12,    1.90,        2.01154056574451),
            (13,    0.10,        1.77259799502644),
            (14,    0.00,        1.55102324564814),
            (15,    0.00,        1.35714533994212),
            (16,    5.00,        1.81250217244936),
            (17,    2.00,        1.83593940089319),
            (18,    1.00,        1.73144697578154),
            (19,    0.00,        1.51501610380885),
            (20,    2.00,        1.57563909083274),
            (21,    4.00,        1.87868420447865),
            (22,    6.00,        2.39384867891882),
            (23,    12.00,        3.59461759405396),
            (24,    64.00,        11.1452903947972),
            (25,    128.00,        25.7521290954476),
            (26,    120.00,        37.5331129585166),
            (27,    133.00,        49.466473838702),
            (28,    121.00,        58.4081646088643),
            (29,    130.00,        67.3571440327563),
            (30,    127.00,        74.8125010286617),
            (31,    123.00,        80.835938400079),
            (32,    122.00,        85.9814461000691),
            (33,    120.00,        90.2337653375605),
            (34,    64.00,        86.9545446703654),
            (35,    32.00,        80.0852265865698),
            (36,    24.00,        73.0745732632485),
            (37,    12.00,        65.4402516053425),
            (38,    8.00,        58.2602201546747),
            (39,    17.00,        53.1026926353403),
            (40,    10.00,        47.7148560559228),
            (41,    7.00,        42.6254990489324),
            (42,    3.00,        37.6723116678159),
            (43,    6.00,        33.7132727093389),
            (44,    4.00,        29.9991136206715),
            (45,    2.00,        26.4992244180876),
            (46,    0.00,        23.1868213658266),
            (47,    4.00,        20.7884686950983),
            (48,    2.00,        18.439910108211),
            (49,    1.00,        16.2599213446847),
            (50,    3.00,        14.6024311765991),
            (51,    0.00,        12.7771272795242),
            (52,    2.00,        11.4299863695837),
            (53,    1.00,        10.1262380733857),
            (54,    0.70,        8.94795831421249),
            (55,    0.80,        7.92946352493593),
            (56,    0.40,        6.98828058431894),
            (57,    0.30,        6.15224551127907),
            (58,    0.20,        5.40821482236919),
            (59,    0.10,        4.74468796957304),
            (60,    0.00,        4.15160197337641),
            (61,    0.00,        3.63265172670436),
            (62,    0.00,        3.17857026086631),
            (63,    0.00,        2.78124897825802),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, &format!("input step {}", step));
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_smooth.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_real().value.aprox_eq(target, 4), "step {}\nresult: {:?}\ntarget: {:?}", step, result, target);
            println!("------------")
        }
    }
    ///
    /// Threshold Double's
    #[test]
    fn fn_smooth_double() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        let self_id = "fn_smooth_double";
        info!("{}", self_id);
        let factor = init_each(&self_id, Value::Double(0.125));
        let input = init_each(&self_id, Value::Double(0.0));
        let mut fn_smooth = FnSmooth::new(
            self_id,
            factor,
            input.clone(),
        );
        let test_data = vec![
        //  step    input  target
            (0,    0.00,        0.0),
            (1,    0.00,        0.0),
            (2,    3.30,        0.4125),
            (3,    0.10,        0.3734375),
            (4,    0.00,        0.3267578125),
            (5,    1.60,        0.4859130859375),
            (6,    0.00,        0.425173950195313),
            (7,    7.20,        1.2720272064209),
            (8,    0.00,        1.11302380561829),
            (9,    0.30,        1.011395829916),
            (10,    2.20,        1.1599713511765),
            (11,    8.10,        2.02747493227944),
            (12,    1.90,        2.01154056574451),
            (13,    0.10,        1.77259799502644),
            (14,    0.00,        1.55102324564814),
            (15,    0.00,        1.35714533994212),
            (16,    5.00,        1.81250217244936),
            (17,    2.00,        1.83593940089319),
            (18,    1.00,        1.73144697578154),
            (19,    0.00,        1.51501610380885),
            (20,    2.00,        1.57563909083274),
            (21,    4.00,        1.87868420447865),
            (22,    6.00,        2.39384867891882),
            (23,    12.00,        3.59461759405396),
            (24,    64.00,        11.1452903947972),
            (25,    128.00,        25.7521290954476),
            (26,    120.00,        37.5331129585166),
            (27,    133.00,        49.466473838702),
            (28,    121.00,        58.4081646088643),
            (29,    130.00,        67.3571440327563),
            (30,    127.00,        74.8125010286617),
            (31,    123.00,        80.835938400079),
            (32,    122.00,        85.9814461000691),
            (33,    120.00,        90.2337653375605),
            (34,    64.00,        86.9545446703654),
            (35,    32.00,        80.0852265865698),
            (36,    24.00,        73.0745732632485),
            (37,    12.00,        65.4402516053425),
            (38,    8.00,        58.2602201546747),
            (39,    17.00,        53.1026926353403),
            (40,    10.00,        47.7148560559228),
            (41,    7.00,        42.6254990489324),
            (42,    3.00,        37.6723116678159),
            (43,    6.00,        33.7132727093389),
            (44,    4.00,        29.9991136206715),
            (45,    2.00,        26.4992244180876),
            (46,    0.00,        23.1868213658266),
            (47,    4.00,        20.7884686950983),
            (48,    2.00,        18.439910108211),
            (49,    1.00,        16.2599213446847),
            (50,    3.00,        14.6024311765991),
            (51,    0.00,        12.7771272795242),
            (52,    2.00,        11.4299863695837),
            (53,    1.00,        10.1262380733857),
            (54,    0.70,        8.94795831421249),
            (55,    0.80,        7.92946352493593),
            (56,    0.40,        6.98828058431894),
            (57,    0.30,        6.15224551127907),
            (58,    0.20,        5.40821482236919),
            (59,    0.10,        4.74468796957304),
            (60,    0.00,        4.15160197337641),
            (61,    0.00,        3.63265172670436),
            (62,    0.00,        3.17857026086631),
            (63,    0.00,        2.78124897825802),
        ];
        for (step, value, target) in test_data {
            let point = value.to_point(0, &format!("input step {}", step));
            input.borrow_mut().add(&point);
            // debug!("input: {:?}", &input);
            let result = fn_smooth.out().unwrap();
            // debug!("input: {:?}", &mut input);
            debug!("step {} \t value: {:?}   |   result: {:?}", step, value, result);
            assert!(result.as_double().value.aprox_eq(target, 6), "step {}\nresult: {:?}\ntarget: {:?}", step, result, target);
            println!("------------")
        }
    }
}
