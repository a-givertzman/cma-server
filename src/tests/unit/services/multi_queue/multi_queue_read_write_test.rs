#[cfg(test)]

mod multi_queue {
    use log::debug;
    use sal_sync::services::{retain::retain_conf::RetainConf, service::service::Service};
    use std::{sync::{Arc, Once, RwLock}, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::multi_queue_config::MultiQueueConfig,
        services::{multi_queue::multi_queue::MultiQueue, safe_lock::SafeLock, services::Services},
        tests::unit::services::multi_queue::mock_recv_send_service::MockRecvSendService,
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
    /// Test MultiQueue for static link
    /// - action: read-write
    #[test]
    fn read_write() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "multi_queue_read_write_test";
        println!("\n{}", self_id);
        let iterations = 10;
        let test_data = RandomTestValues::new(
            self_id,
            vec![
                Value::Int(i64::MIN),
                Value::Int(i64::MAX),
                Value::Int(-7),
                Value::Int(0),
                Value::Int(12),
                Value::Real(f32::MAX),
                Value::Real(f32::MIN),
                Value::Real(f32::MIN_POSITIVE),
                Value::Real(-f32::MIN_POSITIVE),
                Value::Real(0.0),
                Value::Real(1.33),
                Value::Double(f64::MAX),
                Value::Double(f64::MIN),
                Value::Double(f64::MIN_POSITIVE),
                Value::Double(-f64::MIN_POSITIVE),
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(true),
                Value::String("test1".to_string()),
                Value::String("test1test1test1test1test1test1test1test1test1test1test1test1test1test1test1".to_string()),
                Value::String("test2".to_string()),
                Value::String("test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2test2".to_string()),
            ],
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        let test_data_len = test_data.len();
        let count = 3;
        let total_count = count * test_data_len;
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let mut conf = r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
        "#.to_string();
        for i in 0..count {
            conf = format!("{}\n                    - /{}/MockRecvSendService{}.in-queue", conf, self_id, i)
        }
        let conf = serde_yaml::from_str(&conf).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(self_id, &conf);
        debug!("mqConf: {:?}", mq_conf);
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(None::<&str>, None))));
        let mq_service = Arc::new(RwLock::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());
        let timer = Instant::now();
        let mut rs_services = vec![];
        for _ in 0..count {
            let rs_service = Arc::new(RwLock::new(MockRecvSendService::new(
                self_id,
                "in-queue",
                &format!("/{}/MultiQueue.in-queue", self_id),
                services.clone(),
                test_data.clone(),
                Some(total_count),
            )));
            services.wlock(self_id).insert(rs_service.clone());
            rs_services.push(rs_service);
        }
        let services_handle = services.wlock(self_id).run().unwrap();
        mq_service.write().unwrap().run().unwrap();
        let mut recv_handles = vec![];
        for service in &mut rs_services {
            let handle = service.write().unwrap().run().unwrap();
            recv_handles.push(handle);
        }
        for thd in recv_handles {
            thd.wait().unwrap();
        }
        println!("\nelapsed: {:?}", timer.elapsed());
        println!("total test events: {:?}", total_count);
        for service in &rs_services {
            println!("sent events: {:?}\n", service.read().unwrap().sent().read().unwrap().len());
        }
        let mut received = vec![];
        let target = total_count;
        for recv_service in &rs_services {
            let len = recv_service.read().unwrap().received().read().unwrap().len();
            assert!(len == target, "\nresult: {:?}\ntarget: {:?}", len, target);
            received.push(len);
        }
        println!("recv events: {} {:?}", received.iter().sum::<usize>(), received);
        for service in rs_services {
            service.read().unwrap().exit();
        }
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        test_duration.exit();
    }
}
