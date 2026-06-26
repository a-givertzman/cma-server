#[cfg(test)]

use std::{sync::{Arc, Once}, thread, time::{Duration, Instant}};
use rand::Rng;
use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::Name, Service, Services}, thread_pool::ThreadPool};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::{
    services::{task::TaskTestReceiver, udp_client::{UdpClient, UdpClientConf}},
    tests::unit::services::udp_client::mock_udp_server::{MockUdpServer, MockUdpServerConfig},
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
/// Testing UdpClient basic functionality
#[test]
fn random_i16() {
    DebugSession::new().filter(LogLevel::Debug).init();
    init_once();
    init_each();
    log::debug!("");
    let dbg = "test";
    log::debug!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(30));
    test_duration.run().unwrap();
    let mut rng = rand::rng();
    let rng = &mut rng;
    ////////////////////////////////////////////////////////
    //     Configure here                                 //
    ////////////////////////////////////////////////////////
    // Total test values                                  //
    let count = 512 * 100;
    // Values<i16> in DATA field of UDP message
    let message_length = 1024;
    // Sampling frequency                                 //
    let freq = 300_000.0; // Hz
    ////////////////////////////////////////////////////////
    // Messages sent per second
    let messages_per_sec = freq / (message_length as f64);
    let test_data: Vec<u16> = (0..count).map(|_| rng.random_range(0000..4096) as u16).collect();
    // let test_data: Vec<i16> = (0..count).collect();
    log::info!("{}.random_i16 | test data len: {}", dbg, test_data.len());
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
                path: assets/testing/retain/
                point:
                    path: point/id.json
        "#).unwrap()),
    ), Some(tp.scheduler())).unwrap());
    let path = "./src/tests/unit/services/udp_client/udp-client.yaml";
    let conf = UdpClientConf::read(dbg, path);
    let udp_client = Arc::new(UdpClient::new(conf, services.clone(), tp.scheduler()));
    services.insert(udp_client.clone());
    // let conf = MultiQueueConf::from_yaml(
    //     self_id,
    //     &serde_yaml::from_str(r"service MultiQueue:
    //         in queue in-queue:
    //             max-length: 10000
    //     ").unwrap(),
    // );
    // let multi_queue = Arc::new(MultiQueue::new(conf, services.clone())));
    // services.insert(multi_queue.clone());
    let receiver = Arc::new(TaskTestReceiver::new(&dbg, "", test_data.len()));
    services.insert(receiver.clone());
    let udp_server = Arc::new(MockUdpServer::new(
        dbg,
        MockUdpServerConfig {
            name: Name::new(dbg, "MockUdpServer"),
            local_addr: "127.0.0.1:15180".to_owned(),
            channel: 0,
            count: 512,
            mtu: 1500,
            cycle: Duration::from_secs_f64(1.0 / messages_per_sec),
        },
        services.clone(),
        &test_data,
    ));
    services.insert(udp_server.clone());
    let time = Instant::now();
    services.run().unwrap();
    thread::sleep(Duration::from_millis(10));
    receiver.run().unwrap();
    // let multi_queue_handle = multi_queue.run().unwrap();
    udp_client.run().unwrap();
    thread::sleep(Duration::from_millis(10));
    udp_server.run().unwrap();
    
    let mut received = 0;
    let timeout = Duration::from_millis(128);
    let wait_time = Instant::now();
    while received < test_data.len() {
        thread::yield_now();
        received = receiver.received().len();
        log::debug!("{} | receiver {}/{} ...", dbg, received, test_data.len());
        if wait_time.elapsed() > timeout {
            break;
        }
    }
    receiver.wait().unwrap();
    let elapsed = time.elapsed();
    log::debug!("{} | wait for receiver - finished", dbg);
    log::debug!("{} | get received...", dbg);
    let received = receiver.received();
    log::debug!("{} | get received points - ok", dbg);
    log::info!("Sampling freq: {}", freq);
    log::info!("Messages sent per second: {}", messages_per_sec);
    log::info!("Total test values: {}", test_data.len());
    log::info!("Total received: {}", received.len());
    log::info!("Total elapsed: {:?}", elapsed);
    let mut test_data_clone = test_data.clone();
    for (_, point) in received.iter().enumerate() {
        let result = point.value().as_int();
        if let Some(index) = test_data_clone.iter().position(|value| *value as i64 == result) {
            test_data_clone.swap_remove(index);
        } else {
            log::warn!("missed: {:?}", result);
        }
    }
    let mut test_data_iter = test_data.iter();
    for (step, point) in received.iter().enumerate() {
        log::trace!("point: {:?} | {}", point.value(), point.name());
        let result = point.name();
        let target = "/test/UdpClient/Sensor1".to_owned();
        assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        let result = point.value().as_int();
        let target = test_data_iter.next().unwrap();
        assert!(result == *target as i64, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
    }
    receiver.exit();
    udp_client.exit();
    udp_client.wait().unwrap();
    udp_server.exit();
    // multi_queue.exit();
    services.exit();
    udp_server.wait().unwrap();
    // multi_queue_handle.wait().unwrap();
    services.wait().unwrap();
    test_duration.exit();
}
