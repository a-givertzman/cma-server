#[cfg(test)]

mod fn_retain {
    use chrono::Utc;
    use sal_sync::{math::AproxEq, services::{
        conf::{ConfTree, ServicesConf}, entity::{Cot, Name, Point, PointConfType, PointHlr, Status}, types::Bool, MultiQueue, MultiQueueConf, Service, Services
    }, thread_pool::ThreadPool};
    use std::{env, fs, io::Read, sync::{Arc, Once}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        services::task::{Task, TaskConf, TaskTestReceiver},
        tests::unit::services::task::task_test_producer::TaskTestProducer
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
    /// Loads retained Point value from the disk
    fn load(self_id: &str, path: &str, type_: PointConfType) -> Option<Point> {
        let tx_id = 10001;
        match fs::OpenOptions::new().read(true).open(&path) {
            Ok(mut f) => {
                let mut input = String::new();
                match f.read_to_string(&mut input) {
                    Ok(_) => {
                        match type_ {
                            PointConfType::Bool => match input.as_str() {
                                "true" => Some(Point::Bool(PointHlr::new(tx_id, &self_id, Bool(true), Status::Ok, Cot::Inf, Utc::now()))),
                                "false" => Some(Point::Bool(PointHlr::new(tx_id, &self_id, Bool(false), Status::Ok, Cot::Inf, Utc::now()))),
                                _ => {
                                    log::error!("{}.load | Error parse 'bool' from '{}' \n\tretain: '{:?}'", self_id, input, path);
                                    None
                                }
                            }
                            PointConfType::Int => match input.as_str().parse() {
                                Ok(value) => {
                                    Some(Point::Int(PointHlr::new(tx_id, &self_id, value, Status::Ok, Cot::Inf, Utc::now())))
                                }
                                Err(err) => {
                                    log::error!("{}.load | Error parse 'Int' from '{}' \n\tretain: '{:?}'\n\terror: {:?}", self_id, input, path, err);
                                    None
                                }
                            }
                            PointConfType::Real => match input.as_str().parse() {
                                Ok(value) => {
                                    Some(Point::Real(PointHlr::new(tx_id, &self_id, value, Status::Ok, Cot::Inf, Utc::now())))
                                }
                                Err(err) => {
                                    log::error!("{}.load | Error parse 'Real' from '{}' \n\tretain: '{:?}'\n\terror: {:?}", self_id, input, path, err);
                                    None
                                }
                            }
                            PointConfType::Double => match input.as_str().parse() {
                                Ok(value) => {
                                    Some(Point::Double(PointHlr::new(tx_id, &self_id, value, Status::Ok, Cot::Inf, Utc::now())))
                                }
                                Err(err) => {
                                    log::error!("{}.load | Error parse 'Double' from '{}' \n\tretain: '{:?}'\n\terror: {:?}", self_id, input, path, err);
                                    None
                                }
                            }
                            PointConfType::String => {
                                Some(Point::String(PointHlr::new(tx_id, &self_id, input, Status::Ok, Cot::Inf, Utc::now())))
                            }
                            PointConfType::Json => {
                                Some(Point::String(PointHlr::new(tx_id, &self_id, input, Status::Ok, Cot::Inf, Utc::now())))
                            }
                        }

                    }
                    Err(err) => {
                        log::warn!("{}.load | Error read from retain: '{:?}'\n\terror: {:?}", self_id, path, err);
                        None
                    }
                }
            }
            Err(err) => {
                log::warn!("{}.load | Error open file: '{:?}'\n\terror: {:?}", self_id, path, err);
                None
            }
        }
    }
    ///
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Testing Task function 'Retain' for int value
    #[test]
    fn retain_point_bool() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbg = "AppTest";
        let self_name = Name::new("", dbg);
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        log::trace!("dir: {:?}", env::current_dir());
        let initial = load(dbg, &format!("./assets/testing/retain/{}/RetainTask/BoolFlag.json", dbg), PointConfType::Bool)
            .map_or(false, |init| init.as_bool().value.0);
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json
                    api:
                        table: public.tags
                        address: 0.0.0.0:8080
                        auth_token: 123!@#
                        database: crane_data_server
            "#).unwrap()),
        ), Some(tp.scheduler())));
        let config = TaskConf::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RetainTask:
                    cycle: 1 ms
                    in queue in-queue:
                        max-length: 10000
                    subscribe:
                        /AppTest/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf
                    
                    fn Debug debug01:
                        input1 fn Export:
                            send-to: /AppTest/TaskTestReceiver.in-queue
                            input fn Retain:
                                default: const bool false
                                key: 'BoolFlag'
                        input2 fn Retain:
                            key: 'BoolFlag'
                            input: point bool '/AppTest/BoolFlag'
            ").unwrap(),
        );
        log::trace!("config: {:?}", config);
        log::debug!("Task config points: {:#?}", config.points());
        let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
        log::debug!("Task points: {:#?}", task.points());
        services.insert(task.clone());
        let conf = MultiQueueConf::from_yaml(
            dbg,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                # send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(MultiQueue::new(conf, services.clone(), Some(tp.scheduler())));
        services.insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/BoolFlag", dbg), Value::Bool(!initial)),
            (format!("/{}/BoolFlag", dbg), Value::Bool(initial)),
            (format!("/{}/BoolFlag", dbg), Value::Bool(!initial)),
            (format!("/{}/BoolFlag", dbg), Value::Bool(initial)),
            (format!("/{}/BoolFlag", dbg), Value::Bool(!initial)),
        ];
        let total_count = test_data.len();
        let mut target_data = vec![
            Value::Bool(initial),
            Value::Bool(initial),
            Value::Bool(initial),
            Value::Bool(initial),
            Value::Bool(initial),
        ];
        let target_count = target_data.len();
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "in-queue",
            target_count,
        ));
        services.insert(receiver.clone());      // "TaskTestReceiver",
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            &format!("/{}/MultiQueue.in-queue", dbg),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        ));
        services.insert(producer.clone());
        services.run().unwrap();
        multi_queue.run().unwrap();
        receiver.run().unwrap();
        log::info!("receiver runing - ok");
        task.run().unwrap();
        log::info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        producer.run().unwrap();
        log::info!("producer runing - ok");
        let time = Instant::now();
        receiver.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.exit();
        task.exit();
        task.wait().unwrap();
        producer.wait().unwrap();
        multi_queue.exit();
        multi_queue.wait().unwrap();
        services.exit();
        services.wait().unwrap();
        let sent = producer.sent().len();
        let result = receiver.received().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in target_data.iter().enumerate() {
            println!("target {}: {:?}", i, point)
        }
        for (i, point) in receiver.received().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        // let target_name = "/AppTest/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.received() {
            let target = target_data.pop().unwrap();
            assert!(result.value() == target, "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            // assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
    ///
    /// Testing Task function 'Retain' for int value
    #[test]
    fn retain_point_int() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbg = "AppTest";
        let self_name = Name::new("", dbg);
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        log::trace!("dir: {:?}", env::current_dir());
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json
                    api:
                        table: public.tags
                        address: 0.0.0.0:8080
                        auth_token: 123!@#
                        database: crane_data_server
            "#).unwrap()),
        ), Some(tp.scheduler())));
        let config = TaskConf::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RetainTask:
                    cycle: 1 ms
                    in queue in-queue:
                        max-length: 10000
                    subscribe:
                        /AppTest/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf
                    
                    fn Debug debug01:
                        input fn Export:
                            send-to: /AppTest/TaskTestReceiver.in-queue
                            input fn Retain:
                                key: 'Count'
                                input fn Count:
                                    initial fn Retain:
                                        default: const int 0
                                        key: 'Count'
                                    input fn Ge:
                                        input1: point real '/AppTest/Load'
                                        input2: const real 0.1
            ").unwrap(),
        );
        log::trace!("config: {:?}", config);
        log::debug!("Task config points: {:#?}", config.points());

        let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
        log::debug!("Task points: {:#?}", task.points());

        services.insert(task.clone());
        let conf = MultiQueueConf::from_yaml(
            dbg,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                # send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(MultiQueue::new(conf, services.clone(), Some(tp.scheduler())));
        services.insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", dbg), Value::Real(0.0)),
            (format!("/{}/Load", dbg), Value::Real(1.5)),
            (format!("/{}/Load", dbg), Value::Real(0.0)),
            (format!("/{}/Load", dbg), Value::Real(1.5)),
            (format!("/{}/Load", dbg), Value::Real(1.0)),
            (format!("/{}/Load", dbg), Value::Real(0.0)),
            (format!("/{}/Load", dbg), Value::Real(0.7)),
            (format!("/{}/Load", dbg), Value::Real(0.0)),
            (format!("/{}/Load", dbg), Value::Real(1.5)),
            (format!("/{}/Load", dbg), Value::Real(0.0)),
        ];
        let total_count = test_data.len();
        let initial = load(dbg, &format!("./assets/testing/retain/{}/RetainTask/Count.json", dbg), PointConfType::Int)
            .map_or(0, |init| init.as_int().value);
        let mut target_data = vec![
            Value::Int(initial + 0),
            Value::Int(initial + 1),
            Value::Int(initial + 1),
            Value::Int(initial + 2),
            Value::Int(initial + 2),
            Value::Int(initial + 2),
            Value::Int(initial + 3),
            Value::Int(initial + 3),
            Value::Int(initial + 4),
            Value::Int(initial + 4),
        ];
        let target_count = target_data.len();
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "in-queue",
            target_count,
        ));
        services.insert(receiver.clone());      // "TaskTestReceiver",
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            &format!("/{}/MultiQueue.in-queue", dbg),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        ));
        services.insert(producer.clone());
        services.run().unwrap();
        multi_queue.run().unwrap();
        receiver.run().unwrap();
        log::info!("receiver runing - ok");
        task.run().unwrap();
        log::info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        producer.run().unwrap();
        log::info!("producer runing - ok");
        let time = Instant::now();
        receiver.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.exit();
        task.exit();
        task.wait().unwrap();
        producer.wait().unwrap();
        multi_queue.exit();
        multi_queue.wait().unwrap();
        services.exit();
        services.wait().unwrap();
        let sent = producer.sent().len();
        let result = receiver.received().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in target_data.iter().enumerate() {
            println!("target {}: {:?}", i, point)
        }
        for (i, point) in receiver.received().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        // let target_name = "/AppTest/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.received() {
            let target = target_data.pop().unwrap();
            assert!(result.value() == target, "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            // assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
    ///
    /// Testing Task function 'Retain' for real value
    #[test]
    fn retain_point_real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        // #[derive(Copy, Clone, Eq, PartialEq)]
        // struct T(());
        // let uid = uid::Id::<T>::new();
        let dbg = &format!("AppTest");
        let self_name = Name::new("", dbg);
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        log::trace!("dir: {:?}", env::current_dir());
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json
                    api:
                        table: public.tags
                        address: 0.0.0.0:8080
                        auth_token: 123!@#
                        database: crane_data_server
            "#).unwrap()),
        ), Some(tp.scheduler())));
        let config = TaskConf::from_yaml(
            &self_name,
            &serde_yaml::from_str(&format!(r"
                service Task RetainTask:
                    cycle: 1 ms
                    in queue in-queue:
                        max-length: 10000
                    subscribe:
                        /{}/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {{cot: Inf}}: []                      #   - on all points having Cot::Inf
                    
                    let realRetain:
                        input fn Retain:
                            default: const real 0.0
                            key: 'RealRetain'
                    fn Debug:
                        in1: realRetain
                        in2 fn Retain:
                            key: 'RealRetain'
                            input fn Export:
                                send-to: '/{}/TaskTestReceiver.in-queue'
                                input fn Add:
                                    input1: realRetain
                                    input2: point real '/{}/Load'
            ", dbg, dbg, dbg)).unwrap(),
        );
        log::trace!("config: {:?}", config);
        log::debug!("Task config points: {:#?}", config.points());

        let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
        log::debug!("Task points: {:#?}", task.points());

        services.insert(task.clone());
        let conf = MultiQueueConf::from_yaml(
            dbg,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                # send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(MultiQueue::new(conf, services.clone(), Some(tp.scheduler())));
        services.insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.2)),
            (format!("/{}/Load", dbg), Value::Real(0.3)),
            (format!("/{}/Load", dbg), Value::Real(0.4)),
            (format!("/{}/Load", dbg), Value::Real(0.5)),
            (format!("/{}/Load", dbg), Value::Real(0.6)),
            (format!("/{}/Load", dbg), Value::Real(0.7)),
            (format!("/{}/Load", dbg), Value::Real(0.8)),
            (format!("/{}/Load", dbg), Value::Real(0.9)),
            (format!("/{}/Load", dbg), Value::Real(1.0)),
            (format!("/{}/Load", dbg), Value::Real(1.1)),
        ];
        let total_count = test_data.len();
        let initial = load(dbg, &format!("./assets/testing/retain/{}/RetainTask/RealRetain.json", dbg), PointConfType::Real)
            .map_or(0.0, |init| init.as_real().value);
        let mut target_data = vec![
            Value::Real(initial + 0.1),
            Value::Real(initial + 0.2),
            Value::Real(initial + 0.3),
            Value::Real(initial + 0.4),
            Value::Real(initial + 0.5),
            Value::Real(initial + 0.6),
            Value::Real(initial + 0.7),
            Value::Real(initial + 0.8),
            Value::Real(initial + 0.9),
            Value::Real(initial + 1.0),
            Value::Real(initial + 1.1),
        ];
        let target_count = target_data.len();
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "in-queue",
            target_count,
        ));
        services.insert(receiver.clone());      // "TaskTestReceiver",
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            &format!("/{}/MultiQueue.in-queue", dbg),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        ));
        services.insert(producer.clone());
        services.run().unwrap();
        multi_queue.run().unwrap();
        receiver.run().unwrap();
        log::info!("receiver runing - ok");
        task.run().unwrap();
        log::info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        producer.run().unwrap();
        log::info!("producer runing - ok");
        let time = Instant::now();
        receiver.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.exit();
        task.exit();
        task.wait().unwrap();
        producer.wait().unwrap();
        multi_queue.exit();
        multi_queue.wait().unwrap();
        services.exit();
        services.wait().unwrap();
        let sent = producer.sent().len();
        let result = receiver.received().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in target_data.iter().enumerate() {
            println!("target {}: {:?}", i, point)
        }
        for (i, point) in receiver.received().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        // let target_name = "/AppTest/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.received() {
            let target = target_data.pop().unwrap();
            assert!(result.value() == target, "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            // assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
    ///
    /// Testing Task function 'Retain' for real value
    ///  - using [every-cycle] = true
    #[test]
    fn retain_every_cycle_point_real() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let dbg = "AppTest";
        let self_name = Name::new("", dbg);
        println!("\n{}", dbg);
        let test_duration = TestDuration::new(dbg, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        log::trace!("dir: {:?}", env::current_dir());
        let initial = load(dbg, &format!("./assets/testing/retain/{}/RetainTask/RealRetainEveryCycle.json", dbg), PointConfType::Real)
            .map_or(0.0, |init| init.as_real().value);
        let tp = ThreadPool::new(dbg, Some(8));
        let services = Arc::new(Services::new(dbg, ServicesConf::new(
            dbg, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json
                    api:
                        table: public.tags
                        address: 0.0.0.0:8080
                        auth_token: 123!@#
                        database: crane_data_server
            "#).unwrap()),
        ), Some(tp.scheduler())));
        let config = TaskConf::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RetainTask:
                    cycle: 1 ms
                    in queue in-queue:
                        max-length: 10000
                    subscribe:
                        /AppTest/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf
                    
                    fn Debug:
                        in fn Retain RetainStore:
                            key: 'RealRetainEveryCycle'
                            input fn Export:
                                conf point Retained.Point:
                                    type: real
                                send-to: /AppTest/TaskTestReceiver.in-queue
                                input fn Add:
                                    input1 fn Retain RetainLoad:
                                        key: 'RealRetainEveryCycle'
                                        every-cycle: true
                                        default: const real 0.0
                                    input2: point real '/AppTest/Load'
            ").unwrap(),
        );
        log::trace!("config: {:?}", config);
        log::debug!("Task config points: {:#?}", config.points());
        let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
        log::debug!("Task points: {:#?}", task.points());
        services.insert(task.clone());
        let conf = MultiQueueConf::from_yaml(
            dbg,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                # send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(MultiQueue::new(conf, services.clone(), Some(tp.scheduler())));
        services.insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
            (format!("/{}/Load", dbg), Value::Real(0.1)),
        ];
        let total_count = test_data.len();
        let mut target_data = vec![
            Value::Real(initial + 0.1),
            Value::Real(initial + 0.2),
            Value::Real(initial + 0.3),
            Value::Real(initial + 0.4),
            Value::Real(initial + 0.5),
            Value::Real(initial + 0.6),
            Value::Real(initial + 0.7),
            Value::Real(initial + 0.8),
            Value::Real(initial + 0.9),
            Value::Real(initial + 1.0),
            Value::Real(initial + 1.1),
        ];
        let target_count = target_data.len();
        let receiver = Arc::new(TaskTestReceiver::new(
            dbg,
            "",
            "in-queue",
            target_count,
        ));
        services.insert(receiver.clone());
        thread::sleep(Duration::from_millis(100));
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(TaskTestProducer::new(
            dbg,
            &format!("/{}/MultiQueue.in-queue", dbg),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        ));
        services.insert(producer.clone());
        thread::sleep(Duration::from_millis(100));
        services.run().unwrap();
        multi_queue.run().unwrap();
        receiver.run().unwrap();
        log::info!("receiver runing - ok");
        task.run().unwrap();
        log::info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        producer.run().unwrap();
        log::info!("producer runing - ok");
        let time = Instant::now();
        receiver.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.exit();
        task.exit();
        task.wait().unwrap();
        producer.wait().unwrap();
        multi_queue.exit();
        multi_queue.wait().unwrap();
        services.exit();
        services.wait().unwrap();
        let sent = producer.sent().len();
        let result = receiver.received().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in target_data.iter().enumerate() {
            println!("target {}: {:?}", i, point)
        }
        for (i, point) in receiver.received().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        // let target_name = "/AppTest/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.received() {
            let target = target_data.pop().unwrap();
            assert!(result.value().aprox_eq(&target, 3), "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            // assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
}

