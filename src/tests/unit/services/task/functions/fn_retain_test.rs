#[cfg(test)]

mod fn_retain {
    use chrono::Utc;
    use log::{debug, error, info, trace, warn};
    use sal_sync::services::{entity::{cot::Cot, name::Name, point::{point::Point, point_config_type::PointConfigType, point_hlr::PointHlr}, status::status::Status}, retain::retain_point_conf::{RetainPointConf, RetainPointConfApi}, service::service::Service, types::bool::Bool};
    use std::{env, fs, io::Read, sync::{Arc, Mutex, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{multi_queue_config::MultiQueueConfig, task_config::TaskConfig},
        core_::aprox_eq::aprox_eq::AproxEq,
        services::{
            multi_queue::multi_queue::MultiQueue, safe_lock::SafeLock, services::Services,
            task::{task::Task, task_test_receiver::TaskTestReceiver}
        },
        tests::unit::services::task::task_test_producer::TaskTestProducer,
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
    fn load(self_id: &str, path: &str, type_: PointConfigType) -> Option<Point> {
        let tx_id = 10001;
        match fs::OpenOptions::new().read(true).open(&path) {
            Ok(mut f) => {
                let mut input = String::new();
                match f.read_to_string(&mut input) {
                    Ok(_) => {
                        match type_ {
                            PointConfigType::Bool => match input.as_str() {
                                "true" => Some(Point::Bool(PointHlr::new(tx_id, &self_id, Bool(true), Status::Ok, Cot::Inf, Utc::now()))),
                                "false" => Some(Point::Bool(PointHlr::new(tx_id, &self_id, Bool(false), Status::Ok, Cot::Inf, Utc::now()))),
                                _ => {
                                    error!("{}.load | Error parse 'bool' from '{}' \n\tretain: '{:?}'", self_id, input, path);
                                    None
                                }
                            }
                            PointConfigType::Int => match input.as_str().parse() {
                                Ok(value) => {
                                    Some(Point::Int(PointHlr::new(tx_id, &self_id, value, Status::Ok, Cot::Inf, Utc::now())))
                                }
                                Err(err) => {
                                    error!("{}.load | Error parse 'Int' from '{}' \n\tretain: '{:?}'\n\terror: {:?}", self_id, input, path, err);
                                    None
                                }
                            }
                            PointConfigType::Real => match input.as_str().parse() {
                                Ok(value) => {
                                    Some(Point::Real(PointHlr::new(tx_id, &self_id, value, Status::Ok, Cot::Inf, Utc::now())))
                                }
                                Err(err) => {
                                    error!("{}.load | Error parse 'Real' from '{}' \n\tretain: '{:?}'\n\terror: {:?}", self_id, input, path, err);
                                    None
                                }
                            }
                            PointConfigType::Double => match input.as_str().parse() {
                                Ok(value) => {
                                    Some(Point::Double(PointHlr::new(tx_id, &self_id, value, Status::Ok, Cot::Inf, Utc::now())))
                                }
                                Err(err) => {
                                    error!("{}.load | Error parse 'Double' from '{}' \n\tretain: '{:?}'\n\terror: {:?}", self_id, input, path, err);
                                    None
                                }
                            }
                            PointConfigType::String => {
                                Some(Point::String(PointHlr::new(tx_id, &self_id, input, Status::Ok, Cot::Inf, Utc::now())))
                            }
                            PointConfigType::Json => {
                                Some(Point::String(PointHlr::new(tx_id, &self_id, input, Status::Ok, Cot::Inf, Utc::now())))
                            }
                        }

                    }
                    Err(err) => {
                        warn!("{}.load | Error read from retain: '{:?}'\n\terror: {:?}", self_id, path, err);
                        None
                    }
                }
            }
            Err(err) => {
                warn!("{}.load | Error open file: '{:?}'\n\terror: {:?}", self_id, path, err);
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
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "AppTest";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let initial = load(self_id, &format!("./assets/testing/retain/{}/RetainTask/BoolFlag.json", self_id), PointConfigType::Bool)
            .map_or(false, |init| init.as_bool().value.0);
        let services = Arc::new(RwLock::new(Services::new(
            self_id,
            Some(RetainPointConf::new(
                "assets/testing/retain/point/id.json", 
                Some(RetainPointConfApi::new("public.tags", "0.0.0.0:8080", "123!@#", "crane_data_server"))
            )),
        )));
        let config = TaskConfig::from_yaml(
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
        trace!("config: {:?}", config);
        debug!("Task config points: {:#?}", config.points());
        let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
        debug!("Task points: {:#?}", task.lock().unwrap().points());
        services.wlock(self_id).insert(task.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(Mutex::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/BoolFlag", self_id), Value::Bool(!initial)),
            (format!("/{}/BoolFlag", self_id), Value::Bool(initial)),
            (format!("/{}/BoolFlag", self_id), Value::Bool(!initial)),
            (format!("/{}/BoolFlag", self_id), Value::Bool(initial)),
            (format!("/{}/BoolFlag", self_id), Value::Bool(!initial)),
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
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            target_count,
        )));
        services.wlock(self_id).insert(receiver.clone());      // "TaskTestReceiver",
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(Mutex::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.lock().unwrap().run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.lock().unwrap().run().unwrap();
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.lock().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.lock().unwrap().exit();
        task.lock().unwrap().exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue.lock().unwrap().exit();
        multi_queue_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let sent = producer.lock().unwrap().sent().lock().unwrap().len();
        let result = receiver.lock().unwrap().received().lock().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in target_data.iter().enumerate() {
            println!("target {}: {:?}", i, point)
        }
        for (i, point) in receiver.lock().unwrap().received().lock().unwrap().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        // let target_name = "/AppTest/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.lock().unwrap().received().lock().unwrap().iter() {
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
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "AppTest";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let services = Arc::new(RwLock::new(Services::new(
            self_id,
            Some(RetainPointConf::new(
                "assets/testing/retain/point/id.json", 
                Some(RetainPointConfApi::new("public.tags", "0.0.0.0:8080", "123!@#", "crane_data_server"))
            )),
        )));
        let config = TaskConfig::from_yaml(
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
        trace!("config: {:?}", config);
        debug!("Task config points: {:#?}", config.points());

        let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
        debug!("Task points: {:#?}", task.lock().unwrap().points());

        services.wlock(self_id).insert(task.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(Mutex::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", self_id), Value::Real(0.0)),
            (format!("/{}/Load", self_id), Value::Real(1.5)),
            (format!("/{}/Load", self_id), Value::Real(0.0)),
            (format!("/{}/Load", self_id), Value::Real(1.5)),
            (format!("/{}/Load", self_id), Value::Real(1.0)),
            (format!("/{}/Load", self_id), Value::Real(0.0)),
            (format!("/{}/Load", self_id), Value::Real(0.7)),
            (format!("/{}/Load", self_id), Value::Real(0.0)),
            (format!("/{}/Load", self_id), Value::Real(1.5)),
            (format!("/{}/Load", self_id), Value::Real(0.0)),
        ];
        let total_count = test_data.len();
        let initial = load(self_id, &format!("./assets/testing/retain/{}/RetainTask/Count.json", self_id), PointConfigType::Int)
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
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            target_count,
        )));
        services.wlock(self_id).insert(receiver.clone());      // "TaskTestReceiver",
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(Mutex::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.lock().unwrap().run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.lock().unwrap().run().unwrap();
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.lock().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.lock().unwrap().exit();
        task.lock().unwrap().exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue.lock().unwrap().exit();
        multi_queue_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let sent = producer.lock().unwrap().sent().lock().unwrap().len();
        let result = receiver.lock().unwrap().received().lock().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in target_data.iter().enumerate() {
            println!("target {}: {:?}", i, point)
        }
        for (i, point) in receiver.lock().unwrap().received().lock().unwrap().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        // let target_name = "/AppTest/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.lock().unwrap().received().lock().unwrap().iter() {
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
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "AppTest";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let services = Arc::new(RwLock::new(Services::new(
            self_id,
            Some(RetainPointConf::new(
                "assets/testing/retain/point/id.json", 
                Some(RetainPointConfApi::new("public.tags", "0.0.0.0:8080", "123!@#", "crane_data_server"))
            )),
        )));
        let config = TaskConfig::from_yaml(
            &self_name,
            &serde_yaml::from_str(r"
                service Task RetainTask:
                    cycle: 1 ms
                    in queue in-queue:
                        max-length: 10000
                    subscribe:
                        /AppTest/MultiQueue:                    # - multicast subscription to the MultiQueue
                            {cot: Inf}: []                      #   - on all points having Cot::Inf
                    
                    let realRetain:
                        input fn Retain:
                            default: const real 0.0
                            key: 'RealRetain'
                    fn Debug:
                        in1: realRetain
                        in2 fn Retain:
                            key: 'RealRetain'
                            input fn Export:
                                send-to: /AppTest/TaskTestReceiver.in-queue
                                input fn Add:
                                    input1: realRetain
                                    input2: point real '/AppTest/Load'
            ").unwrap(),
        );
        trace!("config: {:?}", config);
        debug!("Task config points: {:#?}", config.points());

        let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
        debug!("Task points: {:#?}", task.lock().unwrap().points());

        services.wlock(self_id).insert(task.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(Mutex::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.2)),
            (format!("/{}/Load", self_id), Value::Real(0.3)),
            (format!("/{}/Load", self_id), Value::Real(0.4)),
            (format!("/{}/Load", self_id), Value::Real(0.5)),
            (format!("/{}/Load", self_id), Value::Real(0.6)),
            (format!("/{}/Load", self_id), Value::Real(0.7)),
            (format!("/{}/Load", self_id), Value::Real(0.8)),
            (format!("/{}/Load", self_id), Value::Real(0.9)),
            (format!("/{}/Load", self_id), Value::Real(1.0)),
            (format!("/{}/Load", self_id), Value::Real(1.1)),
        ];
        let total_count = test_data.len();
        let initial = load(self_id, &format!("./assets/testing/retain/{}/RetainTask/RealRetain.json", self_id), PointConfigType::Real)
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
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            target_count,
        )));
        services.wlock(self_id).insert(receiver.clone());      // "TaskTestReceiver",
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(Mutex::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.lock().unwrap().run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.lock().unwrap().run().unwrap();
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.lock().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.lock().unwrap().exit();
        task.lock().unwrap().exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue.lock().unwrap().exit();
        multi_queue_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let sent = producer.lock().unwrap().sent().lock().unwrap().len();
        let result = receiver.lock().unwrap().received().lock().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in target_data.iter().enumerate() {
            println!("target {}: {:?}", i, point)
        }
        for (i, point) in receiver.lock().unwrap().received().lock().unwrap().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        // let target_name = "/AppTest/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.lock().unwrap().received().lock().unwrap().iter() {
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
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "AppTest";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(20));
        test_duration.run().unwrap();
        //
        // can be changed
        trace!("dir: {:?}", env::current_dir());
        let initial = load(self_id, &format!("./assets/testing/retain/{}/RetainTask/RealRetainEveryCycle.json", self_id), PointConfigType::Real)
            .map_or(0.0, |init| init.as_real().value);
        let services = Arc::new(RwLock::new(Services::new(
            self_id,
            Some(RetainPointConf::new(
                "assets/testing/retain/point/id.json", 
                Some(RetainPointConfApi::new("public.tags", "0.0.0.0:8080", "123!@#", "crane_data_server"))
            )),
        )));
        let config = TaskConfig::from_yaml(
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
        trace!("config: {:?}", config);
        debug!("Task config points: {:#?}", config.points());
        let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
        debug!("Task points: {:#?}", task.lock().unwrap().points());
        services.wlock(self_id).insert(task.clone());
        let conf = MultiQueueConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service MultiQueue:
                in queue in-queue:
                    max-length: 10000
                # send-to:
            ").unwrap(),
        );
        let multi_queue = Arc::new(Mutex::new(MultiQueue::new(conf, services.clone())));
        services.wlock(self_id).insert(multi_queue.clone());
        let test_data = vec![
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
            (format!("/{}/Load", self_id), Value::Real(0.1)),
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
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            target_count,
        )));
        services.wlock(self_id).insert(receiver.clone());
        thread::sleep(Duration::from_millis(100));
        // assert!(total_count == iterations, "\nresult: {:?}\ntarget: {:?}", total_count, iterations);
        let producer = Arc::new(Mutex::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        thread::sleep(Duration::from_millis(100));
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.lock().unwrap().run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        info!("receiver runing - ok");
        let task_handle = task.lock().unwrap().run().unwrap();
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(100));
        let producer_handle = producer.lock().unwrap().run().unwrap();
        info!("producer runing - ok");
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        thread::sleep(Duration::from_millis(100));
        producer.lock().unwrap().exit();
        task.lock().unwrap().exit();
        task_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue.lock().unwrap().exit();
        multi_queue_handle.wait().unwrap();
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        let sent = producer.lock().unwrap().sent().lock().unwrap().len();
        let result = receiver.lock().unwrap().received().lock().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        println!("trget: {:?}", target_count);
        for (i, point) in target_data.iter().enumerate() {
            println!("target {}: {:?}", i, point)
        }
        for (i, point) in receiver.lock().unwrap().received().lock().unwrap().iter().enumerate() {
            println!("received {}: {:?}", i, point)
        }
        assert!(sent == total_count, "\nresult: {:?}\ntarget: {:?}", sent, total_count);
        assert!(result == target_count, "\nresult: {:?}\ntarget: {:?}", result, target_count);
        // let target_name = "/AppTest/RecorderTask/Load002";
        target_data.reverse();
        for result in receiver.lock().unwrap().received().lock().unwrap().iter() {
            let target = target_data.pop().unwrap();
            assert!(result.value().aprox_eq(&target, 3), "\nresult: {:?}\ntarget: {:?}", result.value(), target);
            // assert!(result.name() == target_name, "\nresult: {:?}\ntarget: {:?}", result.name(), target_name);
        };
        test_duration.exit();
    }
}

