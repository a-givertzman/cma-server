#[cfg(test)]

mod profinet_client {
    use chrono::Utc;
    use std::{sync::{Arc, Once}, thread, time::Duration};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use sal_sync::{math::AproxEq, services::{conf::{ConfTree, ServicesConf}, entity::{Cot, Name, Point, PointHlr, PointTxId, Status}, MultiQueue, MultiQueueConf, Service, Services}};
    use crate::{conf::profinet_client_config::profinet_client_config::ProfinetClientConfig, services::profinet_client::profinet_client::ProfinetClient};
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
    #[ignore = "Integration test"]
    fn basic() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let self_id = "profinet_client_test";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let services = Arc::new(Services::new(self_id, ServicesConf::new(
            self_id, 
            ConfTree::new_root(serde_yaml::from_str(r#""#).unwrap()),
        )));
        let conf = r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to: queue
        "#.to_string();
        let conf = serde_yaml::from_str(&conf).unwrap();
        let mq_conf = MultiQueueConf::from_yaml(&self_name, &conf);
        let mq_service = Arc::new(MultiQueue::new(mq_conf, services.clone()));
        services.insert(mq_service.clone());
        let path = "./src/tests/unit/services/profinet_client/profinet_client.yaml";
        let conf = ProfinetClientConfig::read(self_name, path);
        log::debug!("config: {:?}", &conf);
        log::debug!("config points:");
        let client = Arc::new(ProfinetClient::new(conf, services.clone()));
        services.insert(client.clone());
        services.run().unwrap();
        mq_service.run().unwrap();
        client.run().unwrap();
        thread::sleep(Duration::from_millis(2000));
        let tx_id = PointTxId::from_str(self_id);
        let test_data = [
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Real(0.00101),
            Value::Real(0.00201),
            Value::Real(0.10201),
            Value::Real(9.10201),
            Value::Double(0.00101),
            Value::Double(0.00201),
            Value::Double(0.10201),
            Value::Double(9.10201),
        ];
        let send = mq_service.get_link("in-queue");
        let (_, recv) = mq_service.subscribe(self_id, &[]);
        for value in test_data {
            let point = match value {
                Value::Bool(value) => panic!("{} | Bool does not supported: {:?}", self_id, value),
                Value::Int(value) => {
                    Point::Int(PointHlr::new(tx_id, &Name::new("/Ied01/db999/", "Capacitor.Capacity").join(), value, Status::Ok, Cot::Act, Utc::now()))
                }
                Value::Real(value) => {
                    Point::Real(PointHlr::new(tx_id, &Name::new("/Ied01/db899/", "Drive.Speed").join(), value, Status::Ok, Cot::Act, Utc::now()))
                }
                Value::Double(value) => {
                    Point::Double(PointHlr::new(tx_id, &Name::new("/Ied01/db899/", "Drive.Speed").join(), value, Status::Ok, Cot::Act, Utc::now()))
                }
                Value::String(value) => panic!("{} | String does not supported: {:?}", self_id, value),
            };
            if let Err(err) = send.send(point.clone()) {
                log::warn!("{} | Send error: {:#?}", self_id, err);
            }
            match recv.recv_timeout(Duration::from_secs(3)) {
                Ok(received_point) => {
                    if received_point.cot() == Cot::Inf {
                        match received_point {
                            Point::Bool(value) => {
                                panic!("{} | Bool does not supported: {:?}", self_id, value)
                            }
                            Point::Int(received_point) => {
                                let result = received_point.value;
                                let target = point.as_int().value;
                                assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
                            }
                            Point::Real(received_point) => {
                                let result = received_point.value;
                                let target = point.as_real().value;
                                assert!(result.aprox_eq(target, 3), "\nresult: {:?}\ntarget: {:?}", result, target);
                            }
                            Point::Double(received_point) => {
                                let result = received_point.value;
                                let target = point.as_double().value;
                                assert!(result.aprox_eq(target, 3), "\nresult: {:?}\ntarget: {:?}", result, target);
                            }
                            Point::String(value) => {
                                panic!("{} | Bool does not supported: {:?}", self_id, value)
                            }
                        }
                    }
                }
                Err(err) => {
                    log::warn!("{} | Receive changed value error: {:#?}", self_id, err);
                }
            }
        }
        // thread::sleep(Duration::from_millis(3000));
        client.exit();
        mq_service.exit();
        services.exit();
        client.wait().unwrap();
        mq_service.wait().unwrap();
        services.wait().unwrap();
        test_duration.exit();
    }
}
