#[cfg(test)]

mod jds_encode_message {
    use chrono::{DateTime, Utc};
    use sal_sync::services::{entity::{cot::Cot, point::{point::Point, point_hlr::PointHlr}, status::status::Status}, types::bool::Bool};
    use std::sync::{Once, mpsc};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::{core_::net::protocols::jds::{jds_encode_message::JdsEncodeMessage, jds_serialize::JdsSerialize}, tcp::steam_read::StreamRead};
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
    fn basic() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test");
        let name = "/server/line1/ied1/test";
        let ts = ts();
        let tx_id = 0;
        // debug!("timestamp: {:?}", ts);j
        let test_data = [
            (
                format!(r#"{{"type": "Bool",  "name": "{}", "value": 0,   "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                &format!("{}00", name), ts_str(ts)), Point::Bool(PointHlr::new(tx_id, &format!("{}00", name), Bool(false), Status::Ok, Cot::default(), ts))
            ),
            (
                format!(r#"{{"type": "Bool",  "name": "{}", "value": 0,   "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                &format!("{}01", name), ts_str(ts)), Point::Bool(PointHlr::new(tx_id, &format!("{}01", name), Bool(false), Status::Ok, Cot::Inf, ts))
            ),
            (
                format!(r#"{{"type": "Bool",  "name": "{}", "value": 1,    "status": 0, "cot": "Act", "timestamp":"{}"}}"#,
                &format!("{}02", name), ts_str(ts)), Point::Bool(PointHlr::new(tx_id, &format!("{}02", name), Bool(true), Status::Ok, Cot::Act, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value": 1,   "status": 0, "cot": "ActCon", "timestamp":"{}"}}"#,
                &format!("{}03", name), ts_str(ts)), Point::Int(PointHlr::new(tx_id, &format!("{}03", name), 1, Status::Ok, Cot::ActCon, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value": -9223372036854775808,   "status": 0, "cot": "ActErr", "timestamp":"{}"}}"#,
                &format!("{}04", name), ts_str(ts)), Point::Int(PointHlr::new(tx_id, &format!("{}04", name), -9223372036854775808, Status::Ok, Cot::ActErr, ts))
            ),
            (
                format!(r#"{{"type": "Int",   "name": "{}", "value":  9223372036854775807,   "status": 0, "cot": "Req", "timestamp":"{}"}}"#,
                &format!("{}05", name), ts_str(ts)), Point::Int(PointHlr::new(tx_id, &format!("{}05", name),  9223372036854775807, Status::Ok, Cot::Req, ts))
            ),


            (
                format!(r#"{{"type": "Real", "name": "{}", "value":  0.0, "status": 0, "cot": "ReqCon", "timestamp":"{}"}}"#,
                &format!("{}06", name), ts_str(ts)), Point::Real(PointHlr::new(tx_id, &format!("{}06", name),  0.0, Status::Ok, Cot::ReqCon, ts))
            ),
            (
                format!(r#"{{"type": "Real", "name": "{}", "value":  1.0, "status": 0, "cot": "ReqCon", "timestamp":"{}"}}"#,
                &format!("{}06", name), ts_str(ts)), Point::Real(PointHlr::new(tx_id, &format!("{}06", name),  1.0, Status::Ok, Cot::ReqCon, ts))
            ),
            // (
            //     format!(r#"{{"type": "Real", "name": "{}", "value": -1.1, "status": 0, "cot": "ReqErr", "timestamp":"{}"}}"#,
            //     &format!("{}07", name), tsStr(ts)), PointType::Real(Point::new(txId, &format!("{}07", name), -1.1f32, Status::Ok, Cot::ReqErr, ts))
            // ),
            // (
            //     format!(r#"{{"type": "Real", "name": "{}", "value":  1.1, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
            //     &format!("{}08", name), tsStr(ts)), PointType::Real(Point::new(txId, &format!("{}08", name),  1.1f32, Status::Ok, Cot::Inf, ts))
            // ),
            // (
            //     format!(r#"{{"type": "Real", "name": "{}", "value": -3.4028235e38, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
            //     &format!("{}09", name), tsStr(ts)), PointType::Real(Point::new(txId, &format!("{}09", name), -f32::MAX, Status::Ok, Cot::Inf, ts))
            // ),
            // (
            //     format!(r#"{{"type": "Real", "name": "{}", "value":  3.4028235e38, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
            //     &format!("{}10", name), tsStr(ts)), PointType::Real(Point::new(txId, &format!("{}10", name),  f32::MAX, Status::Ok, Cot::Inf, ts))
            // ),


            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  0.0, "status": 0, "cot": "ReqCon", "timestamp":"{}"}}"#,
                &format!("{}06", name), ts_str(ts)), Point::Double(PointHlr::new(tx_id, &format!("{}06", name),  0.0, Status::Ok, Cot::ReqCon, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value": -1.1, "status": 0, "cot": "ReqErr", "timestamp":"{}"}}"#,
                &format!("{}07", name), ts_str(ts)), Point::Double(PointHlr::new(tx_id, &format!("{}07", name), -1.1, Status::Ok, Cot::ReqErr, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  1.1, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                &format!("{}08", name), ts_str(ts)), Point::Double(PointHlr::new(tx_id, &format!("{}08", name),  1.1, Status::Ok, Cot::Inf, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value": -1.7976931348623157e308, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                &format!("{}09", name), ts_str(ts)), Point::Double(PointHlr::new(tx_id, &format!("{}09", name), -1.7976931348623157e308, Status::Ok, Cot::Inf, ts))
            ),
            (
                format!(r#"{{"type": "Double", "name": "{}", "value":  1.7976931348623157e308, "status": 0, "cot": "Inf", "timestamp":"{}"}}"#,
                &format!("{}10", name), ts_str(ts)), Point::Double(PointHlr::new(tx_id, &format!("{}10", name),  1.7976931348623157e308, Status::Ok, Cot::Inf, ts))
            ),
            (
                format!(r#"{{"type": "String","name": "{}", "value": "~!@#$%^&*()_+`1234567890-=","status": 0,"cot": "Inf",  "timestamp":"{}"}}"#,
                &format!("{}11", name), ts_str(ts)), Point::String(PointHlr::new(tx_id, &format!("{}11", name), "~!@#$%^&*()_+`1234567890-=".to_string(), Status::Ok, Cot::Inf, ts))
            ),
        ];
        let (send, recv) = mpsc::channel();
        let mut jds_serialize = JdsEncodeMessage::new(
            "test",
            JdsSerialize::new("test", recv),
        );
        for (target, point) in test_data {
            send.send(point.clone()).unwrap();
            let result = jds_serialize.read().unwrap();
            let value: serde_json::Value = serde_json::from_str(&target).expect(&format!("Error parsing value: {:?}", target));
            let mut target = vec![];
            serde_json::to_writer(&mut target, &value).expect(&format!("Error parsing value: {:?}", value));
            target.push(4);
            assert!(result == target, "\n name: {} \nresult: {:?}\ntarget: {:?}", point.name(), String::from_utf8(result), String::from_utf8(target));
        }
    }
}
