#![allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use std::sync::{Once, mpsc};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{core_::{cot::cot::Cot, net::protocols::jds::jds_serialize::JdsSerialize, point::{point_hlr::PointHlr, point::Point}, status::status::Status, types::bool::Bool}, tcp::steam_read::StreamRead};
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
    ///
    fn ts() -> DateTime<Utc> {
        chrono::offset::Utc::now()
    }
    ///
    ///
    fn tsStr(ts: DateTime<Utc>) -> String {
        ts.to_rfc3339()
    }
    ///
    ///
    #[test]
    fn test_JdsSerialize() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test JdsSerialize");
        let name = "/server/line1/ied1/test1";
        let ts = ts();
        let txId = 0;
        // debug!("timestamp: {:?}", ts);j
        let test_data = [
            (
                format!(r#"{{"type": "Bool",  "name": "{}", "value": 0,   "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Bool(PointHlr::new(txId, name, Bool(false), Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "Bool",  "name": "{}", "value": 1,    "status": 0, "cot": "Act", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Bool(PointHlr::new(txId, name, Bool(true), Status::Ok, Cot::Act, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value": 1,   "status": 0, "cot": "ActCon", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Int(PointHlr::new(txId, name, 1, Status::Ok, Cot::ActCon, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value": -9223372036854775808,   "status": 0, "cot": "ActErr", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Int(PointHlr::new(txId, name, -9223372036854775808, Status::Ok, Cot::ActErr, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value":  9223372036854775807,   "status": 0, "cot": "Req", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Int(PointHlr::new(txId, name,  9223372036854775807, Status::Ok, Cot::Req, ts))
            ),
            (
                format!(r#"{{"type": "Real", "name": "{}", "value":  0.0, "status": 0, "cot": "ReqCon", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Real(PointHlr::new(txId, name,  0.0, Status::Ok, Cot::ReqCon, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  0.0, "status": 0, "cot": "ReqCon", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Double(PointHlr::new(txId, name,  0.0, Status::Ok, Cot::ReqCon, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value": -1.1, "status": 0, "cot": "ReqErr", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Double(PointHlr::new(txId, name, -1.1, Status::Ok, Cot::ReqErr, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  1.1, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Double(PointHlr::new(txId, name,  1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value": -1.7976931348623157e308, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Double(PointHlr::new(txId, name, -1.7976931348623157e308, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  1.7976931348623157e308, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::Double(PointHlr::new(txId, name,  1.7976931348623157e308, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "String","name": "{}", "value": "~!@#$%^&*()_+`1234567890-=","status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, tsStr(ts)), Point::String(PointHlr::new(txId, name, "~!@#$%^&*()_+`1234567890-=".to_string(), Status::Ok, Cot::default(), ts))
            ),
        ];
        let (send, recv) = mpsc::channel();
        let mut jdsSerialize = JdsSerialize::new("test", recv);
        for (target, point) in test_data {
            send.send(point).unwrap();
            let result = jdsSerialize.read().unwrap();
            let target: serde_json::Value = serde_json::from_str(&target).unwrap();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
}
