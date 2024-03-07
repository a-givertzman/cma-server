#[cfg(test)]

mod jds_service {
    use log::debug;
    use testing::stuff::{max_test_duration::TestDuration, wait::WaitTread};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use std::{sync::{Arc, Mutex, Once}, thread, time::Duration};
    use crate::{
        conf::{jds_service_config::jds_service_config::JdsServiceConfig, multi_queue_config::MultiQueueConfig, point_config::{point_config::PointConfig, point_name::PointName}}, 
        core_::{cot::cot::Cot, point::{point::Point, point_tx_id::PointTxId, point_type::PointType}, status::status::Status}, 
        services::{jds_service::{jds_service::JdsService, request_kind::RequestKind}, multi_queue::multi_queue::MultiQueue, service::service::Service, services::Services}, 
        tests::unit::services::{multi_queue::mock_recv_service::MockRecvService, service::moc_service_points::MockServicePoints},
    }; 
    
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    // use super::*;
    
    static INIT: Once = Once::new();
    
    ///
    /// once called initialisation
    fn init_once() {
        INIT.call_once(|| {
                // implement your initialisation code to be called only once for current test file
            }
        )
    }
    
    
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {
    
    }
    ///
    fn point_configs(parent: &str) -> Vec<PointConfig> {
        vec![
            PointConfig::from_yaml(parent, &serde_yaml::from_str(&format!(
                r#"{}:
                    type: String      # Bool / Int / Float / String / Json
                    comment: Auth request, contains token / pass string"#, 
                RequestKind::AUTH_SECRET
            )).unwrap()),
            PointConfig::from_yaml(parent, &serde_yaml::from_str(&format!(
                r#"{}:
                    type: String      # Bool / Int / Float / String / Json
                    comment: Auth request, contains SSH key"#, 
                RequestKind::AUTH_SSH,
            )).unwrap()),
            PointConfig::from_yaml(parent, &serde_yaml::from_str(&format!(
                r#"{}:
                    type: String      # Bool / Int / Float / String / Json
                    comment: Request all Ponts configurations"#, 
                RequestKind::POINTS,
            )).unwrap()),
            PointConfig::from_yaml(parent, &serde_yaml::from_str(&format!(
                r#"{}:
                    type: String      # Bool / Int / Float / String / Json
                    comment: Request to begin transmossion of all configured Points"#, 
                RequestKind::SUBSCRIBE,
            )).unwrap()),
        ]
    }
    ///
    #[test]
    fn auth_secret() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!("");
        let self_id = "test JdsService";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // Configuring MultiQueue service 
        let services = Arc::new(Mutex::new(Services::new(self_id)));
        let conf = serde_yaml::from_str(r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                out queue: 
                    - MockRecvService.in-queue
        "#).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(&conf);
        let mq_service = Arc::new(Mutex::new(MultiQueue::new(self_id, mq_conf, services.clone())));
        services.lock().unwrap().insert("MultiQueue", mq_service.clone());
        //
        // Configuring JdsService service 
        let conf = serde_yaml::from_str(r#"
            service JdsService JdsService:
                in queue in-queue:
                    max-length: 10000
                out queue: MultiQueue.in-queue
        "#).unwrap();
        let conf = JdsServiceConfig::from_yaml(&conf);
        debug!("config: {:?}", &conf);
        let jds_service = Arc::new(Mutex::new(JdsService::new(self_id, conf, services.clone())));
        services.lock().unwrap().insert("JdsService", jds_service.clone());
        println!("{} | JdsService - ready", self_id);
        //
        // Preparing test data
        let tx_id = PointTxId::fromStr(self_id);
        let parent = self_id;
        let test_data = [
            PointType::String(Point::new(
                tx_id, 
                &PointName::new(parent, "JdsService/Auth.Secret").full(),
                r#"{
                    \"secret\": \"Auth.Secret\"
                }"#.to_string(), 
                Status::Ok, 
                Cot::Req, 
                chrono::offset::Utc::now(),
            )),
            PointType::String(Point::new(
                tx_id, 
                &PointName::new(parent, "JdsService/Auth.Ssh").full(),
                r#"{
                    \"ssh\": \"Auth.Ssh\"
                }"#.to_string(), 
                Status::Ok, 
                Cot::Req, 
                chrono::offset::Utc::now(),
            )),
            PointType::String(Point::new(
                tx_id, 
                &PointName::new(parent, "JdsService/Points").full(),
                r#"{
                    \"points\": []
                }"#.to_string(), 
                Status::Ok, 
                Cot::Req, 
                chrono::offset::Utc::now(),
            )),
            PointType::String(Point::new(
                tx_id, 
                &PointName::new(parent, "JdsService/Subscribe").full(),
                r#"{
                    \"points\": []
                }"#.to_string(), 
                Status::Ok, 
                Cot::Req, 
                chrono::offset::Utc::now(),
            )),
        ];
        let test_items_count = test_data.len();
        //
        // preparing MockServicePoints with the Vec<PontConfig>
        let service_points = Arc::new(Mutex::new(MockServicePoints::new(self_id, point_configs(self_id))));
        services.lock().unwrap().insert("MockServicePoints", service_points);
        //
        // Configuring Receiver
        let receiver = Arc::new(Mutex::new(MockRecvService::new(self_id, "in-queue", Some(test_items_count * 2))));
        services.lock().unwrap().insert("MockRecvService", receiver.clone());
        println!("{} | MockRecvService - ready", self_id);
        //
        // Starting all services
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        let mq_service_handle = mq_service.lock().unwrap().run().unwrap();
        let jds_service_handle = jds_service.lock().unwrap().run().unwrap();
        println!("{} | All services - are executed", self_id);
        thread::sleep(Duration::from_millis(200));
        //
        // Sending test events
        println!("{} | Try to get send from MultiQueue...", self_id);
        let send = services.lock().unwrap().get_link("MultiQueue.in-queue");
        println!("{} | Try to get send from MultiQueue - ok", self_id);
        let mut sent = 0;
        for point in test_data {
            match send.send(point.clone()) {
                Ok(_) => {
                    sent += 1;
                    println!("{} | \t sent: {:?}", self_id, point);
                },
                Err(err) => {
                    panic!("{} | Send error: {:?}", self_id, err)
                },
            }
        }
        println!("{} | Total sent: {}", self_id, sent);
        //
        // Waiting while all events being received
        receiver_handle.wait().unwrap();
        thread::sleep(Duration::from_millis(1000));
        //
        // Stopping all services
        receiver.lock().unwrap().exit();
        jds_service.lock().unwrap().exit();
        mq_service.lock().unwrap().exit();
        //
        // Verivications
        let received = receiver.lock().unwrap().received();
        let received_len = received.lock().unwrap().len();
        let result = received_len;
        let target = test_items_count * 2;
        println!("{} | Total received: {}", self_id, received_len);
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        //
        // Verifing JdsService replies
        let mut replies = 0;
        let mut reply_errors = 0;
        for point in received.lock().unwrap().iter() {
            match point.cot() {
                // Cot::Inf => todo!(),
                // Cot::Act => todo!(),
                // Cot::ActCon => todo!(),
                // Cot::ActErr => todo!(),
                // Cot::Req => todo!(),
                Cot::ReqCon => {
                    replies += 1;
                    println!("{} | Received ReqCon reply: {:?}", self_id, point);
                    if point.name() == PointName::new(parent, "JdsService/Points").full() {
                        let result: Vec<PointConfig> = serde_json::from_str(point.value().as_string().as_str()).unwrap();
                        let target = point_configs(self_id);
                        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
                    }
                },
                Cot::ReqErr => {
                    reply_errors += 1;
                    println!("{} | Received ReqErr reply: {:?}", self_id, point);
                },
                // Cot::Read => todo!(),
                // Cot::Write => todo!(),
                // Cot::All => todo!(),
                _ => {
                    println!("{} | Received unknown point: {:?}", self_id, point);
                },
            }
        }
        let result = replies;
        let target = test_items_count;
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        let result = reply_errors;
        let target = 0;
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        //
        // Waiting while all services being finished
        mq_service_handle.wait().unwrap();
        jds_service_handle.wait().unwrap();
        //
        // Reseting dureation timer
        test_duration.exit();
    }

    // #[test]
    fn auth_ssh() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!("");
        let self_id = "test JdsService";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        //
        // Configuring MultiQueue service 
        let services = Arc::new(Mutex::new(Services::new(self_id)));
        let conf = serde_yaml::from_str(r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                out queue: 
                    - MockRecvService.in-queue
        "#).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(&conf);
        let mq_service = Arc::new(Mutex::new(MultiQueue::new(self_id, mq_conf, services.clone())));
        services.lock().unwrap().insert("MultiQueue", mq_service.clone());
        //
        // Configuring JdsService service 
        let conf = serde_yaml::from_str(r#"
            service JdsService JdsService:
                in queue in-queue:
                    max-length: 10000
                out queue: MultiQueue.in-queue
        "#).unwrap();
        let conf = JdsServiceConfig::from_yaml(&conf);
        debug!("config: {:?}", &conf);
        let jds_service = Arc::new(Mutex::new(JdsService::new(self_id, conf, services.clone())));
        services.lock().unwrap().insert("JdsService", jds_service.clone());
        println!("{} | JdsService - ready", self_id);
        //
        // Preparing test data
        let tx_id = PointTxId::fromStr(self_id);
        let parent = self_id;
        let test_data = [
            PointType::String(Point::new(
                tx_id, 
                &PointName::new(parent, "JdsService/Auth.Secret").full(),
                r#"{
                    \"secret\": \"Auth.Secret\"
                }"#.to_string(), 
                Status::Ok, 
                Cot::Req, 
                chrono::offset::Utc::now(),
            )),
            PointType::String(Point::new(
                tx_id, 
                &PointName::new(parent, "JdsService/Auth.Ssh").full(),
                r#"{
                    \"ssh\": \"Auth.Ssh\"
                }"#.to_string(), 
                Status::Ok, 
                Cot::Req, 
                chrono::offset::Utc::now(),
            )),
            PointType::String(Point::new(
                tx_id, 
                &PointName::new(parent, "JdsService/Points").full(),
                r#"{
                    \"points\": []
                }"#.to_string(), 
                Status::Ok, 
                Cot::Req, 
                chrono::offset::Utc::now(),
            )),
            PointType::String(Point::new(
                tx_id, 
                &PointName::new(parent, "JdsService/Subcribe").full(),
                r#"{
                    \"points\": []
                }"#.to_string(), 
                Status::Ok, 
                Cot::Req, 
                chrono::offset::Utc::now(),
            )),
        ];
        let test_items_count = test_data.len();
        //
        // Configuring Receiver
        let receiver = Arc::new(Mutex::new(MockRecvService::new(self_id, "in-queue", Some(test_items_count * 2))));
        services.lock().unwrap().insert("MockRecvService", receiver.clone());
        println!("{} | MockRecvService - ready", self_id);
        //
        // Starting all services
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        let mq_service_handle = mq_service.lock().unwrap().run().unwrap();
        let jds_service_handle = jds_service.lock().unwrap().run().unwrap();
        println!("{} | All services - are executed", self_id);
        thread::sleep(Duration::from_millis(200));
        //
        // Sending test events
        println!("{} | Try to get send from MultiQueue...", self_id);
        let send = services.lock().unwrap().get_link("MultiQueue.in-queue");
        println!("{} | Try to get send from MultiQueue - ok", self_id);
        let mut sent = 0;
        for point in test_data {
            match send.send(point.clone()) {
                Ok(_) => {
                    sent += 1;
                    println!("{} | \t sent: {:?}", self_id, point);
                },
                Err(err) => {
                    panic!("{} | Send error: {:?}", self_id, err)
                },
            }
        }
        println!("{} | Total sent: {}", self_id, sent);
        //
        // Waiting while all events being received
        receiver_handle.wait().unwrap();
        thread::sleep(Duration::from_millis(800));
        //
        // Stopping all services
        receiver.lock().unwrap().exit();
        jds_service.lock().unwrap().exit();
        mq_service.lock().unwrap().exit();
        //
        // Verivications
        let received = receiver.lock().unwrap().received();
        let received_len = received.lock().unwrap().len();
        let result = received_len;
        let target = test_items_count * 2;
        println!("{} | Total received: {}", self_id, received_len);
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        //
        // Verifing JdsService replies
        let mut replies = 0;
        let mut reply_errors = 0;
        for point in received.lock().unwrap().iter() {
            match point.cot() {
                // Cot::Inf => todo!(),
                // Cot::Act => todo!(),
                // Cot::ActCon => todo!(),
                // Cot::ActErr => todo!(),
                // Cot::Req => todo!(),
                Cot::ReqCon => {
                    replies += 1;
                    println!("{} | Received ReqCon reply: {:?}", self_id, point);
                },
                Cot::ReqErr => {
                    reply_errors += 1;
                    println!("{} | Received ReqErr reply: {:?}", self_id, point);
                },
                // Cot::Read => todo!(),
                // Cot::Write => todo!(),
                // Cot::All => todo!(),
                _ => {
                    println!("{} | Received unknown point: {:?}", self_id, point);
                },
            }
        }
        let result = replies;
        let target = test_items_count;
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        let result = reply_errors;
        let target = 0;
        assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        //
        // Waiting while all services being finished
        mq_service_handle.wait().unwrap();
        jds_service_handle.wait().unwrap();
        //
        // Reseting dureation timer
        test_duration.exit();
    }
}
