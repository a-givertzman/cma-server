#[cfg(test)]

mod multi_queue {
    use log::debug;
    use sal_sync::services::{entity::object::Object, retain::{retain_conf::RetainConf, retain_point_conf::RetainPointConf}, service::service::Service};
    use std::{sync::{Arc, RwLock, Once}, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::multi_queue_config::MultiQueueConfig, services::{multi_queue::multi_queue::MultiQueue, safe_lock::rwlock::SafeLock, services::Services, task::nested_function::reset_counter::AtomicReset},
        tests::unit::services::multi_queue::{mock_recv_service::MockRecvService, mock_send_service::{self, MockSendService}},
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
    /// Test MultiQueue for - static link
    /// - action: read
    #[test]
    fn static_read() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "multi_queue_read_test";
        println!("\n{}", self_id);
        //
        // can be changed
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
        let count = 30;
        let total_count = count * test_data.len();
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(
            Some("assets/testing/retain/"),
            Some(RetainPointConf::new("point/id.json", None))
        ))));
        let mut recv_services = vec![];
        for _ in 0..count {
            let recv_service = Arc::new(RwLock::new(MockRecvService::new(
                self_id,
                "in-queue",
                Some(iterations),
            )));
            services.wlock(self_id).insert(recv_service.clone());
            recv_services.push(recv_service);
        }
        let mut conf = r#"
            service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
        "#.to_string();
        for s in &recv_services {
            // conf = format!("{}\n                    - /{}/MockRecvService{}.in-queue", conf, self_id, i)
            conf = format!("{}\n                    - {}.in-queue", conf, s.read().unwrap().name().join())
        }
        let conf = serde_yaml::from_str(&conf).unwrap();
        let mq_conf = MultiQueueConfig::from_yaml(self_id, &conf);
        debug!("mqConf: {:?}", mq_conf);
        let mq_service = Arc::new(RwLock::new(MultiQueue::new(mq_conf, services.clone())));
        services.wlock(self_id).insert(mq_service.clone());
        let mut recv_handles = vec![];
        let timer = Instant::now();
        mock_send_service::COUNT.reset(0);
        let send_service = Arc::new(RwLock::new(MockSendService::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            services.clone(),
            test_data.clone(),
            None,
        )));
        services.wlock(self_id).insert(send_service.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        mq_service.write().unwrap().run().unwrap();
        for service in &mut recv_services {
            let handle = service.write().unwrap().run().unwrap();
            recv_handles.push(handle);
        }
        send_service.write().unwrap().run().unwrap();
        for thd in recv_handles {
            thd.wait().unwrap();
        }
        println!("\nelapsed: {:?}", timer.elapsed());
        println!("total test events: {:?}", total_count);
        println!("sent events: {:?}\n", count * send_service.read().unwrap().sent().read().unwrap().len());
        let mut received = vec![];
        let target = test_data_len;
        for recv_service in &recv_services {
            let len = recv_service.read().unwrap().received().read().len();
            assert!(len == target, "\nresult: {:?}\ntarget: {:?}", len, target);
            received.push(len);
        }
        println!("recv events: {} {:?}", received.iter().sum::<usize>(), received);
        for service in recv_services {
            service.read().unwrap().exit();
        }
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        test_duration.exit();
    }
}
