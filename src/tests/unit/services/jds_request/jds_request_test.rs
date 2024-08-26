#[cfg(test)]

mod jds_routes {
    use testing::{session::test_session::TestSession, stuff::{max_test_duration::TestDuration, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use std::{collections::HashMap, io::{Read, Write}, net::TcpStream, sync::{Arc, Mutex, Once, RwLock}, thread, time::Duration};
    use crate::{
        conf::{multi_queue_config::MultiQueueConfig, point_config::{name::Name, point_config::PointConfig}, tcp_server_config::TcpServerConfig},
        core_::{
            cot::cot::Cot, net::protocols::jds::{jds_define::JDS_END_OF_TRANSMISSION, jds_deserialize::JdsDeserialize, request_kind::RequestKind}, point::{point_hlr::PointHlr, point_tx_id::PointTxId, point_type::PointType}, status::status::Status
        },
        services::{multi_queue::multi_queue::MultiQueue, queue_name::QueueName, safe_lock::SafeLock, server::tcp_server::TcpServer, service::service::Service, services::Services, task::nested_function::reset_counter::AtomicReset},
        tests::unit::services::{multi_queue::mock_recv_service::{self, MockRecvService}, service::moc_service_points::MockServicePoints},
    };
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
    /// JDS request to the TcpServer
    fn request(self_id: &str, tcp_stream: &mut TcpStream, request: PointType) -> PointType {
        let cot = request.cot();
        let mut request = serde_json::to_vec(&request).unwrap();
        request.push(JDS_END_OF_TRANSMISSION);
        tcp_stream.write_all(&request).unwrap();
        let reply: &mut [u8; 4098] = &mut [0; 4098];
        tcp_stream.read(reply).unwrap();
        let reply: Vec<u8> = reply.iter().filter(|b| {b != &&JDS_END_OF_TRANSMISSION && b != &&0}).map(|b| *b).collect();
        // println!("{} | {:?} reply: {:?}", self_id, cot, reply);
        let reply = JdsDeserialize::deserialize(self_id, 0, reply.to_vec()).unwrap();
        println!("{} | {:?} reply: {:#?}", self_id, cot, reply);
        reply
    }
    ///
    /// Generets configurations of points
    fn point_configs(parent_name: &Name) -> Vec<PointConfig> {
        vec![
            PointConfig::from_yaml(parent_name, &serde_yaml::from_str(&format!(
                r#"{}:
                    type: String      # Bool / Int / Real / Double / String / Json
                    comment: Auth request, contains token / pass string"#,
                format!("Jds/{}", RequestKind::AUTH_SECRET),
            )).unwrap()),
            PointConfig::from_yaml(parent_name, &serde_yaml::from_str(&format!(
                r#"{}:
                    type: String      # Bool / Int / Real / Double / String / Json
                    comment: Auth request, contains SSH key"#,
                format!("Jds/{}", RequestKind::AUTH_SSH),
            )).unwrap()),
            PointConfig::from_yaml(parent_name, &serde_yaml::from_str(&format!(
                r#"{}:
                    type: String      # Bool / Int / Real / Double / String / Json
                    comment: Request all Ponts configurations"#,
                format!("Jds/{}", RequestKind::POINTS),
            )).unwrap()),
            PointConfig::from_yaml(parent_name, &serde_yaml::from_str(&format!(
                r#"{}:
                    type: String      # Bool / Int / Real / Double / String / Json
                    comment: Request to begin transmossion of all configured Points"#,
                format!("Jds/{}", RequestKind::SUBSCRIBE),
            )).unwrap()),
        ]
    }
    ///
    ///
    #[test]
    fn reject() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "jds_request_test";
        let self_name = Name::new(self_id, "");
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // Configuring Services
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        //
        // Configuring MultiQueue service
        let conf = serde_yaml::from_str(&format!(r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
                    - {}/MockRecvService0.in-queue
        "#, self_name)).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(&self_name, &conf);
        let mq_service = Arc::new(Mutex::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());
        //
        // Configuring TcpServer service
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_server_addr = format!("127.0.0.1:{}", tcp_port);
        let conf = format!(r#"
            service TcpServer:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: {}
                auth-secret:
                    pass: password      # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                in queue link:
                    max-length: 10000
                send-to: {}/MultiQueue.in-queue
        "#, tcp_server_addr, self_name);
        let conf = serde_yaml::from_str(&conf).unwrap();
        let conf = TcpServerConfig::from_yaml(self_name, &conf);
        let tcp_server = Arc::new(Mutex::new(TcpServer::new(conf, services.clone())));
        services.wlock(self_id).insert(tcp_server.clone());
        println!("{} | TcpServer - ready", self_id);
        //
        // Preparing test data
        let self_name = Name::new(self_id, "Jds");
        let test_data = [
            PointType::String(PointHlr::new(
                0,
                &Name::new(&self_name, "Auth.Secret").join(),
                r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
                Status::Ok,
                Cot::Inf,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                0,
                &Name::new(&self_name, "Auth.Secret").join(),
                r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
                Status::Ok,
                Cot::Act,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                0,
                &Name::new(&self_name, "Auth.Secret").join(),
                r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
                Status::Ok,
                Cot::ActCon,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                0,
                &Name::new(&self_name, "Auth.Secret").join(),
                r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
                Status::Ok,
                Cot::ActErr,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                0,
                &Name::new(&self_name, "Auth.Secret").join(),
                r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
                Status::Ok,
                Cot::ReqCon,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                0,
                &Name::new(&self_name, "Auth.Secret").join(),
                r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
                Status::Ok,
                Cot::ReqErr,
                chrono::offset::Utc::now(),
            )),
        ];
        let test_items_count = test_data.len();
        //
        // preparing MockServicePoints with the Vec<PontConfig>
        let service_points = Arc::new(Mutex::new(MockServicePoints::new(self_id, point_configs(&self_name))));
        services.wlock(self_id).insert(service_points);
        //
        // Configuring Receiver
        mock_recv_service::COUNT.reset(0);
        let receiver = Arc::new(Mutex::new(MockRecvService::new(self_id, "in-queue", Some(test_items_count))));
        services.wlock(self_id).insert(receiver.clone());
        println!("{} | MockRecvService - ready", self_id);
        println!("\n{} | All configurations - ok\n", self_id);
        //
        // Starting all services
        let services_handle = services.wlock(self_id).run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        let mq_service_handle = mq_service.lock().unwrap().run().unwrap();
        let tcp_server_handle = tcp_server.lock().unwrap().run().unwrap();
        println!("{} | All services - are executed", self_id);
        thread::sleep(Duration::from_millis(1000));
        //
        // Sending tcp test events / receiver must not receive anything before subscription activated
        println!("{} | Sending tcp test events - to be rejected (not authenticated)", self_id);
        let mut tcp_stream = TcpStream::connect(tcp_server_addr).unwrap();
        for request in test_data {
            let mut request = serde_json::to_vec(&request).unwrap();
            request.push(JDS_END_OF_TRANSMISSION);
            tcp_stream.write_all(&request).unwrap();
        }
        thread::sleep(Duration::from_millis(2000));
        receiver.lock().unwrap().exit();
        receiver_handle.wait().unwrap();
        let received = receiver.lock().unwrap().received();
        let result = received.lock().unwrap().len();
        assert!(result == 0, "All points must be rejected, but some of them passed: \nresult: {:?}\ntarget: {:?}", result, 0);
        receiver.lock().unwrap().exit();
        tcp_server.lock().unwrap().exit();
        mq_service.lock().unwrap().exit();
        services.rlock(self_id).exit();
        //
        // Waiting while all services being finished
        mq_service_handle.wait().unwrap();
        tcp_server_handle.wait().unwrap();
        services_handle.wait().unwrap();
        //
        // Reseting dureation timer
        test_duration.exit();
    }
    ///
    ///
    #[test]
    fn request_auth_secret() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "jds_request_test";
        let self_name = Name::new(self_id, "");
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // Configuring MultiQueue service
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        let conf = serde_yaml::from_str(&format!(r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
                    - {}/MockRecvService0.in-queue
        "#, self_name)).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(&self_name, &conf);
        let mq_service = Arc::new(Mutex::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());
        //
        // Configuring TcpServer service
        let secret = "123!@#qwe";
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_server_addr = format!("127.0.0.1:{}", tcp_port);
        let conf = format!(r#"
            service TcpServer:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: {}
                auth-secret:
                    pass: {}      # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                in queue link:
                    max-length: 10000
                send-to: {}/MultiQueue.in-queue
        "#, tcp_server_addr, secret, self_name);
        let conf = serde_yaml::from_str(&conf).unwrap();
        let conf = TcpServerConfig::from_yaml(self_name, &conf);
        let tcp_server = Arc::new(Mutex::new(TcpServer::new(conf, services.clone())));
        services.wlock(self_id).insert(tcp_server.clone());
        println!("{} | TcpServer - ready", self_id);
        //
        // Preparing test data
        let self_name = Name::new(self_id, "Jds");
        //
        // preparing MockServicePoints with the Vec<PontConfig>
        let service_points = Arc::new(Mutex::new(MockServicePoints::new(self_id, point_configs(&self_name))));
        services.wlock(self_id).insert(service_points);
        //
        // Configuring Receiver
        mock_recv_service::COUNT.reset(0);
        let receiver = Arc::new(Mutex::new(MockRecvService::new(self_id, "in-queue", None)));
        services.wlock(self_id).insert(receiver.clone());
        println!("{} | MockRecvService - ready", self_id);
        println!("\n{} | All configurations - ok\n", self_id);
        //
        // Starting all services
        let services_handle = services.wlock(self_id).run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        let mq_service_handle = mq_service.lock().unwrap().run().unwrap();
        let tcp_server_handle = tcp_server.lock().unwrap().run().unwrap();
        println!("{} | All services - are executed", self_id);
        thread::sleep(Duration::from_millis(1000));
        //
        // Sending tcp test events / receiver must not receive anything before subscription activated
        println!("{} | Sending tcp test events - to be rejected (not authenticated)", self_id);
        let mut tcp_stream = TcpStream::connect(tcp_server_addr).unwrap();
        let auth_req = PointType::String(PointHlr::new(
            0,
            &Name::new(&self_name, "Auth.Secret").join(),
            secret.into(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        ));
        let result = request(self_id, &mut tcp_stream, auth_req);
        let target = PointType::String(PointHlr::new(0, &Name::new(&self_name, "Auth.Secret").join(), "Authentication successful".to_owned(), Status::Ok, Cot::ReqCon, chrono::offset::Utc::now()));
        assert!(result.name() == target.name(), "\nresult: {:?}\ntarget: {:?}", result.name(), target.name());
        assert!(result.value() == target.value(), "\nresult: {:?}\ntarget: {:?}", result.value(), target.value());
        assert!(result.status() == target.status(), "\nresult: {:?}\ntarget: {:?}", result.status(), target.status());
        assert!(result.cot() == target.cot(), "\nresult: {:?}\ntarget: {:?}", result.cot(), target.cot());
        println!("{} | Auth.Secret request successful!\n", self_id);
        //
        // Stopping all services
        receiver.lock().unwrap().exit();
        tcp_server.lock().unwrap().exit();
        mq_service.lock().unwrap().exit();
        services.rlock(self_id).exit();
        //
        // Waiting while all services being finished
        receiver_handle.wait().unwrap();
        mq_service_handle.wait().unwrap();
        tcp_server_handle.wait().unwrap();
        services_handle.wait().unwrap();
        //
        // Reseting dureation timer
        test_duration.exit();
    }
    ///
    ///
    #[test]
    fn request_points() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "jds_request_test";
        let self_name = Name::new(self_id, "");
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // Configuring MultiQueue service
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        let conf = serde_yaml::from_str(&format!(r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
                    - {}/MockRecvService0.in-queue
        "#, self_name)).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(&self_name, &conf);
        let mq_service = Arc::new(Mutex::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());
        //
        // Configuring TcpServer service
        let secret = "123!@#qwe";
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_server_addr = format!("127.0.0.1:{}", tcp_port);
        let conf = format!(r#"
            service TcpServer:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: {}
                auth-secret:
                    pass: {}      # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                in queue link:
                    max-length: 10000
                send-to: {}/MultiQueue.in-queue
        "#, tcp_server_addr, secret, self_name);
        let conf = serde_yaml::from_str(&conf).unwrap();
        let conf = TcpServerConfig::from_yaml(self_name, &conf);
        let tcp_server = Arc::new(Mutex::new(TcpServer::new(conf, services.clone())));
        services.wlock(self_id).insert(tcp_server.clone());
        println!("{} | TcpServer - ready", self_id);
        //
        // Preparing test data
        let tx_id = PointTxId::from_str(self_id);
        let self_name = Name::new(self_id, "Jds");
        let test_data = [
            PointType::String(PointHlr::new(
                tx_id,
                &Name::new(&self_name, "Auth.Secret").join(),
                r#"{
                    \"secret\": \"Auth.Secret\"
                }"#.to_string(),
                Status::Ok,
                Cot::Req,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                tx_id,
                &Name::new(&self_name, "Auth.Ssh").join(),
                r#"{
                    \"ssh\": \"Auth.Ssh\"
                }"#.to_string(),
                Status::Ok,
                Cot::Req,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                tx_id,
                &Name::new(&self_name, "Points").join(),
                r#"{
                    \"points\": []
                }"#.to_string(),
                Status::Ok,
                Cot::Req,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                tx_id,
                &Name::new(&self_name, "Subscribe").join(),
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
        let service_points = Arc::new(Mutex::new(MockServicePoints::new(self_id, point_configs(&self_name))));
        services.wlock(self_id).insert(service_points);
        //
        // Configuring Receiver
        mock_recv_service::COUNT.reset(0);
        let receiver = Arc::new(Mutex::new(MockRecvService::new(self_id, "in-queue", Some(test_items_count * 2))));
        services.wlock(self_id).insert(receiver.clone());
        println!("{} | MockRecvService - ready", self_id);
        println!("\n{} | All configurations - ok\n", self_id);
        //
        // Starting all services
        let services_handle = services.wlock(self_id).run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        let mq_service_handle = mq_service.lock().unwrap().run().unwrap();
        let tcp_server_handle = tcp_server.lock().unwrap().run().unwrap();
        println!("{} | All services - are executed", self_id);
        thread::sleep(Duration::from_millis(1000));
        //
        // Authenticating
        println!("{} | Sending tcp test events - to be rejected (not authenticated)", self_id);
        let mut tcp_stream = TcpStream::connect(tcp_server_addr).unwrap();
        let auth_req = PointType::String(PointHlr::new(
            0,
            &Name::new(&self_name, "Auth.Secret").join(),
            secret.into(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        ));
        let result = request(self_id, &mut tcp_stream, auth_req);
        assert!(result.cot() == Cot::ReqCon, "\nresult: {:?}\ntarget: {:?}", result.cot(), Cot::ReqCon);
        //
        // Sending Points request
        let subscribe_req = PointType::String(PointHlr::new(
            0,
            &Name::new(&self_name, "Points").join(),
            "".to_string(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        ));
        let result = request(self_id, &mut tcp_stream, subscribe_req);
        let target = PointType::String(PointHlr::new(0, &Name::new(&self_name, "Points").join(), "".to_owned(), Status::Ok, Cot::ReqCon, chrono::offset::Utc::now()));
        // assert!(result.name() == target.name(), "\nresult: {:?}\ntarget: {:?}", result.name(), target.name());
        // assert!(result.value() == target.value(), "\nresult: {:?}\ntarget: {:?}", result.value(), target.value());
        let points: HashMap<String, serde_json::Value> = serde_json::from_str(&result.value().as_string()).unwrap();
        let points: HashMap<_, PointConfig> = points.iter().map(|(name, value)| {
            (name, PointConfig::from_json(name, value).unwrap())
        }).collect();
        println!("{} | Points request reply: {:#?}", self_id, points);
        for target in point_configs(&self_name) {
            match points.get(&target.name) {
                Some(result) => {
                    assert!(result.name == target.name, "\nresult: {:?}\ntarget: {:?}", result.name, target.name);
                    assert!(result.type_ == target.type_, "\nresult: {:?}\ntarget: {:?}", result.type_, target.type_);
                    assert!(result.history == target.history, "\nresult: {:?}\ntarget: {:?}", result.history, target.history);
                    assert!(result.alarm == target.alarm, "\nresult: {:?}\ntarget: {:?}", result.alarm, target.alarm);
                    assert!(result.address == target.address, "\nresult: {:?}\ntarget: {:?}", result.address, target.address);
                }
                None => {
                    panic!("PointConfig '{}' - not found in the Points request reply", target.name)
                }
            }
        }
        assert!(result.status() == target.status(), "\nresult: {:?}\ntarget: {:?}", result.status(), target.status());
        assert!(result.cot() == target.cot(), "\nresult: {:?}\ntarget: {:?}", result.cot(), target.cot());
        println!("{} | Points request successful!\n", self_id);
        //
        // Stopping all services
        receiver.lock().unwrap().exit();
        tcp_server.lock().unwrap().exit();
        mq_service.lock().unwrap().exit();
        services.rlock(self_id).exit();
        //
        // Waiting while all services being finished
        receiver_handle.wait().unwrap();
        mq_service_handle.wait().unwrap();
        tcp_server_handle.wait().unwrap();
        services_handle.wait().unwrap();
        //
        // Reseting dureation timer
        test_duration.exit();
    }
    ///
    ///
    #[test]
    #[ignore = "To be implementes..."]
    fn auth_ssh() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "jds_request_test";
        let self_name = Name::new(self_id, "");
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        //
        // Configuring MultiQueue service
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        let conf = serde_yaml::from_str(&format!(r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
                    - {}/MockRecvService0.in-queue
        "#, self_id)).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(&self_name, &conf);
        let mq_service = Arc::new(Mutex::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());
        //
        // Configuring TcpServer service
        let tcp_port = TestSession::free_tcp_port_str();
        let tcp_addr = format!("127.0.0.1:{}", tcp_port);
        let conf = format!(r#"
            service TcpServer:
                cycle: 1 ms
                reconnect: 1 s  # default 3 s
                address: {}
                auth: none      # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
                in queue link:
                    max-length: 10000
                send-to: {}/MultiQueue.in-queue
        "#, tcp_addr, self_id);
        let conf = serde_yaml::from_str(&conf).unwrap();
        let conf = TcpServerConfig::from_yaml(self_name, &conf);
        let tcp_server = Arc::new(Mutex::new(TcpServer::new(conf, services.clone())));
        services.wlock(self_id).insert(tcp_server.clone());
        println!("{} | TcpServer - ready", self_id);
        //
        // Preparing test data
        let tx_id = PointTxId::from_str(self_id);
        let parent = self_id;
        let test_data = [
            PointType::String(PointHlr::new(
                tx_id,
                &Name::new(parent, "JdsService/Auth.Secret").join(),
                r#"{
                    \"secret\": \"Auth.Secret\"
                }"#.to_string(),
                Status::Ok,
                Cot::Req,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                tx_id,
                &Name::new(parent, "JdsService/Auth.Ssh").join(),
                r#"{
                    \"ssh\": \"Auth.Ssh\"
                }"#.to_string(),
                Status::Ok,
                Cot::Req,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                tx_id,
                &Name::new(parent, "JdsService/Points").join(),
                r#"{
                    \"points\": []
                }"#.to_string(),
                Status::Ok,
                Cot::Req,
                chrono::offset::Utc::now(),
            )),
            PointType::String(PointHlr::new(
                tx_id,
                &Name::new(parent, "JdsService/Subcribe").join(),
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
        services.wlock(self_id).insert(receiver.clone());
        println!("{} | MockRecvService - ready", self_id);
        //
        // Starting all services
        let services_handle = services.wlock(self_id).run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        let mq_service_handle = mq_service.lock().unwrap().run().unwrap();
        let jds_service_handle = tcp_server.lock().unwrap().run().unwrap();
        println!("{} | All services - are executed", self_id);
        thread::sleep(Duration::from_millis(200));
        //
        // Sending test events
        println!("{} | Try to get send from MultiQueue...", self_id);
        let send = services.wlock(self_id).get_link(&QueueName::new("MultiQueue.in-queue")).unwrap();
        println!("{} | Try to get send from MultiQueue - ok", self_id);
        let mut sent = 0;
        for point in test_data {
            match send.send(point.clone()) {
                Ok(_) => {
                    sent += 1;
                    println!("{} | \t sent: {:?}", self_id, point);
                }
                Err(err) => {
                    panic!("{} | Send error: {:?}", self_id, err)
                }
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
        tcp_server.lock().unwrap().exit();
        mq_service.lock().unwrap().exit();
        services.rlock(self_id).exit();
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
                }
                Cot::ReqErr => {
                    reply_errors += 1;
                    println!("{} | Received ReqErr reply: {:?}", self_id, point);
                }
                // Cot::Read => todo!(),
                // Cot::Write => todo!(),
                // Cot::All => todo!(),
                _ => {
                    println!("{} | Received unknown point: {:?}", self_id, point);
                }
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
        services_handle.wait().unwrap();
        //
        // Reseting dureation timer
        test_duration.exit();
    }
}
