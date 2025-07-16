#[cfg(test)]

mod fn_export {
    use log::{debug, info, trace};
    use sal_sync::services::{entity::name::Name, retain::{retain_conf::RetainConf, retain_point_conf::RetainPointConf}, service::service::Service};
    use std::{env, sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{multi_queue_config::MultiQueueConfig, task_config::TaskConfig},
        services::{multi_queue::multi_queue::MultiQueue, safe_lock::rwlock::SafeLock, services::Services, task::{task::Task, task_test_receiver::TaskTestReceiver}},
        tests::unit::services::task::cma_recorder::task_test_producer::TaskTestProducer,
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
    /// Testing Task function 'Export' with 'enable' input used
    #[test]
    fn export_point_with_enable() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "App";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(
            Some("assets/testing/retain/"),
            Some(RetainPointConf::new("point/id.json", None))
        ))));
        let config = TaskConfig::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RecorderTask:
                    cycle: 1 ms
                    in queue recv-queue:
                        max-length: 10000
                    subscribe:
                        /App/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf
                    fn Debug debug01:
                        input fn Export:
                            enable: const bool true
                            conf point Load001:
                                type: 'Real'
                            input: point real '/App/Load'
                            send-to: /App/MultiQueue.in-queue
                    fn Debug debug02:
                        input fn Export:
                            enable: point bool '/App/Enable'
                            conf point Load002:
                                type: 'Real'
                            input: point real '/App/RecorderTask/Load001'
                            send-to: /App/TaskTestReceiver.in-queue
            ").unwrap(),
        );
        trace!("config: {:?}", config);
        debug!("Task config points: {:#?}", config.points());
        let task = Arc::new(RwLock::new(Task::new(config, services.clone())));
        debug!("Task points: {:#?}", task.read().unwrap().points());
        services.wlock(self_id).insert(task.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
            ").unwrap(),
        );
        let multi_queue = Arc::new(RwLock::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Enable", self_id), Value::Bool(false)),
            (format!("/{}/Load", self_id), Value::Real(-7.035)),
            (format!("/{}/Load", self_id), Value::Real(-2.5)),
            (format!("/{}/Enable", self_id), Value::Bool(true)),
            (format!("/{}/Load", self_id), Value::Real(-5.5)),
            (format!("/{}/Load", self_id), Value::Real(-1.5)),
            (format!("/{}/Load", self_id), Value::Real(-1.0)),
            (format!("/{}/Load", self_id), Value::Real(-0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(1.0)),
            (format!("/{}/Load", self_id), Value::Real(1.5)),
            (format!("/{}/Load", self_id), Value::Real(5.5)),
            (format!("/{}/Enable", self_id), Value::Bool(false)),
            (format!("/{}/Load", self_id), Value::Real(2.5)),
            (format!("/{}/Load", self_id), Value::Real(7.035)),
        ];
        let total_count = test_data.len();
        let mut target_data = vec![];
        let mut enable = false;
        let mut target = (String::new(), Value::Real(0.0));
        for (name, value) in test_data.clone() {
            if name == format!("/{}/Enable", self_id) {
                enable = value.as_bool();
            } else if name == format!("/{}/Load", self_id) {
                target = (name.clone(), value);
            }
            if enable {
                target_data.push(target.clone());
            }
        }
        let target_count = target_data.len();
        let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            target_count,
        )));
        services.wlock(self_id).insert(receiver.clone());
        let producer = Arc::new(RwLock::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.write().unwrap().run().unwrap();
        let receiver_handle = receiver.write().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.write().unwrap().run().unwrap();
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.write().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.read().unwrap().exit();
        task.read().unwrap().exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue.read().unwrap().exit();
        multi_queue_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let sent = producer.read().unwrap().sent().read().unwrap().len();
        let result = receiver.read().unwrap().received().read().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in receiver.read().unwrap().received().read().unwrap().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        let target_name = "/App/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.read().unwrap().received().read().unwrap().iter() {
            let (_, target) = target_data.pop().unwrap();
            assert!(result.value() == target, "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
    ///
    /// Testing Task function 'Export' with default 'enable' = true
    #[test]
    fn export_point_without_enable() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "App";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(
            Some("assets/testing/retain/"),
            Some(RetainPointConf::new("point/id.json", None))
        ))));
        let config = TaskConfig::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RecorderTask:
                    cycle: 1 ms
                    in queue recv-queue:
                        max-length: 10000
                    subscribe:
                        /App/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf
                    fn Debug debug01:
                        input fn Export:
                            enable: const bool true
                            conf point Load001:
                                type: 'Real'
                            input: point real '/App/Load'
                            send-to: /App/MultiQueue.in-queue
                    fn Debug debug02:
                        input fn Export:
                            conf point Load002:
                                type: 'Real'
                            input: point real '/App/RecorderTask/Load001'
                            send-to: /App/TaskTestReceiver.in-queue
                    ").unwrap(),
        );
        trace!("config: {:?}", config);
        debug!("Task config points: {:#?}", config.points());

        let task = Arc::new(RwLock::new(Task::new(config, services.clone())));
        debug!("Task points: {:#?}", task.read().unwrap().points());

        services.wlock(self_id).insert(task.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
            ").unwrap(),
        );
        let multi_queue = Arc::new(RwLock::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", self_id), Value::Real(-7.035)),
            (format!("/{}/Load", self_id), Value::Real(-2.5)),
            (format!("/{}/Load", self_id), Value::Real(-5.5)),
            (format!("/{}/Load", self_id), Value::Real(-1.5)),
            (format!("/{}/Load", self_id), Value::Real(-1.0)),
            (format!("/{}/Load", self_id), Value::Real(-0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(1.0)),
            (format!("/{}/Load", self_id), Value::Real(1.5)),
            (format!("/{}/Load", self_id), Value::Real(5.5)),
            (format!("/{}/Load", self_id), Value::Real(2.5)),
            (format!("/{}/Load", self_id), Value::Real(7.035)),
        ];
        let total_count = test_data.len();
        let mut target_data = test_data.clone();
        let target_count = target_data.len();
        let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            target_count,
        )));
        services.wlock(self_id).insert(receiver.clone());      // "TaskTestReceiver",
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(RwLock::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::ZERO,
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.write().unwrap().run().unwrap();
        let receiver_handle = receiver.write().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.write().unwrap().run().unwrap();
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.write().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        producer.read().unwrap().exit();
        task.read().unwrap().exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue.read().unwrap().exit();
        multi_queue_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let sent = producer.read().unwrap().sent().read().unwrap().len();
        let result = receiver.read().unwrap().received().read().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in receiver.read().unwrap().received().read().unwrap().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        let target_name = "/App/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.read().unwrap().received().read().unwrap().iter() {
            let (_, target) = target_data.pop().unwrap();
            assert!(result.value() == target, "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
    ///
    /// Testing Task function 'Export' with default 'enable' = true
    #[test]
    fn export_unconfigured_point() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "App";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(
            Some("assets/testing/retain/"),
            Some(RetainPointConf::new("point/id.json", None))
        ))));
        let config = TaskConfig::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RecorderTask:
                    cycle: 1 ms
                    in queue recv-queue:
                        max-length: 10000
                    subscribe:
                        /App/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf
                    fn Debug debug01:
                        input fn Export:
                            enable: const bool true
                            conf point Load001:
                                type: 'Real'
                            input: point real '/App/Load'
                            send-to: /App/MultiQueue.in-queue
                    fn Debug debug02:
                        input fn Export:
                            input: point real '/App/RecorderTask/Load001'
                            send-to: /App/TaskTestReceiver.in-queue
                    ").unwrap(),
        );
        trace!("config: {:?}", config);
        debug!("Task config points: {:#?}", config.points());

        let task = Arc::new(RwLock::new(Task::new(config, services.clone())));
        debug!("Task points: {:#?}", task.read().unwrap().points());

        services.wlock(self_id).insert(task.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
            ").unwrap(),
        );
        let multi_queue = Arc::new(RwLock::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", self_id), Value::Real(-7.035)),
            (format!("/{}/Load", self_id), Value::Real(-2.5)),
            (format!("/{}/Load", self_id), Value::Real(-5.5)),
            (format!("/{}/Load", self_id), Value::Real(-1.5)),
            (format!("/{}/Load", self_id), Value::Real(-1.0)),
            (format!("/{}/Load", self_id), Value::Real(-0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(1.0)),
            (format!("/{}/Load", self_id), Value::Real(1.5)),
            (format!("/{}/Load", self_id), Value::Real(5.5)),
            (format!("/{}/Load", self_id), Value::Real(2.5)),
            (format!("/{}/Load", self_id), Value::Real(7.035)),
        ];
        let total_count = test_data.len();
        let mut target_data = test_data.clone();
        let target_count = target_data.len();
        let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            target_count,
        )));
        services.wlock(self_id).insert(receiver.clone());      // "TaskTestReceiver",
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(RwLock::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::ZERO,
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.write().unwrap().run().unwrap();
        let receiver_handle = receiver.write().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.write().unwrap().run().unwrap();
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.write().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        producer.read().unwrap().exit();
        task.read().unwrap().exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue.read().unwrap().exit();
        multi_queue_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let sent = producer.read().unwrap().sent().read().unwrap().len();
        let result = receiver.read().unwrap().received().read().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in receiver.read().unwrap().received().read().unwrap().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        let target_name = "/App/RecorderTask/Load001";
        target_data.reverse();
        for result in receiver.read().unwrap().received().read().unwrap().iter() {
            let (_, target) = target_data.pop().unwrap();
            assert!(result.value() == target, "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
}

