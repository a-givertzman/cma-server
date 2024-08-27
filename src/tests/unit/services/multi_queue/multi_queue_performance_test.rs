#[cfg(test)]

mod multi_queue {
    use std::{collections::HashMap, sync::{Arc, Mutex, Once, RwLock}, thread, time::{Duration, Instant}};
    use sal_sync::services::{entity::name::Name, retain::retain_conf::RetainConf, service::service::Service};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        services::{safe_lock::SafeLock, services::Services}, tests::unit::services::multi_queue::{mock_multi_queue::MockMultiQueue, mock_multi_queue_match::MockMultiQueueMatch, mock_recv_service::MockRecvService, mock_send_service::MockSendService}
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
    /// Can be changed
    const ITERATIONS: usize = 1_000_000;
    ///
    /// Use to estimate performance of multiqueue without matching producer's id
    #[ignore = "Performance test"]
    #[test]
    fn performance() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test MultiQueue Performance";
        println!("\n{}", self_id);
        let iterations = ITERATIONS;
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let receiver_count = 3;
        let producer_count = 3;
        let total_count = iterations * producer_count;
        let mut receivers: HashMap<String, Arc<Mutex<MockRecvService>>> = HashMap::new();
        let mut producers: HashMap<String, MockSendService> = HashMap::new();
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(None::<&str>, None))));
        for i in 0..receiver_count {
            let receiver = Arc::new(Mutex::new(MockRecvService::new(
                self_id,
                "rx-queue",
                Some(total_count)
            )));
            let receiver_id = format!("Receiver{}", i + 1);
            services.wlock(self_id).insert(receiver.clone());
            receivers.insert(receiver_id.clone(), receiver);
            println!(" Receiver {} created", receiver_id);
        }
        println!(" All receivers created");
        println!(" Creating Mock Multiqueue...");
        let mq_service = Arc::new(Mutex::new(MockMultiQueue::new(
            self_id,
            receivers.keys().map(|v| {
                format!("{}.rx-queue", v)
            }).collect(),
            "rx-queue",
            services.clone(),
        )));
        println!(" Creating Mock Multiqueue - ok");
        println!(" Inserting Mock Multiqueue into Services...");
        services.wlock(self_id).insert(mq_service.clone());
        println!(" Inserting Mock Multiqueue into Services - ok");
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
                Value::Double(0.0),
                Value::Double(1.33),
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
        let services_handle = services.wlock(self_id).run().unwrap();
        println!(" Trying to start Multiqueue...:");
        mq_service.lock().unwrap().run().unwrap();
        let mut recv_handles  = vec![];
        for (_recv_id, recv) in &receivers {
            let h = recv.lock().unwrap().run().unwrap();
            recv_handles.push(h)
        }
        for i in 0..producer_count {
            let mut prod = MockSendService::new(self_id, "MultiQueue.rx-queue", services.clone(), test_data.clone(), None);
            prod.run().unwrap();
            producers.insert(format!("MockSendService{}", i), prod);
        }

        let timer = Instant::now();
        for h in recv_handles {
            h.wait().unwrap();
        }
        services_handle.wait().unwrap();
        println!("\n Elapsed: {:?}", timer.elapsed());
        println!(" Total test events: {:?}", total_count);
        let (total_sent, all_sent) = get_sent(&producers);
        println!(" Sent events: {}\t{:?}", total_sent, all_sent);
        let (total_received, all_received) = get_received(&receivers);
        println!(" Recv events: {}\t{:?}\n", total_received, all_received);

        assert!(total_sent == total_count, "\nresult: {:?}\ntarget: {:?}", total_sent, total_count);
        assert!(total_received == total_count * receiver_count, "\nresult: {:?}\ntarget: {:?}", total_received, total_count * receiver_count);
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        test_duration.exit();
    }
    ///
    /// Use to estimate performance of multiqueue with matching producer's id
    #[ignore = "Performance test"]
    #[test]
    fn match_performance() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test MultiQueue Performance with matching by producer ID";
        println!("\n{}", self_id);
        let self_id = "MultiQueuePerformance";
        let iterations = ITERATIONS;
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let receiver_count = 3;
        let producer_count = 3;
        let total_count = iterations * producer_count;
        let mut receivers: HashMap<String, Arc<Mutex<MockRecvService>>> = HashMap::new();
        let mut producers: HashMap<String, MockSendService> = HashMap::new();
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(None::<&str>, None))));
        for i in 0..receiver_count {
            let receiver = Arc::new(Mutex::new(MockRecvService::new(
                self_id,
                "rx-queue",
                Some(total_count)
            )));
            let receiver_id = format!("/{}/MockRecvService{}", self_id, i);
            services.wlock(self_id).insert(receiver.clone());
            receivers.insert(receiver_id.clone(), receiver);
            println!(" Receiver {} created", receiver_id);
        }
        println!(" All receivers created");
        println!(" Creating Mock Multiqueue...");
        let mq_service = Arc::new(Mutex::new(MockMultiQueueMatch::new(
            self_id,
            receivers.keys().map(|v| {
                format!("{}.rx-queue", v)
            }).collect(),
            "rx-queue",
            services.clone(),
        )));
        println!(" Creating Mock Multiqueue - ok");
        println!(" Inserting Mock Multiqueue into Services...");
        services.wlock(self_id).insert(mq_service.clone());
        println!(" Inserting Mock Multiqueue into Services - ok");
        let test_data = RandomTestValues::new(
            self_id,
            vec![
                Value::Int(7),
                Value::Real(1.3),
                Value::Double(1.3),
                Value::Bool(true),
                Value::Bool(false),
                Value::String("test1".to_string()),
                Value::String("test2".to_string()),
            ],
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        let services_handle = services.wlock(self_id).run().unwrap();
        thread::sleep(Duration::from_millis(50));
        println!(" Trying to start Multiqueue...:");
        mq_service.lock().unwrap().run().unwrap();
        let mut recv_handles  = vec![];
        for (_recv_id, recv) in &receivers {
            let h = recv.lock().unwrap().run().unwrap();
            recv_handles.push(h)
        }
        for i in 0..producer_count {
            let mut prod = MockSendService::new(
                self_id,
                &Name::new(self_id, "MockMultiQueueMatch0.rx-queue").join(),
                services.clone(),
                test_data.clone(),
                None,
            );
            prod.run().unwrap();
            producers.insert(format!("MockSendService{}", i), prod);
        }
        let timer = Instant::now();
        for h in recv_handles {
            h.wait().unwrap();
        }
        services_handle.wait().unwrap();
        println!("\n Elapsed: {:?}", timer.elapsed());
        println!(" Total test events: {:?}", total_count);
        let (total_sent, all_sent) = get_sent(&producers);
        println!(" Sent events: {}\t{:?}", total_sent, all_sent);
        let (total_received, all_received) = get_received(&receivers);
        println!(" Recv events: {}\t{:?}\n", total_received, all_received);
        assert!(total_sent == total_count, "\nresult: {:?}\ntarget: {:?}", total_sent, total_count);
        assert!(total_received == total_count * receiver_count, "\nresult: {:?}\ntarget: {:?}", total_received, total_count * receiver_count);
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        test_duration.exit();
    }
    ///
    ///
    fn get_sent<'a>(producers: &'a HashMap<String, MockSendService>) -> (usize, HashMap<&'a str, usize>) {
        let mut total_sent = 0;
        let mut all_sent: HashMap<&'a str, usize> = HashMap::new();
        for (prod_id, prod) in producers {
            let sent = prod.sent().lock().unwrap().len();
            total_sent += sent;
            all_sent.insert(prod_id, sent);
        }
        (total_sent, all_sent)
    }
    ///
    ///
    fn get_received<'a>(receivers: &'a HashMap<String, Arc<Mutex<MockRecvService>>>) -> (usize, HashMap<&'a str, usize>) {
        let mut total_received = 0;
        let mut all_received: HashMap<&'a str, usize> = HashMap::new();
        for (recv_id, recv) in receivers {
            let recved = recv.lock().unwrap().received().lock().unwrap().len();
            total_received += recved;
            all_received.insert(recv_id, recved);
        }
        (total_received, all_received)
    }
}