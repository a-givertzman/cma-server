#![allow(non_snake_case)]
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
#[cfg(test)]
use log::{debug, info};
use std::{sync::Once, str::FromStr};

use crate::conf::fn_conf_keywd::{FnConfKeywd, FnConfKeywdValue, FnConfPointType};

// Note this useful idiom: importing names from outer (for mod tests) scope.
// use super::*;

static INIT: Once = Once::new();

///
/// once called initialisation
fn initOnce() {
    INIT.call_once(|| {
            // implement your initialisation code to be called only once for current test file
        }
    )
}


///
/// returns:
///  - ...
fn initEach() -> () {

}


#[test]
fn test_create_valid() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    initOnce();
    initEach();
    println!("test_create_valid");
    // let (initial, switches) = initEach();
    let testData = vec![
        ("input1 fn fnName", FnConfKeywd::Fn( FnConfKeywdValue {input: format!("input1"), type_: FnConfPointType::Unknown, data: format!("fnName")} )),
        ("fn name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name")} )),
        ("fn  name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name")} )),
        ("fn   name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name")} )),
        ("fn\tname", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name")} )),
        ("let name", FnConfKeywd::Var( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name")} )),
        ("input1 const", FnConfKeywd::Const( FnConfKeywdValue {input: format!("input1"), type_: FnConfPointType::Unknown, data: format!("")} )),
        ("const name", FnConfKeywd::Const( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name")} )),
        ("input2 const name", FnConfKeywd::Const( FnConfKeywdValue {input: format!("input2"), type_: FnConfPointType::Unknown, data: format!("name")} )),
        ("point /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name")} )),
        ("point '/path/Point.Name'", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name")} )),
        ("point \"/path/Point.Name\"", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name")} )),
        ("input1 point /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input1"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name")} )),
        ("input2 point '/path/Point.Name'", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input2"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name")} )),
        ("input3 point \"/path/Point.Name\"", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input3"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name")} )),
        ("input4 point bool /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input4"), type_: FnConfPointType::Bool, data: format!("/path/Point.Name")} )),
        ("input5 point int /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input5"), type_: FnConfPointType::Int, data: format!("/path/Point.Name")} )),
        ("input6 point float /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input6"), type_: FnConfPointType::Float, data: format!("/path/Point.Name")} )),
        ("input7 point string /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input7"), type_: FnConfPointType::String, data: format!("/path/Point.Name")} )),
    ];
    for (value, target) in testData {
        let fnConfigType = FnConfKeywd::from_str(value).unwrap();
        debug!("value: {:?}   |   fnConfigType: {:?}   |   target: {:?}", value, fnConfigType, target);
        assert_eq!(fnConfigType, target);
    }
}

#[test]
fn test_create_invalid() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    initOnce();
    initEach();
    info!("test_create_invalid");
    // let (initial, switches) = initEach();
    let testData: Vec<(&str, Result<&str, ()>)> = vec![
        ("fn:name", Err(())),
        ("fn\nname", Err(())),
        ("fn: name", Err(())),
        ("fn :name", Err(())),
        ("fn : name", Err(())),
        ("Fn name", Err(())),
        ("FN name", Err(())),
        ("fnName", Err(())),
        ("fn_name", Err(())),
        ("let:name", Err(())),
        ("Let name", Err(())),
        ("LET name", Err(())),
        ("letName", Err(())),
        ("let_name", Err(())),
        ("const:name", Err(())),
        ("Const name", Err(())),
        ("CONST name", Err(())),
        ("constName", Err(())),
        ("const_name", Err(())),
        ("point:name", Err(())),
        ("Point name", Err(())),
        ("POINT name", Err(())),
        ("pointName", Err(())),
        ("point_name", Err(())),
    ];
    for (value, target) in testData {
        let fnConfigType = FnConfKeywd::from_str(value);
        debug!("value: {:?}   |   fnConfigType: {:?}   |   target: {:?}", value, fnConfigType, target);
        assert_eq!(fnConfigType.is_err(), true);
    }
}
