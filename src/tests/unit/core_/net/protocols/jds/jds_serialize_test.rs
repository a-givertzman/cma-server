#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use sal_sync::services::{entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr}, status::status::Status}, types::bool::Bool};
    use std::sync::{Once, mpsc};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{core_::net::protocols::jds::jds_serialize::JdsSerialize, tcp::steam_read::StreamRead};
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
    fn ts_str(ts: DateTime<Utc>) -> String {
        ts.to_rfc3339()
    }
    ///
    ///
    #[test]
    fn test_jds_serialize() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test JdsSerialize");
        let name = "/server/line1/ied1/test1";
        let ts = ts();
        let tx_id = 0;
        // debug!("timestamp: {:?}", ts);j
        let test_data = [
            (
                format!(r#"{{"type": "Bool",  "name": "{}", "value": 0,   "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Bool(PointHlr::new(tx_id, name, Bool(false), Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "Bool",  "name": "{}", "value": 1,    "status": 0, "cot": "Act", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Bool(PointHlr::new(tx_id, name, Bool(true), Status::Ok, Cot::Act, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value": 1,   "status": 0, "cot": "ActCon", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Int(PointHlr::new(tx_id, name, 1, Status::Ok, Cot::ActCon, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value": -9223372036854775808,   "status": 0, "cot": "ActErr", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Int(PointHlr::new(tx_id, name, -9223372036854775808, Status::Ok, Cot::ActErr, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value":  9223372036854775807,   "status": 0, "cot": "Req", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Int(PointHlr::new(tx_id, name,  9223372036854775807, Status::Ok, Cot::Req, ts))
            ),
            (
                format!(r#"{{"type": "Real", "name": "{}", "value":  0.0, "status": 0, "cot": "ReqCon", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Real(PointHlr::new(tx_id, name,  0.0, Status::Ok, Cot::ReqCon, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  0.0, "status": 0, "cot": "ReqCon", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Double(PointHlr::new(tx_id, name,  0.0, Status::Ok, Cot::ReqCon, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value": -1.1, "status": 0, "cot": "ReqErr", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Double(PointHlr::new(tx_id, name, -1.1, Status::Ok, Cot::ReqErr, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  1.1, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Double(PointHlr::new(tx_id, name,  1.1, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value": -1.7976931348623157e308, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Double(PointHlr::new(tx_id, name, -1.7976931348623157e308, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  1.7976931348623157e308, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::Double(PointHlr::new(tx_id, name,  1.7976931348623157e308, Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "String","name": "{}", "value": "~!@#$%^&*()_+`1234567890-=","status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                name, ts_str(ts)), Point::String(PointHlr::new(tx_id, name, "~!@#$%^&*()_+`1234567890-=".to_string(), Status::Ok, Cot::default(), ts))
            ),
        ];
        let (send, recv) = mpsc::channel();
        let mut jds_serialize = JdsSerialize::new("test", recv);
        for (target, point) in test_data {
            send.send(point).unwrap();
            let result = jds_serialize.read().unwrap();
            let target: serde_json::Value = serde_json::from_str(&target).unwrap();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
}
