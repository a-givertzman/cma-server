#[cfg(test)]

use sal_sync::{services::{
    conf::{ConfTree, ServicesConf}, entity::{
        Cot, Name, Object,
        Point, PointConf, PointHlr, PointTxId,
        Status,
    }, LinkName, MultiQueue, MultiQueueConf, Service, Services
}, thread_pool::ThreadPool};
use testing::{session::test_session::TestSession, stuff::max_test_duration::TestDuration};
use debugging::session::debug_session::{DebugSession, LogLevel};
use std::{collections::HashMap, io::{Read, Write}, net::TcpStream, str::FromStr, sync::{Arc, Once}, thread, time::Duration};
use crate::{
    domain::{net::protocols::jds::{jds_define::JDS_END_OF_TRANSMISSION, jds_deserialize::JdsDeserialize, request_kind::RequestKind}, testing::{RecvService, RecvServiceConf}},
    services::server::{TcpServer, TcpServerConf},
    tests::unit::services::jds_request::mock_service_points::MockServicePoints,
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
fn request(self_id: &str, tcp_stream: &mut TcpStream, request: Point) -> Point {
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
fn point_configs(parent_name: &Name) -> Vec<PointConf> {
    vec![
        PointConf::from_yaml(parent_name, &serde_yaml::from_str(&format!(
            r#"{}:
                type: String      # Bool / Int / Real / Double / String / Json
                comment: Auth request, contains token / pass string"#,
            format!("Jds/{}", RequestKind::AUTH_SECRET),
        )).unwrap()),
        PointConf::from_yaml(parent_name, &serde_yaml::from_str(&format!(
            r#"{}:
                type: String      # Bool / Int / Real / Double / String / Json
                comment: Auth request, contains SSH key"#,
            format!("Jds/{}", RequestKind::AUTH_SSH),
        )).unwrap()),
        PointConf::from_yaml(parent_name, &serde_yaml::from_str(&format!(
            r#"{}:
                type: String      # Bool / Int / Real / Double / String / Json
                comment: Request all Ponts configurations"#,
            format!("Jds/{}", RequestKind::POINTS),
        )).unwrap()),
        PointConf::from_yaml(parent_name, &serde_yaml::from_str(&format!(
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
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "jds_request_test";
    println!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
    test_duration.run().unwrap();
    //
    // Preparing test data
    let self_name = Name::new(dbg, "Jds");
    let test_data = [
        Point::String(PointHlr::new(
            0,
            &Name::new(&self_name, "Auth.Secret").join(),
            r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
            Status::Ok,
            Cot::Inf,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
            0,
            &Name::new(&self_name, "Auth.Secret").join(),
            r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
            Status::Ok,
            Cot::Act,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
            0,
            &Name::new(&self_name, "Auth.Secret").join(),
            r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
            Status::Ok,
            Cot::ActCon,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
            0,
            &Name::new(&self_name, "Auth.Secret").join(),
            r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
            Status::Ok,
            Cot::ActErr,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
            0,
            &Name::new(&self_name, "Auth.Secret").join(),
            r#"{\"reply\": \"Auth.Ssh Reply\"}"#.to_string(),
            Status::Ok,
            Cot::ReqCon,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
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
    // Configuring Services
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
                path: assets/testing/retain/
                point:
                    path: point/id.json
        "#).unwrap()),
    ), Some(tp.scheduler())));
    //
    // Configuring Receiver
    let conf = serde_yaml::from_str(&format!(r#"
        service RecvService RecvService:
            recv-limit: {test_items_count}
            in queue in-queue:
                max-length: 10000
    "#)).unwrap();
    let receiver = Arc::new(RecvService::new(
        dbg,
        RecvServiceConf::from_yaml(dbg, &conf),
        tp.scheduler(),
    ));
    services.insert(receiver.clone());
    println!("{} | RecvService - ready", dbg);
    //
    // Configuring MultiQueue service
    let conf = serde_yaml::from_str(&format!(r#"
        service MultiQueue:
            in queue in-queue:
                max-length: 10000
            send-to:
                - {}.in-queue
    "#, receiver.name().join())).unwrap();
    let mq_conf = MultiQueueConf::from_yaml(&self_name, &conf);
    let mq_service = Arc::new(MultiQueue::new(mq_conf, services.clone(), Some(tp.scheduler())));
    services.insert(mq_service.clone());
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
    let conf = TcpServerConf::from_yaml(&self_name, &conf);
    let tcp_server = Arc::new(TcpServer::new(conf, services.clone(), tp.scheduler()));
    services.insert(tcp_server.clone());
    println!("{} | TcpServer - ready", dbg);
    //
    // preparing MockServicePoints with the Vec<PontConfig>
    let service_points = Arc::new(MockServicePoints::new(dbg, point_configs(&self_name)));
    services.insert(service_points);
    println!("\n{} | All configurations - ok\n", dbg);
    //
    // Starting all services
    services.run().unwrap();
    receiver.run().unwrap();
    mq_service.run().unwrap();
    tcp_server.run().unwrap();
    println!("{} | All services - are executed", dbg);
    thread::sleep(Duration::from_millis(1000));
    //
    // Sending tcp test events / receiver must not receive anything before subscription activated
    println!("{} | Sending tcp test events - to be rejected (not authenticated)", dbg);
    let mut tcp_stream = TcpStream::connect(tcp_server_addr).unwrap();
    for request in test_data {
        let mut request = serde_json::to_vec(&request).unwrap();
        request.push(JDS_END_OF_TRANSMISSION);
        tcp_stream.write_all(&request).unwrap();
    }
    thread::sleep(Duration::from_millis(2000));
    receiver.exit();
    receiver.wait().unwrap();
    let received = receiver.received();
    let result = received.write().len();
    assert!(result == 0, "All points must be rejected, but some of them passed: \nresult: {:?}\ntarget: {:?}", result, 0);
    receiver.exit();
    tcp_server.exit();
    mq_service.exit();
    services.exit();
    //
    // Waiting while all services being finished
    mq_service.wait().unwrap();
    tcp_server.wait().unwrap();
    services.wait().unwrap();
    //
    // Reseting dureation timer
    test_duration.exit();
}
///
///
#[test]
fn request_auth_secret() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "jds_request_test";
    let self_name = Name::new(dbg, "");
    println!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
    test_duration.run().unwrap();
    //
    // Configuring MultiQueue service
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
                path: assets/testing/retain/
                point:
                    path: point/id.json
        "#).unwrap()),
    ), Some(tp.scheduler())));
    //
    // Configuring Receiver
    let conf = serde_yaml::from_str(&format!(r#"
        service RecvService RecvService:
            in queue in-queue:
                max-length: 10000
    "#)).unwrap();
    let receiver = Arc::new(RecvService::new(
        dbg,
        RecvServiceConf::from_yaml(dbg, &conf),
        tp.scheduler(),
    ));
    services.insert(receiver.clone());
    println!("{} | RecvService - ready", dbg);
    let conf = serde_yaml::from_str(&format!(r#"
        service MultiQueue:
            in queue in-queue:
                max-length: 10000
            send-to:
                - {}.in-queue
    "#, receiver.name().join())).unwrap();
    let mq_conf = MultiQueueConf::from_yaml(&self_name, &conf);
    let mq_service = Arc::new(MultiQueue::new(mq_conf, services.clone(), Some(tp.scheduler())));
    services.insert(mq_service.clone());
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
                pass: '{}'      # auth: none / auth-secret: pass: ... / auth-ssh: path: ...
            in queue link:
                max-length: 10000
            send-to: {}/MultiQueue.in-queue
    "#, tcp_server_addr, secret, self_name);
    let conf = serde_yaml::from_str(&conf).unwrap();
    let conf = TcpServerConf::from_yaml(self_name, &conf);
    let tcp_server = Arc::new(TcpServer::new(conf, services.clone(), tp.scheduler()));
    services.insert(tcp_server.clone());
    println!("{} | TcpServer - ready", dbg);
    //
    // Preparing test data
    let self_name = Name::new(dbg, "Jds");
    //
    // preparing MockServicePoints with the Vec<PontConfig>
    let service_points = Arc::new(MockServicePoints::new(dbg, point_configs(&self_name)));
    services.insert(service_points);
    println!("\n{} | All configurations - ok\n", dbg);
    //
    // Starting all services
    services.run().unwrap();
    receiver.run().unwrap();
    mq_service.run().unwrap();
    tcp_server.run().unwrap();
    println!("{} | All services - are executed", dbg);
    thread::sleep(Duration::from_millis(1000));
    //
    // Sending tcp test events / receiver must not receive anything before subscription activated
    println!("{} | Sending tcp test events - to be rejected (not authenticated)", dbg);
    let mut tcp_stream = TcpStream::connect(tcp_server_addr).unwrap();
    let auth_req = Point::String(PointHlr::new(
        0,
        &Name::new(&self_name, "Auth.Secret").join(),
        secret.into(),
        Status::Ok,
        Cot::Req,
        chrono::offset::Utc::now(),
    ));
    let result = request(dbg, &mut tcp_stream, auth_req);
    let target = Point::String(PointHlr::new(0, &Name::new(&self_name, "Auth.Secret").join(), "Authentication successful".to_owned(), Status::Ok, Cot::ReqCon, chrono::offset::Utc::now()));
    assert!(result.name() == target.name(), "\nresult: {:?}\ntarget: {:?}", result.name(), target.name());
    assert!(result.value() == target.value(), "\nresult: {:?}\ntarget: {:?}", result.value(), target.value());
    assert!(result.status() == target.status(), "\nresult: {:?}\ntarget: {:?}", result.status(), target.status());
    assert!(result.cot() == target.cot(), "\nresult: {:?}\ntarget: {:?}", result.cot(), target.cot());
    println!("{} | Auth.Secret request successful!\n", dbg);
    //
    // Stopping all services
    receiver.exit();
    tcp_server.exit();
    mq_service.exit();
    services.exit();
    //
    // Waiting while all services being finished
    receiver.wait().unwrap();
    mq_service.wait().unwrap();
    tcp_server.wait().unwrap();
    services.wait().unwrap();
    //
    // Reseting dureation timer
    test_duration.exit();
}
///
///
#[test]
fn request_points() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "jds_request_test";
    println!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
    test_duration.run().unwrap();
    //
    // Preparing test data
    let tx_id = PointTxId::from_str(dbg);
    let self_name = Name::new(dbg, "Jds");
    let test_data = [
        Point::String(PointHlr::new(
            tx_id,
            &Name::new(&self_name, "Auth.Secret").join(),
            r#"{
                \"secret\": \"Auth.Secret\"
            }"#.to_string(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
            tx_id,
            &Name::new(&self_name, "Auth.Ssh").join(),
            r#"{
                \"ssh\": \"Auth.Ssh\"
            }"#.to_string(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
            tx_id,
            &Name::new(&self_name, "Points").join(),
            r#"{
                \"points\": []
            }"#.to_string(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
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
    // Configuring MultiQueue service
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
                path: assets/testing/retain/
                point:
                    path: point/id.json
        "#).unwrap()),
    ),Some(tp.scheduler())));
    //
    // Configuring Receiver
    let recv_limit = test_items_count * 2;
    let conf = serde_yaml::from_str(&format!(r#"
        service RecvService RecvService:
            recv-limit: {recv_limit}
            in queue in-queue:
                max-length: 10000
    "#)).unwrap();
    let receiver = Arc::new(RecvService::new(
        dbg,
        RecvServiceConf::from_yaml(dbg, &conf),
        tp.scheduler(),
    ));
    services.insert(receiver.clone());
    println!("{} | RecvService - ready", dbg);
    let conf = serde_yaml::from_str(&format!(r#"
        service MultiQueue:
            in queue in-queue:
                max-length: 10000
            send-to:
                - {}.in-queue
    "#, receiver.name().join())).unwrap();
    let mq_conf = MultiQueueConf::from_yaml(&self_name, &conf);
    let mq_service = Arc::new(MultiQueue::new(mq_conf, services.clone(), Some(tp.scheduler())));
    services.insert(mq_service.clone());
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
    let conf = TcpServerConf::from_yaml(&self_name, &conf);
    let tcp_server = Arc::new(TcpServer::new(conf, services.clone(), tp.scheduler()));
    services.insert(tcp_server.clone());
    println!("{} | TcpServer - ready", dbg);
    //
    // preparing MockServicePoints with the Vec<PontConfig>
    let service_points = Arc::new(MockServicePoints::new(dbg, point_configs(&self_name)));
    services.insert(service_points);
    println!("\n{} | All configurations - ok\n", dbg);
    //
    // Starting all services
    services.run().unwrap();
    receiver.run().unwrap();
    mq_service.run().unwrap();
    tcp_server.run().unwrap();
    println!("{} | All services - are executed", dbg);
    thread::sleep(Duration::from_millis(1000));
    //
    // Authenticating
    println!("{} | Sending tcp test events - to be rejected (not authenticated)", dbg);
    let mut tcp_stream = TcpStream::connect(tcp_server_addr).unwrap();
    let auth_req = Point::String(PointHlr::new(
        0,
        &Name::new(&self_name, "Auth.Secret").join(),
        secret.into(),
        Status::Ok,
        Cot::Req,
        chrono::offset::Utc::now(),
    ));
    let result = request(dbg, &mut tcp_stream, auth_req);
    assert!(result.cot() == Cot::ReqCon, "\nresult: {:?}\ntarget: {:?}", result.cot(), Cot::ReqCon);
    //
    // Sending Points request
    let subscribe_req = Point::String(PointHlr::new(
        0,
        &Name::new(&self_name, "Points").join(),
        "".to_string(),
        Status::Ok,
        Cot::Req,
        chrono::offset::Utc::now(),
    ));
    let result = request(dbg, &mut tcp_stream, subscribe_req);
    let target = Point::String(PointHlr::new(0, &Name::new(&self_name, "Points").join(), "".to_owned(), Status::Ok, Cot::ReqCon, chrono::offset::Utc::now()));
    // assert!(result.name() == target.name(), "\nresult: {:?}\ntarget: {:?}", result.name(), target.name());
    // assert!(result.value() == target.value(), "\nresult: {:?}\ntarget: {:?}", result.value(), target.value());
    let points: HashMap<String, serde_json::Value> = serde_json::from_str(&result.value().as_string()).unwrap();
    let points: HashMap<_, PointConf> = points.iter().map(|(name, value)| {
        (name, PointConf::from_json(name, value).unwrap())
    }).collect();
    println!("{} | Points request reply: {:#?}", dbg, points);
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
                panic!("PointConf '{}' - not found in the Points request reply", target.name)
            }
        }
    }
    assert!(result.status() == target.status(), "\nresult: {:?}\ntarget: {:?}", result.status(), target.status());
    assert!(result.cot() == target.cot(), "\nresult: {:?}\ntarget: {:?}", result.cot(), target.cot());
    println!("{} | Points request successful!\n", dbg);
    //
    // Stopping all services
    receiver.exit();
    tcp_server.exit();
    mq_service.exit();
    services.exit();
    //
    // Waiting while all services being finished
    receiver.wait().unwrap();
    mq_service.wait().unwrap();
    tcp_server.wait().unwrap();
    services.wait().unwrap();
    //
    // Reseting dureation timer
    test_duration.exit();
}
///
///
#[test]
#[ignore = "To be implementes..."]
fn auth_ssh() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    let dbg = "jds_request_test";
    let self_name = Name::new(dbg, "");
    println!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    //
    // Configuring MultiQueue service
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
                path: assets/testing/retain/
                point:
                    path: point/id.json
        "#).unwrap()),
    ), Some(tp.scheduler())));
    let conf = serde_yaml::from_str(&format!(r#"
        service MultiQueue:
            in queue in-queue:
                max-length: 10000
            send-to:
                - {}/RecvService0.in-queue
    "#, dbg)).unwrap();
    let mq_conf = MultiQueueConf::from_yaml(&self_name, &conf);
    let mq_service = Arc::new(MultiQueue::new(mq_conf, services.clone(), Some(tp.scheduler())));
    services.insert(mq_service.clone());
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
    "#, tcp_addr, dbg);
    let conf = serde_yaml::from_str(&conf).unwrap();
    let conf = TcpServerConf::from_yaml(self_name, &conf);
    let tcp_server = Arc::new(TcpServer::new(conf, services.clone(), tp.scheduler()));
    services.insert(tcp_server.clone());
    println!("{} | TcpServer - ready", dbg);
    //
    // Preparing test data
    let tx_id = PointTxId::from_str(dbg);
    let parent = dbg;
    let test_data = [
        Point::String(PointHlr::new(
            tx_id,
            &Name::new(parent, "JdsService/Auth.Secret").join(),
            r#"{
                \"secret\": \"Auth.Secret\"
            }"#.to_string(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
            tx_id,
            &Name::new(parent, "JdsService/Auth.Ssh").join(),
            r#"{
                \"ssh\": \"Auth.Ssh\"
            }"#.to_string(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
            tx_id,
            &Name::new(parent, "JdsService/Points").join(),
            r#"{
                \"points\": []
            }"#.to_string(),
            Status::Ok,
            Cot::Req,
            chrono::offset::Utc::now(),
        )),
        Point::String(PointHlr::new(
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
    let recv_limit = test_items_count * 2;
    let conf = serde_yaml::from_str(&format!(r#"
        service RecvService RecvService:
            recv-limit: {recv_limit}
            in queue in-queue:
                max-length: 10000
    "#)).unwrap();
    let receiver = Arc::new(RecvService::new(
        dbg,
        RecvServiceConf::from_yaml(dbg, &conf),
        tp.scheduler(),
    ));
    services.insert(receiver.clone());
    println!("{} | RecvService - ready", dbg);
    //
    // Starting all services
    services.run().unwrap();
    receiver.run().unwrap();
    mq_service.run().unwrap();
    tcp_server.run().unwrap();
    println!("{} | All services - are executed", dbg);
    thread::sleep(Duration::from_millis(200));
    //
    // Sending test events
    println!("{} | Try to get send from MultiQueue...", dbg);
    let send = services.get_link(&LinkName::from_str("MultiQueue.in-queue").unwrap()).unwrap();
    println!("{} | Try to get send from MultiQueue - ok", dbg);
    let mut sent = 0;
    for point in test_data {
        match send.send(point.clone()) {
            Ok(_) => {
                sent += 1;
                println!("{} | \t sent: {:?}", dbg, point);
            }
            Err(err) => {
                panic!("{} | Send error: {:?}", dbg, err)
            }
        }
    }
    println!("{} | Total sent: {}", dbg, sent);
    //
    // Waiting while all events being received
    receiver.wait().unwrap();
    thread::sleep(Duration::from_millis(800));
    //
    // Stopping all services
    receiver.exit();
    tcp_server.exit();
    mq_service.exit();
    services.exit();
    //
    // Verivications
    let received = receiver.received();
    let received_len = received.read().len();
    let result = received_len;
    let target = test_items_count * 2;
    println!("{} | Total received: {}", dbg, received_len);
    assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    //
    // Verifing JdsService replies
    let mut replies = 0;
    let mut reply_errors = 0;
    for point in received.read().iter() {
        match point.cot() {
            // Cot::Inf => todo!(),
            // Cot::Act => todo!(),
            // Cot::ActCon => todo!(),
            // Cot::ActErr => todo!(),
            // Cot::Req => todo!(),
            Cot::ReqCon => {
                replies += 1;
                println!("{} | Received ReqCon reply: {:?}", dbg, point);
            }
            Cot::ReqErr => {
                reply_errors += 1;
                println!("{} | Received ReqErr reply: {:?}", dbg, point);
            }
            // Cot::Read => todo!(),
            // Cot::Write => todo!(),
            // Cot::All => todo!(),
            _ => {
                println!("{} | Received unknown point: {:?}", dbg, point);
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
    mq_service.wait().unwrap();
    tcp_server.wait().unwrap();
    services.wait().unwrap();
    //
    // Reseting dureation timer
    test_duration.exit();
}
