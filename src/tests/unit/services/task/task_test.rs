#[cfg(test)]

mod task {
    use log::{trace, info};
    use std::{env, sync::{Arc, Mutex, Once, RwLock}, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{point_config::name::Name, task_config::TaskConfig},
        services::{safe_lock::SafeLock, service::service::Service, services::Services, task::{task::Task, task_test_producer::TaskTestProducer, task_test_receiver::TaskTestReceiver}},
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
    ///
    #[test]
    fn structure() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "task_test";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(3));
        test_duration.run().unwrap();
        //
        // can be changed
        let iterations = 10;
        trace!("dir: {:?}", env::current_dir());
        let path = "./src/tests/unit/services/task/task_test_struct.yaml";
        let config = TaskConfig::read(&self_name, path);
        trace!("config: {:?}", &config);
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            iterations,
        )));
        services.wlock(self_id).insert(receiver.clone());      // "TaskTestReceiver",
        let test_data = RandomTestValues::new(
            self_id,
            vec![
                Value::Real(-7.035),
                Value::Real(-2.5),
                Value::Real(-5.5),
                Value::Real(-1.5),
                Value::Real(-1.0),
                Value::Real(-0.1),
                Value::Real(0.1),
                Value::Real(1.0),
                Value::Real(1.5),
                Value::Real(5.5),
                Value::Real(2.5),
                Value::Real(7.035),
            ],
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        let total_count = test_data.len();
        assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(Mutex::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/Task1.in-queue", self_id),
            Duration::ZERO,
            services.clone(),
            test_data,
        )));
        let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
        services.wlock(self_id).insert(task.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.lock().unwrap().run().unwrap();
        info!("task runing - ok");
        // thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.lock().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        producer.lock().unwrap().exit();
        task.lock().unwrap().exit();
        services.rlock(self_id).exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        services_handle.wait().unwrap();
        let sent = producer.lock().unwrap().sent().lock().unwrap().len();
        let result = receiver.lock().unwrap().received().lock().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        assert!(sent == iterations, "\nresult: {:?}\ntarget: {:?}", sent, iterations);
        assert!(result == iterations, "\nresult: {:?}\ntarget: {:?}", result, iterations);
        test_duration.exit();
    }
    ///
    ///
    #[test]
    #[ignore = "TODO - transfered values assertion not implemented yet"]
    fn transfer() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        info!("test");
        let self_id = "test";
        let self_name = Name::new("", self_id);
        //
        // Can be changed
        let iterations = 10;
        trace!("dir: {:?}", env::current_dir());
        let path = "./src/tests/unit/services/task/task_test_struct.yaml";
        // let path = "./src/tests/unit/task/task_test.yaml";
        let config = TaskConfig::read(&self_name, path);
        trace!("config: {:?}", &config);
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            iterations,
        )));
        services.wlock(self_id).insert(receiver.clone());      // "TaskTestReceiver",
        let test_data = RandomTestValues::new(
            self_id,
            vec![
                Value::Real(f32::MAX),
                Value::Real(f32::MIN),
                Value::Real(f32::MIN_POSITIVE),
                Value::Real(-f32::MIN_POSITIVE),
                Value::Real(0.11),
                Value::Real(1.33),
                Value::Double(f64::MAX),
                Value::Double(f64::MIN),
                Value::Double(f64::MIN_POSITIVE),
                Value::Double(-f64::MIN_POSITIVE),
                Value::Double(0.11),
                Value::Double(1.33),
            ],
            iterations,
        );
        let test_data: Vec<Value> = test_data.collect();
        // let totalCount = test_data.len();
        let producer = Arc::new(Mutex::new(TaskTestProducer::new(
            self_id,
            "Task.in-queue",
            Duration::ZERO,
            services.clone(),
            test_data,
        )));
        let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
        services.wlock(self_id).insert(task.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        let producer_handle = producer.lock().unwrap().run().unwrap();
        trace!("task runing...");
        let time = Instant::now();
        task.lock().unwrap().run().unwrap();
        trace!("task runing - ok");
        producer_handle.wait().unwrap();
        receiver_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let producer_sent = producer.lock().unwrap().sent();
        let sent = producer_sent.lock().unwrap();
        let receiver_received = receiver.lock().unwrap().received();
        let mut received = receiver_received.lock().unwrap();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent.len());
        println!("received: {:?}", received.len());
        assert!(sent.len() == iterations, "\nresult: {:?}\ntarget: {:?}", sent.len(), iterations);
        assert!(received.len() == iterations, "\nresult: {:?}\ntarget: {:?}", received.len(), iterations);
        for sent_point in sent.iter() {
            let recv_point = received.pop().unwrap();
            assert!(&recv_point == sent_point, "\nresult: {:?}\ntarget: {:?}", recv_point, sent_point);
        }
    }
}

