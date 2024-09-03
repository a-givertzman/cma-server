#[cfg(test)]

mod udp_client {
    use std::{sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use rand::Rng;
    use sal_sync::services::{entity::name::Name, retain::{retain_conf::RetainConf, retain_point_conf::RetainPointConf}, service::service::Service};
    use testing::stuff::{max_test_duration::TestDuration, wait::WaitTread};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::udp_client_config::udp_client_config::UdpClientConfig,
        services::{safe_lock::SafeLock, services::Services, task::task_test_receiver::TaskTestReceiver, udp_client::udp_client::UdpClient},
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
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(100));
        test_duration.run().unwrap();
        let mut rng = rand::thread_rng();
        let rng = &mut rng;
        ////////////////////////////////////////////////////////
        //     Configure here                                 //
        ////////////////////////////////////////////////////////
        // Total test values                                  //
        let count = 100_000;
        // Values<i16> in DATA field of UDP message
        let message_length = 1024;
        // Sampling frequency                                 //
        let freq = 300_000.0; // Hz
        ////////////////////////////////////////////////////////
        // Messages sent per second
        let messages_per_sec = freq / (message_length as f64);
        let test_data: Vec<i16> = (0..count).map(|_| rng.gen_range(-2048..2048) as i16).collect();
        // let test_data: Vec<i16> = (0..count).collect();
        log::info!("{}.random_i16 | test data len: {}", self_id, test_data.len());
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(
            Some("assets/testing/retain/"),
            Some(RetainPointConf::new("point/id.json", None))
        ))));
        let path = "./src/tests/unit/services/udp_client/udp-client.yaml";
        let conf = UdpClientConfig::read(self_id, path);
        let udp_client = Arc::new(RwLock::new(UdpClient::new(conf, services.clone())));
        services.wlock(self_id).insert(udp_client.clone());
        // let conf = MultiQueueConfig::from_yaml(
        //     self_id,
        //     &serde_yaml::from_str(r"service MultiQueue:
        //         in queue in-queue:
        //             max-length: 10000
        //     ").unwrap(),
        // );
        // let multi_queue = Arc::new(RwLock::new(MultiQueue::new(conf, services.clone())));
        // services.wlock(self_id).insert(multi_queue.clone());
        let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(&self_id, "", "in-queue", test_data.len())));
        services.wlock(self_id).insert(receiver.clone());
        let udp_server = Arc::new(RwLock::new(MockUdpServer::new(
            self_id,
            MockUdpServerConfig {
                name: Name::new(self_id, "MockUdpServer"),
                local_addr: "127.0.0.1:15180".to_owned(),
                channel: 0,
                count: 512,
                mtu: 1500,
                cycle: Duration::from_secs_f64(1.0 / messages_per_sec),
            },
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(udp_server.clone());
        let time = Instant::now();
        let services_handle = services.wlock(self_id).run().unwrap();
        thread::sleep(Duration::from_millis(10));
        let receiver_handle = receiver.write().unwrap().run().unwrap();
        // let multi_queue_handle = multi_queue.write().unwrap().run().unwrap();
        let udp_client_handle = udp_client.write().unwrap().run().unwrap();
        thread::sleep(Duration::from_millis(100));
        let udp_server_handle = udp_server.write().unwrap().run().unwrap();
        
        let mut received = 0;
        let timeout = Duration::from_secs(3);
        let wait_time = Instant::now();
        while received < test_data.len() {
            thread::sleep(Duration::from_millis(500));
            let r = receiver.try_read().unwrap().received();
            received = r.read().unwrap().len();
            log::debug!("{} | receiver {}/{} ...", self_id, received, test_data.len());
            if wait_time.elapsed() > timeout {
                break;
            }
        }
        receiver.read().unwrap().exit();
        receiver_handle.wait().unwrap();
        let elapsed = time.elapsed();
        log::debug!("{} | wait for receiver - finished", self_id);
        log::debug!("{} | get received...", self_id);
        let received = receiver.try_read().unwrap().received();
        log::debug!("{} | get received - ok", self_id);
        log::debug!("{} | get received points...", self_id);
        let received = received.read().unwrap();
        log::debug!("{} | get received points - ok", self_id);
        log::info!("Sampling freq: {}", freq);
        log::info!("Messages sent per second: {}", messages_per_sec);
        log::info!("Total test values: {}", test_data.len());
        log::info!("Total received: {}", received.len());
        log::info!("Total elapsed: {:?}", elapsed);
        let mut test_data_clone = test_data.clone();
        for (_, point) in received.iter().enumerate() {
            let result = point.value().as_int() as i16;
            if let Some(index) = test_data_clone.iter().position(|value| *value == result) {
                test_data_clone.swap_remove(index);
            } else {
                log::warn!("missed: {:?}", result);
            }
        }
        let mut test_data_iter = test_data.iter();
        for (step, point) in received.iter().enumerate() {
            log::trace!("point: {:?} | {}", point.value(), point.name());
            let result = point.name();
            let target = "/test/UdpClient/data/Sensor1".to_owned();
            assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
            let result = point.value().as_int();
            let target = test_data_iter.next().unwrap();
            assert!(result == *target as i64, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        }
        udp_client.write().unwrap().exit();
        udp_client_handle.wait().unwrap();
        udp_server.read().unwrap().exit();
        // multi_queue.read().unwrap().exit();
        services.read().unwrap().exit();
        udp_server_handle.wait().unwrap();
        // multi_queue_handle.wait().unwrap();
        services_handle.wait().unwrap();
        test_duration.exit();
    }
}
