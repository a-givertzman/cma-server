#[cfg(test)]
use sal_sync::services::{entity::ToPoint, task::functions::{FnConfOptions, FnConfPointType, FnConfig}};
use testing::entities::test_value::Value;
use std::{sync::Once, rc::Rc, cell::RefCell};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    domain::FnInOutRef, 
    services::task::{fn_::FnOut, fn_input::FnInput, fn_is_changed_value::FnIsChangedValue},
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
fn init_each(default: &str, name: impl Into<String>, type_: FnConfPointType) -> FnInOutRef {
    let mut conf = FnConfig { name: name.into(), type_, options: FnConfOptions {default: Some(default.into()), ..Default::default()}, ..Default::default()};
    Rc::new(RefCell::new(Box::new(
        FnInput::new("test", 0, &mut conf)
    )))
}
///
/// Testing accumulation of the Bool's
#[test]
fn is_changed_bool() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    let dbg = "is_changed_bool";
    log::info!("{}", dbg);
    let input1 = init_each("false", format!("/{}/Bool", dbg), FnConfPointType::Bool);
    let input2 = init_each("0", format!("/{}/Int", dbg), FnConfPointType::Int);
    let input3 = init_each("0.0", format!("/{}/Real", dbg), FnConfPointType::Real);
    let input4 = init_each("0.0", format!("/{}/Double", dbg), FnConfPointType::Double);
    let input5 = init_each("test", format!("/{}/String", dbg), FnConfPointType::String);
    let mut fn_is_changed = FnIsChangedValue::new(
        "test",
        vec![
            input1.clone(),
            input2.clone(),
            input3.clone(),
            input4.clone(),
            input5.clone(),
        ]
    );
    let test_data = vec![
        (00, format!("/{}/Bool", dbg),      Value::Bool(false),     1),
        (01, format!("/{}/Bool", dbg),      Value::Bool(false),     0),
        (02, format!("/{}/Bool", dbg),      Value::Bool(true),      1),
        (03, format!("/{}/Bool", dbg),      Value::Bool(true),      0),
        (04, format!("/{}/Int", dbg),       Value::Int(0),          0),
        (05, format!("/{}/Int", dbg),       Value::Int(0),          0),
        (06, format!("/{}/Real", dbg),      Value::Real(0.0),       0),
        (07, format!("/{}/Int", dbg),       Value::Int(0),          0),
        (08, format!("/{}/Real", dbg),      Value::Real(0.1),       1),
        (09, format!("/{}/Double", dbg),    Value::Double(0.1),     1),
        (10, format!("/{}/Bool", dbg),      Value::Bool(true),      0),
        (11, format!("/{}/Double", dbg),    Value::Double(0.1),     0),
        (12, format!("/{}/Real", dbg),      Value::Real(0.1),       0),
        (13, format!("/{}/Bool", dbg),      Value::Bool(true),      0),
        (13, format!("/{}/String", dbg),    Value::String("..".into()),      1),
        (14, format!("/{}/Bool", dbg),      Value::Bool(false),     1),
        (15, format!("/{}/Bool", dbg),      Value::Bool(false),     0),
        (16, format!("/{}/Double", dbg),    Value::Double(0.0),     1),
        (17, format!("/{}/Real", dbg),      Value::Real(0.1),       0),
        (18, format!("/{}/Double", dbg),    Value::Double(0.0),     0),
        (19, format!("/{}/Bool", dbg),      Value::Bool(false),     0),
    ];
    for (step, name, value, target) in test_data {
        match &value {
            Value::Bool(value) => {
                input1.borrow_mut().add(&value.to_point(0, &name))
            }
            Value::Int(value) => {
                input2.borrow_mut().add(&value.to_point(0, &name))
            }
            Value::Real(value) => {
                input3.borrow_mut().add(&value.to_point(0, &name))
            }
            Value::Double(value) => {
                input4.borrow_mut().add(&value.to_point(0, &name))
            }
            Value::String(value) => {
                input5.borrow_mut().add(&value.to_point(0, &name))
            }
            Value::Bytes(_) => {
                panic!("{dbg} value of type 'Bytes' - is not supported")
            }
        };
        // debug!("input: {:?}", &input);
        let state = fn_is_changed.out().unwrap();
        // debug!("input: {:?}", &mut input);
        log::debug!("step {}   |   value: {:?}   |   state: {:?}", step, value, state);
        assert!(state.as_bool().value.0 == (target > 0), "step {} \n result: {:?} \ntarget: {}", step, state.as_bool().value.0, target > 0);
    }
}
