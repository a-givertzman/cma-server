#[cfg(test)]

mod cma_recorder {
    use log::{info, trace};
    use regex::Regex;
    use std::{env, fs, sync::{Arc, Mutex, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{api_client_config::ApiClientConfig, multi_queue_config::MultiQueueConfig, point_config::name::Name, task_config::TaskConfig},
        services::{
            api_cient::api_client::ApiClient, multi_queue::multi_queue::MultiQueue, safe_lock::SafeLock, service::service::Service, services::Services, task::{task::Task, task_test_receiver::TaskTestReceiver}
        },
        tests::unit::services::task::task_test_producer::TaskTestProducer,
        // tests::unit::services::cma_recorder::task_test_producer::TaskTestProducer
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
    /// Testing the complette configuration of the CMA Recorder
    #[test]
    fn test() {
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
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        let mut tasks = vec![];
        let mut task_handles = vec![];
        let path = "./src/tests/unit/services/task/cma_recorder/cma-recorder.yaml";
        match fs::read_to_string(path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        let config: serde_yaml::Value = config;
                        for (key, config) in config.as_mapping().unwrap() {
                            let mut conf = serde_yaml::Mapping::new();
                            conf.insert(key.clone(), config.clone());
                            let config = TaskConfig::from_yaml(&self_name, &serde_yaml::Value::Mapping(conf));
                            let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
                            services.wlock( self_id).insert(task.clone());
                            tasks.push(task);
                        }
                    }
                    Err(err) => panic!("{}.read | Error in config: {:?}\n\terror: {:?}", self_id, yaml_string, err),
                }
            }
            Err(err) => panic!("{}.read | File {} reading error: {:?}", self_id, path, err),
        }
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
        let conf = ApiClientConfig::from_yaml(
            self_id,
            &serde_yaml::from_str(r"service ApiClient:
                cycle: 100 ms
                reconnect: 1 s  # default 3 s
                address: 127.0.0.1:8080
                database: crane_data_server
                in queue in-queue:
                    max-length: 10000
                auth_token: 123!@#
                debug: true
            ").unwrap(),
        );
        let api_client = Arc::new(Mutex::new(ApiClient::new(conf)));
        services.wlock(self_id).insert(api_client.clone());
        let test_data = test_data(self_id);
        let total_count = test_data.len();
        let receiver = Arc::new(Mutex::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            total_count * 1000,
        )));
        services.wlock(self_id).insert(receiver.clone());
        let test_data: Vec<(String, Value)> = test_data.into_iter().map(|(_, name, value)| {
            (name, value)
        }).collect();
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
        let api_client_handle = api_client.lock().unwrap().run().unwrap();
        let receiver_handle = receiver.lock().unwrap().run().unwrap();
        info!("receiver runing - ok");
        for task in &tasks {
            let handle = task.lock().unwrap().run().unwrap();
            task_handles.push(handle);
        }
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(300));
        let producer_handle = producer.lock().unwrap().run().unwrap();
        info!("producer runing - ok");
        thread::sleep(Duration::from_millis(300));
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        producer.lock().unwrap().exit();
        multi_queue.lock().unwrap().exit();
        for task in tasks {
            task.lock().unwrap().exit();
        }
        services.rlock(self_id).exit();
        for handle in task_handles {
            handle.wait().unwrap();
        }
        api_client.lock().unwrap().exit();
        api_client_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        // exit_producer_handle.wait().unwrap();
        multi_queue_handle.wait().unwrap();
        services_handle.wait().unwrap();
        let sent = producer.lock().unwrap().sent().lock().unwrap().len();
        let result = receiver.lock().unwrap().received().lock().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}", result);
        for (i, result) in receiver.lock().unwrap().received().lock().unwrap().iter().enumerate() {
            println!("received: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };
        let targets = targets();
        let mut index = 0;
        for result in receiver.lock().unwrap().received().lock().unwrap().iter() {
            if result.name().starts_with("input34_1") {
                let name = result.name();
                let result = result.as_string().value;
                let target = targets[index];
                assert!(Regex::new(target).unwrap().is_match(&result), "index {}, name '{}' \nresult: {:?}\ntarget: {:?}", index, name, result, target);
                index += 1;
            }
            if result.name().starts_with("input34_2") {
                let name = result.name();
                let result = result.as_string().value;
                let target = targets[index];
                assert!(Regex::new(target).unwrap().is_match(&result), "index {}, name '{}' \nresult: {:?}\ntarget: {:?}", index, name, result, target);
                index += 1;
            }
            if result.name().starts_with("input34_3") {
                let name = result.name();
                let result = result.as_string().value;
                let target = targets[index];
                assert!(Regex::new(target).unwrap().is_match(&result), "index {}, name '{}' \nresult: {:?}\ntarget: {:?}", index, name, result, target);
                index += 1;
            }
            if result.name().starts_with("input34_4") {
                let name = result.name();
                let result = result.as_string().value;
                let target = targets[index];
                assert!(Regex::new(target).unwrap().is_match(&result), "index {}, name '{}' \nresult: {:?}\ntarget: {:?}", index, name, result, target);
                index += 1;
            }
        };
        assert!(index == targets.len(), "result: {:?}\ntarget: {:?}", index, targets.len());
        test_duration.exit();
        // loop {
        //     thread::sleep(Duration::from_millis(100));
        // }
    }
    ///
    /// Returns test data
    fn test_data<'a>(self_id: &str) -> Vec<(&'a str, String, Value)> {
        vec![
            //  step        name                                input
                ("00.0",    format!("/{}/Load.Nom", self_id),   Value::Real(  150.0)),
                ("00.1",    format!("/{}/Winch1.Load.Nom", self_id),   Value::Real(  150.00)),
                ("00.2",    format!("/{}/Winch2.Load.Nom", self_id),   Value::Real(  150.00)),
                ("00.3",    format!("/{}/Winch3.Load.Nom", self_id),   Value::Real(  150.00)),
                //  Crane 0.05 - 0.15
                ("01.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("01.1",    format!("/{}/Load", self_id),       Value::Real(  7.50)),
                ("01.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("01.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("02.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("02.1",    format!("/{}/Load", self_id),       Value::Real( 22.49)),
                ("02.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("02.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.15 - 0.25
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real( 22.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real( 37.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.25 - 0.35
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real( 37.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real( 52.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.35 - 0.45
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real( 52.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real( 67.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.45 - 0.55
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real( 67.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real( 82.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.55 - 0.65
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real( 82.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real( 97.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.65 - 0.75
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real( 97.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real(112.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.75 - 0.85
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real(112.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real(127.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.85 - 0.95
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real(127.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real(142.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 0.95 - 1.05
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real(142.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real(157.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 1.05 - 1.15
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real(157.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real(172.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 1.15 - 1.25
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real(172.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real(187.49)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Crane 1.25 - 
                ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Load", self_id),       Value::Real(187.50)),
                ("03.2",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Load", self_id),       Value::Real(200.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.05 - 0.15
                ("01.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("01.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(  7.50)),
                ("01.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("01.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("02.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("02.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 22.49)),
                ("02.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("02.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.15 - 0.25
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 22.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 37.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.25 - 0.35
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 37.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 52.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.35 - 0.45
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 52.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 67.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.45 - 0.55
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 67.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 82.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.55 - 0.65
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 82.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 97.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.65 - 0.75
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real( 97.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(112.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.75 - 0.85
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(112.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(127.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.85 - 0.95
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(127.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(142.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 0.95 - 1.05
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(142.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(157.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 1.05 - 1.15
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(157.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(172.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 1.15 - 1.25
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(172.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(187.49)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch1 1.25 - 
                ("03.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(187.50)),
                ("03.2",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch1.Load", self_id),       Value::Real(200.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch1.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.05 - 0.15
                ("01.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("01.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(  7.50)),
                ("01.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("01.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("02.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("02.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 22.49)),
                ("02.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("02.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.15 - 0.25
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 22.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 37.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.25 - 0.35
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 37.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 52.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.35 - 0.45
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 52.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 67.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.45 - 0.55
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 67.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 82.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.55 - 0.65
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 82.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 97.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.65 - 0.75
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real( 97.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(112.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.75 - 0.85
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(112.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(127.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.85 - 0.95
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(127.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(142.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 0.95 - 1.05
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(142.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(157.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 1.05 - 1.15
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(157.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(172.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 1.15 - 1.25
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(172.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(187.49)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch2 1.25 - 
                ("03.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(187.50)),
                ("03.2",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch2.Load", self_id),       Value::Real(200.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch2.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.05 - 0.15
                ("01.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("01.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(  7.50)),
                ("01.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("01.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("02.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("02.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 22.49)),
                ("02.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("02.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.15 - 0.25
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 22.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 37.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.25 - 0.35
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 37.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 52.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.35 - 0.45
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 52.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 67.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.45 - 0.55
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 67.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 82.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.55 - 0.65
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 82.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 97.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.65 - 0.75
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real( 97.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(112.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.75 - 0.85
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(112.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(127.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.85 - 0.95
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(127.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(142.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 0.95 - 1.05
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(142.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(157.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 1.05 - 1.15
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(157.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(172.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 1.15 - 1.25
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(172.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(187.49)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                //  Winch3 1.25 - 
                ("03.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(187.50)),
                ("03.2",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("03.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.0",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.1",    format!("/{}/Winch3.Load", self_id),       Value::Real(200.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("04.3",    format!("/{}/Winch3.Load", self_id),       Value::Real(  0.00)),
                ("128.0",    format!("/{}/Exit", self_id),       Value::String("exit".to_owned())),
            ]
    }
    ///
    /// Returns taget values
    fn targets<'a>() -> [&'a str; 208] {
        [
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_05-0_15-load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_05-0_15-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_05-0_15-load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_05-0_15-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_15-0_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_15-0_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_15-0_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_15-0_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_25-0_35-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_25-0_35-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_35-0_45_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_35-0_45-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_35-0_45_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_35-0_45-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_45-0_55_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_45-0_55-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_45-0_55_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_45-0_55-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_55-0_65_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_55-0_65-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_55-0_65_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_55-0_65-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_65-0_75_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_65-0_75-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_65-0_75_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_65-0_75-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_75-0_85_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_75-0_85-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_75-0_85_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_75-0_85-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_85-0_95_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_85-0_95-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_85-0_95_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_85-0_95-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_95-1_05_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_95-1_05-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_95-1_05_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_95-1_05-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_05-1_15_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_05-1_15-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_05-1_15_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_05-1_15-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_15-1_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_15-1_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_15-1_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_15-1_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_25-_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_25-_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_05-0_15-load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_05-0_15-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_05-0_15-load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_05-0_15-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_15-0_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_15-0_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_15-0_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_15-0_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_25-0_35-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_25-0_35-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_35-0_45_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_35-0_45-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_35-0_45_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_35-0_45-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_45-0_55_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_45-0_55-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_45-0_55_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_45-0_55-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_55-0_65_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_55-0_65-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_55-0_65_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_55-0_65-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_65-0_75_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_65-0_75-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_65-0_75_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_65-0_75-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_75-0_85_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_75-0_85-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_75-0_85_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_75-0_85-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_85-0_95_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_85-0_95-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_85-0_95_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_85-0_95-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_95-1_05_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_95-1_05-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_95-1_05_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_95-1_05-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_05-1_15_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_05-1_15-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_05-1_15_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_05-1_15-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_15-1_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_15-1_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_15-1_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_15-1_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_25-_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_25-_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_05-0_15-load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_05-0_15-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_05-0_15-load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_05-0_15-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_15-0_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_15-0_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_15-0_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_15-0_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_25-0_35-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_25-0_35-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_35-0_45_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_35-0_45-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_35-0_45_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_35-0_45-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_45-0_55_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_45-0_55-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_45-0_55_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_45-0_55-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_55-0_65_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_55-0_65-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_55-0_65_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_55-0_65-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_65-0_75_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_65-0_75-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_65-0_75_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_65-0_75-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_75-0_85_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_75-0_85-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_75-0_85_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_75-0_85-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_85-0_95_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_85-0_95-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_85-0_95_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_85-0_95-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_95-1_05_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_95-1_05-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_95-1_05_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_95-1_05-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_05-1_15_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_05-1_15-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_05-1_15_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_05-1_15-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_15-1_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_15-1_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_15-1_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_15-1_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_25-_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_25-_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_05-0_15-load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_05-0_15-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_05-0_15-load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_05-0_15-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_15-0_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_15-0_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_15-0_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_15-0_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_25-0_35-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_25-0_35-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_35-0_45_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_35-0_45-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_35-0_45_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_35-0_45-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_45-0_55_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_45-0_55-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_45-0_55_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_45-0_55-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_55-0_65_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_55-0_65-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_55-0_65_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_55-0_65-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_65-0_75_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_65-0_75-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_65-0_75_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_65-0_75-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_75-0_85_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_75-0_85-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_75-0_85_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_75-0_85-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_85-0_95_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_85-0_95-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_85-0_95_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_85-0_95-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_95-1_05_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_95-1_05-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_95-1_05_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_95-1_05-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_05-1_15_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_05-1_15-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_05-1_15_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_05-1_15-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_15-1_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_15-1_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_15-1_25_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_15-1_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_25-_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_25-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_25-_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_25-load';"),
        ]        
    }
}

