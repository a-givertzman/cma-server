#[cfg(test)]

mod point {
    use std::{sync::Once, time::Duration};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use log::debug;
    use testing::stuff::max_test_duration::TestDuration;
    use crate::core_::{cot::cot::Cot, point::{point_hlr::PointHlr, point::Point}, status::status::Status, types::bool::Bool};
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
    #[test]
    fn serialize_json() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "serialize_json";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (r#"{"cot":"Inf","name":"/App/path/Point.Name.0","status":0,"timestamp":"2024-04-08T09:44:43.950510784+00:00","type":"Bool","value":1}"#,
                Point::Bool(PointHlr::new(
                    0,
                    &format!("/App/path/Point.Name.0"),
                    Bool(true),
                    Status::Ok,
                    Cot::Inf,
                    "2024-04-08T09:44:43.950510784+00:00".parse().unwrap(),
                ))
            ),
            (r#"{"cot":"Inf","name":"/App/path/Point.Name.1","status":0,"timestamp":"2024-04-08T09:44:44.450961534+00:00","type":"Int","value":1234567}"#,
                Point::Int(PointHlr::new(
                    0,
                    &format!("/App/path/Point.Name.1"),
                    1234567,
                    Status::Ok,
                    Cot::Inf,
                    "2024-04-08T09:44:44.450961534+00:00".parse().unwrap(),
                ))
            ),
            // (r#"{"cot":"Inf","name":"/App/path/Point.Name.2","status":0,"timestamp":"2024-04-08T09:44:43.550386216+00:00","type":"Real","value":123.12345}"#,
            //     PointType::Real(Point::new(
            //         0,
            //         &format!("/App/path/Point.Name.2"),
            //         123.12345,
            //         Status::Ok,
            //         Cot::Inf,
            //         "2024-04-08T09:44:43.550386216+00:00".parse().unwrap(),
            //     ))
            // ),
            (r#"{"cot":"Inf","name":"/App/path/Point.Name.3","status":0,"timestamp":"2024-04-08T09:44:43.550386216+00:00","type":"Double","value":123.12345}"#,
                Point::Double(PointHlr::new(
                    0,
                    &format!("/App/path/Point.Name.3"),
                    123.12345,
                    Status::Ok,
                    Cot::Inf,
                    "2024-04-08T09:44:43.550386216+00:00".parse().unwrap(),
                ))
            ),
        ];
        debug!("{} | Serialized Point: {:?}", self_id, lexical::parse::<f32, _>("1234.12345"));

        for (target, point) in test_data {
            let target: serde_json::Value = serde_json::from_str(target).unwrap();
            let result = serde_json::to_value(point).unwrap();
            debug!("{} | Serialized Point: {:#?}", self_id, result);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
    ///
    ///
    #[test]
    fn deserialize_json() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "deserialize_json";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            (r#"{"cot":"Inf","name":"/App/path/Point.Name.0","status":0,"timestamp":"2024-04-08T09:44:43.950510784+00:00","type":"Bool","value":1}"#,
                Point::Bool(PointHlr::new(
                    0,
                    &format!("/App/path/Point.Name.0"),
                    Bool(true),
                    Status::Ok,
                    Cot::Inf,
                    "2024-04-08T09:44:43.950510784+00:00".parse().unwrap(),
                ))
            ),
            (r#"{"cot":"Inf","name":"/App/path/Point.Name.1","status":0,"timestamp":"2024-04-08T09:44:44.450961534+00:00","type":"Int","value":1234567}"#,
                Point::Int(PointHlr::new(
                    0,
                    &format!("/App/path/Point.Name.1"),
                    1234567,
                    Status::Ok,
                    Cot::Inf,
                    "2024-04-08T09:44:44.450961534+00:00".parse().unwrap(),
                ))
            ),
            (r#"{"cot":"Inf","name":"/App/path/Point.Name.2","status":0,"timestamp":"2024-04-08T09:44:43.550386216+00:00","type":"Real","value":123.12345}"#,
                Point::Real(PointHlr::new(
                    0,
                    &format!("/App/path/Point.Name.2"),
                    123.12345,
                    Status::Ok,
                    Cot::Inf,
                    "2024-04-08T09:44:43.550386216+00:00".parse().unwrap(),
                ))
            ),
        ];
        for (point_json, target) in test_data {
            debug!("{} | input: {:#?}", self_id, point_json);
            let result: Point = serde_json::from_str(point_json).unwrap();
            // let target: serde_json::Value = json!(target)&target.name, &conf).unwrap();
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
}
