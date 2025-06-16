#[cfg(test)]

mod task {
    use sal_sync::{services::{
        conf::{ConfTree, ServicesConf}, entity::Name, Service, Services
    }, thread_pool::ThreadPool};
    use std::{env, sync::{Arc, Once}, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::task_config::TaskConfig,
        services::task::{task::Task, task_test_producer::TaskTestProducer, task_test_receiver::TaskTestReceiver},
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
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbg = "task_test";
        let self_name = Name::new("", dbg);
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(3));
        test_duration.run().unwrap();
        //
        // can be changed
        let iterations = 10;
        log::trace!("dir: {:?}", env::current_dir());
        let path = "./src/tests/unit/services/task/task_test_struct.yaml";
        let config = TaskConfig::read(&self_name, path);
        log::trace!("config: {:?}", &config);
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
            "#).unwrap()),
        ), Some(tp.scheduler())));
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "in-queue",
            iterations,
        ));
        services.insert(receiver.clone());      // "TaskTestReceiver",
        let test_data = RandomTestValues::new(
            dbg,
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
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            &format!("/{}/Task1.in-queue", dbg),
            Duration::ZERO,
            services.clone(),
            test_data,
        ));
        let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
        services.insert(task.clone());
        services.run().unwrap();
        receiver.run().unwrap();
        log::info!("receiver runing - ok");
        task.run().unwrap();
        log::info!("task runing - ok");
        // thread::sleep(Duration::from_millis(100));
        producer.run().unwrap();
        log::info!("producer runing - ok");
        let time = Instant::now();
        receiver.wait().unwrap();
        producer.exit();
        task.exit();
        services.exit();
        task.wait().unwrap();
        producer.wait().unwrap();
        services.wait().unwrap();
        let sent = producer.sent().read().len();
        let result = receiver.received().read().len();
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
        log::info!("test");
        let dbg = "test";
        let self_name = Name::new("", dbg);
        //
        // Can be changed
        let iterations = 10;
        log::trace!("dir: {:?}", env::current_dir());
        let path = "./src/tests/unit/services/task/task_test_struct.yaml";
        // let path = "./src/tests/unit/task/task_test.yaml";
        let config = TaskConfig::read(&self_name, path);
        log::trace!("config: {:?}", &config);
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
            "#).unwrap()),
        ), Some(tp.scheduler())));
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "in-queue",
            iterations,
        ));
        services.insert(receiver.clone());      // "TaskTestReceiver",
        let test_data = RandomTestValues::new(
            dbg,
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
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            "Task.in-queue",
            Duration::ZERO,
            services.clone(),
            test_data,
        ));
        let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
        services.insert(task.clone());
        services.run().unwrap();
        receiver.run().unwrap();
        producer.run().unwrap();
        log::trace!("task runing...");
        let time = Instant::now();
        task.run().unwrap();
        log::trace!("task runing - ok");
        producer.wait().unwrap();
        receiver.wait().unwrap();
        services.exit();
        services.wait().unwrap();
        let producer_sent = producer.sent();
        let sent = producer_sent.read();
        let receiver_received = receiver.received();
        let mut received = receiver_received.write();
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

