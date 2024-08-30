#[cfg(test)]

mod tests {
    use std::{sync::{Arc, Once, RwLock}, thread, time::Duration};
    use sal_sync::services::{retain::{retain_conf::RetainConf, retain_point_conf::RetainPointConf}, service::service::Service};
    use testing::stuff::{max_test_duration::TestDuration, wait::WaitTread};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
        use crate::{conf::{multi_queue_config::MultiQueueConfig, udp_client_config::udp_client_config::UdpClientConfig}, services::{multi_queue::multi_queue::MultiQueue, safe_lock::SafeLock, services::Services, udp_client::udp_client::UdpClient}};
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
    fn test_task_cycle() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        log::debug!("");
        let self_id = "test";
        log::debug!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(1));
        test_duration.run().unwrap();
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(
            Some("assets/testing/retain/"),
            Some(RetainPointConf::new("point/id.json", None))
        ))));
        let path = "./src/tests/unit/services/udp_client/udp-client.yaml";
        let conf = UdpClientConfig::read(self_id, path);
        let udp_client = Arc::new(RwLock::new(UdpClient::new(self_id, conf, services.clone())));
        services.wlock(self_id).insert(udp_client.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
            ").unwrap(),
        );
        let multi_queue = Arc::new(RwLock::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());

        let udp_client_handle = udp_client.write().unwrap().run().unwrap();
        thread::sleep(Duration::from_secs(3));
        
        udp_client.write().unwrap().exit();
        udp_client_handle.wait().unwrap();
        // assert!(result == target, "step {} \nresult: {:?}\ntarget: {:?}", step, result, target);
        test_duration.exit();
    }
}
