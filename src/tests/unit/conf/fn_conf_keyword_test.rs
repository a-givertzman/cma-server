#[cfg(test)]

mod fn_conf_keywd {
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use log::{debug, info};
    use sal_sync::services::entity::status::status::Status;
    use std::{sync::Once, str::FromStr};
    use crate::conf::fn_::{
        fn_conf_options::FnConfOptions,
        fn_conf_keywd::{FnConfKeywd, FnConfKeywdValue, FnConfPointType},
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
    fn init_each() -> () {}
    ///
    /// Testing FnConfKeywd::from_str for valid input
    #[test]
    fn valid() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!("test_create_valid");
        // let (initial, switches) = init_each();
        let test_data = vec![
            ("input1 fn fnName", FnConfKeywd::Fn( FnConfKeywdValue {input: format!("input1"), type_: FnConfPointType::Unknown, data: format!("fnName"), options: FnConfOptions::default()} )),
            ("fn name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("fn  name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("fn   name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("fn\tname", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("let name", FnConfKeywd::Var( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("input1 const", FnConfKeywd::Const( FnConfKeywdValue {input: format!("input1"), type_: FnConfPointType::Unknown, data: format!(""), options: FnConfOptions::default()} )),
            ("const name", FnConfKeywd::Const( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("input2 const name", FnConfKeywd::Const( FnConfKeywdValue {input: format!("input2"), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("point /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("point '/path/Point.Name'", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("point \"/path/Point.Name\"", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input1 point /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input1"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input2 point '/path/Point.Name'", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input2"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input3 point \"/path/Point.Name\"", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input3"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input4 point bool /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input4"), type_: FnConfPointType::Bool, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input5 point int /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input5"), type_: FnConfPointType::Int, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input6 point real /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input6"), type_: FnConfPointType::Real, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input6 point double /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input6"), type_: FnConfPointType::Double, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input7 point string /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input7"), type_: FnConfPointType::String, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
        ];
        for (value, target) in test_data {
            let fn_config_type = FnConfKeywd::from_str(value).unwrap();
            debug!("value: {:?}   |   fnConfigType: {:?}   |   target: {:?}", value, fn_config_type, target);
            assert_eq!(fn_config_type, target);
        }
    }
    ///
    /// Testing FnConfKeywd::from_str for invalid input
    #[test]
    fn invalid() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test_create_invalid");
        // let (initial, switches) = init_each();
        let test_data: Vec<(&str, Result<&str, ()>)> = vec![
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
        for (value, target) in test_data {
            let fn_config_type = FnConfKeywd::from_str(value);
            debug!("value: {:?}   |   fnConfigType: {:?}   |   target: {:?}", value, fn_config_type, target);
            assert_eq!(fn_config_type.is_err(), true);
        }
    }
    ///
    /// Testing FnConfKeywd::from_str for valid input with options
    #[test]
    fn valid_options() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        debug!("valid_options");
        // let (initial, switches) = init_each();
        let test_data = vec![
            ("input1 fn fnName", FnConfKeywd::Fn( FnConfKeywdValue {input: format!("input1"), type_: FnConfPointType::Unknown, data: format!("fnName"), options: FnConfOptions::default()} )),
            ("fn name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("fn  name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("fn   name", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("fn\tname", FnConfKeywd::Fn( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("let name", FnConfKeywd::Var( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("input1 const", FnConfKeywd::Const( FnConfKeywdValue {input: format!("input1"), type_: FnConfPointType::Unknown, data: format!(""), options: FnConfOptions::default()} )),
            ("const name", FnConfKeywd::Const( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("input2 const name", FnConfKeywd::Const( FnConfKeywdValue {input: format!("input2"), type_: FnConfPointType::Unknown, data: format!("name"), options: FnConfOptions::default()} )),
            ("point /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("point '/path/Point.Name'", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("point \"/path/Point.Name\"", FnConfKeywd::Point( FnConfKeywdValue {input: format!(""), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input11 point /path/Point.Name status ok", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input11"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions {status: Some(Status::Ok), default: None}} )),
            ("input2 point '/path/Point.Name' default 0.753", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input2"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions {status: None, default: Some("0.753".to_owned())}} )),
            ("input3 point \"/path/Point.Name\"", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input3"), type_: FnConfPointType::Unknown, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
            ("input4 point bool /path/Point.Name default true", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input4"), type_: FnConfPointType::Bool, data: format!("/path/Point.Name"), options: FnConfOptions {status: None, default: Some("true".to_owned())}} )),
            ("input5 point int /path/Point.Name default 175 status Invalid", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input5"), type_: FnConfPointType::Int, data: format!("/path/Point.Name"), options: FnConfOptions {status: Some(Status::Invalid), default: Some("175".to_owned())}} )),
            ("input6 point real /path/Point.Name status ok", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input6"), type_: FnConfPointType::Real, data: format!("/path/Point.Name"), options: FnConfOptions {status: Some(Status::Ok), default: None}} )),
            ("input6 point double /path/Point.Name status ok default 3.345", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input6"), type_: FnConfPointType::Double, data: format!("/path/Point.Name"), options: FnConfOptions {status: Some(Status::Ok), default: Some("3.345".to_owned())}} )),
            ("input7 point string /path/Point.Name", FnConfKeywd::Point( FnConfKeywdValue {input: format!("input7"), type_: FnConfPointType::String, data: format!("/path/Point.Name"), options: FnConfOptions::default()} )),
        ];
        for (value, target) in test_data {
            let result = FnConfKeywd::from_str(value).unwrap();
            debug!("value: {:?}   |   fnConfigType: {:?}   |   target: {:?}", value, result, target);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
}
