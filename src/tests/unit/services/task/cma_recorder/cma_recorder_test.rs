#[cfg(test)]

mod cma_recorder {
    use log::{info, trace};
    use regex::Regex;
    use sal_sync::services::{entity::name::Name, retain::retain_conf::RetainConf, service::service::Service};
    use std::{env, fs, sync::{Arc, Once, RwLock}, thread, time::{Duration, Instant}};
    use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, wait::WaitTread}};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::{api_client_config::ApiClientConfig, multi_queue_config::MultiQueueConfig, task_config::TaskConfig},
        services::{
            api_cient::api_client::ApiClient, multi_queue::multi_queue::MultiQueue, safe_lock::rwlock::SafeLock, services::Services,
            task::{task::Task, task_test_receiver::TaskTestReceiver},
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
    /// returns:
    ///  - ...
    fn init_each() -> () {}
    ///
    /// Testing the complette configuration of the CMA Recorder
    #[ignore = "Manual test"]
    #[test]
    fn test() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
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
        let services = Arc::new(RwLock::new(Services::new(self_id, RetainConf::new(None::<&str>, None))));
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
                            let task = Arc::new(RwLock::new(Task::new(config, services.clone())));
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
        let multi_queue = Arc::new(RwLock::new(MultiQueue::new(conf, services.clone())));
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
        let api_client = Arc::new(RwLock::new(ApiClient::new(conf)));
        services.wlock(self_id).insert(api_client.clone());
        let test_data = test_data(self_id);
        let total_count = test_data.len();
        let receiver = Arc::new(RwLock::new(TaskTestReceiver::new(
            self_id,
            "",
            "in-queue",
            total_count * 1000,
        )));
        services.wlock(self_id).insert(receiver.clone());
        let test_data: Vec<(String, Value)> = test_data.into_iter().map(|(_, name, value)| {
            (name, value)
        }).collect();
        let producer = Arc::new(RwLock::new(TaskTestProducer::new(
            self_id,
            &format!("/{}/MultiQueue.in-queue", self_id),
            Duration::from_millis(10),
            services.clone(),
            &test_data,
        )));
        services.wlock(self_id).insert(producer.clone());
        thread::sleep(Duration::from_millis(100));
        let services_handle = services.wlock(self_id).run().unwrap();
        let multi_queue_handle = multi_queue.write().unwrap().run().unwrap();
        let api_client_handle = api_client.write().unwrap().run().unwrap();
        let receiver_handle = receiver.write().unwrap().run().unwrap();
        info!("receiver runing - ok");
        for task in &tasks {
            let handle = task.write().unwrap().run().unwrap();
            task_handles.push(handle);
        }
        info!("task runing - ok");
        thread::sleep(Duration::from_millis(600));
        let producer_handle = producer.write().unwrap().run().unwrap();
        info!("producer runing - ok");
        thread::sleep(Duration::from_millis(900));
        let time = Instant::now();
        receiver_handle.wait().unwrap();
        producer.read().unwrap().exit();
        multi_queue.read().unwrap().exit();
        for task in tasks {
            task.read().unwrap().exit();
        }
        services.rlock(self_id).exit();
        for handle in task_handles {
            handle.wait().unwrap();
        }
        api_client.read().unwrap().exit();
        api_client_handle.wait().unwrap();
        producer_handle.wait().unwrap();
        multi_queue_handle.wait().unwrap();
        services_handle.wait().unwrap();
        let sent = producer.read().unwrap().sent().read().unwrap().len();
        let result = receiver.read().unwrap().received().read().unwrap().len();
        println!(" elapsed: {:?}", time.elapsed());
        println!("    sent: {:?}", sent);
        println!("received: {:?}\n", result);
        for (i, result) in receiver.read().unwrap().received().read().unwrap().iter().enumerate() {
            println!("received: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
        };
        let targets = targets();
        let mut index = 0;
        for result in receiver.read().unwrap().received().read().unwrap().iter() {
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
        // assert!(index == targets.len(), "result: {:?}\ntarget: {:?}", index, targets.len());
        test_duration.exit();
        loop {
            thread::sleep(Duration::from_millis(100));
        }
    }
    ///
    /// Returns test data
    fn test_data<'a>(self_id: &str) -> Vec<(&'a str, String, Value)> {
        vec![
            //  step        nape                                input
            ("00.-5",    format!("/{}/Load.Nom", self_id),   Value::Real(  150.00)),
            ("00.-3",    format!("/{}/Winch1.Load.Nom", self_id),   Value::Real(  150.00)),
            ("00.-2",    format!("/{}/Winch2.Load.Nom", self_id),   Value::Real(  150.00)),
            ("00.-1",    format!("/{}/Winch3.Load.Nom", self_id),   Value::Real(  150.00)),
            ("00.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("01.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("02.0",    format!("/{}/Load", self_id),       Value::Real(  3.30)),
            ("03.0",    format!("/{}/Load", self_id),       Value::Real(  0.10)),
            ("04.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("05.0",    format!("/{}/Load", self_id),       Value::Real(  1.60)),
            ("06.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("07.0",    format!("/{}/Load", self_id),       Value::Real(  7.20)),
            ("08.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("09.0",    format!("/{}/Load", self_id),       Value::Real(  0.30)),
            ("10.0",    format!("/{}/Load", self_id),       Value::Real(  2.20)),
            ("11.0",    format!("/{}/Load", self_id),       Value::Real(  8.10)),
            ("12.0",    format!("/{}/Load", self_id),       Value::Real(  1.90)),
            ("13.0",    format!("/{}/Load", self_id),       Value::Real(  0.10)),
            ("14.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("15.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("16.0",    format!("/{}/Load", self_id),       Value::Real(  5.00)),
            ("17.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("17.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("17.2",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("17.3",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("17.4",    format!("/{}/Winch2.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("17.5",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("17.6",    format!("/{}/Winch3.Load.Limiter.Trip", self_id),       Value::Bool(false)),
            ("18.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("19.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("20.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("21.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("22.0",    format!("/{}/Load", self_id),       Value::Real(  6.00)),
            ("23.0",    format!("/{}/Load", self_id),       Value::Real( 12.00)),
            ("24.0",    format!("/{}/Load", self_id),       Value::Real( 64.00)),
            ("26.1",    format!("/{}/CraneMode.MOPS", self_id),       Value::Int(1)),
            ("26.1",    format!("/{}/CraneMode.MOPS", self_id),       Value::Int(0)),
            ("25.0",    format!("/{}/Load", self_id),       Value::Real(128.00)),
            ("26.0",    format!("/{}/Load", self_id),       Value::Real(120.00)),
            ("26.1",    format!("/{}/CraneMode.AOPS", self_id),       Value::Int(1)),
            ("27.0",    format!("/{}/Load", self_id),       Value::Real(133.00)),
            ("28.0",    format!("/{}/Load", self_id),       Value::Real(121.00)),
            // ("28.1",    format!("/{}/Load", self_id),       Value::Real(141.00)),
            ("29.0",    format!("/{}/Load", self_id),       Value::Real(130.00)),
            ("30.0",    format!("/{}/Load", self_id),       Value::Real(127.00)),
            ("31.0",    format!("/{}/Load", self_id),       Value::Real(123.00)),
            ("32.0",    format!("/{}/Load", self_id),       Value::Real(122.00)),
            ("33.0",    format!("/{}/Load", self_id),       Value::Real(120.00)),
            ("34.0",    format!("/{}/Load", self_id),       Value::Real( 64.00)),
            ("35.0",    format!("/{}/Load", self_id),       Value::Real( 32.00)),
            ("36.0",    format!("/{}/Load", self_id),       Value::Real( 24.00)),
            ("37.0",    format!("/{}/Load", self_id),       Value::Real( 12.00)),
            ("38.0",    format!("/{}/Load", self_id),       Value::Real(  8.00)),
            ("39.0",    format!("/{}/Load", self_id),       Value::Real( 17.00)),
            ("26.1",    format!("/{}/CraneMode.MOPS", self_id),       Value::Int(1)),
            ("40.0",    format!("/{}/Load", self_id),       Value::Real( 10.00)),
            ("41.0",    format!("/{}/Load", self_id),       Value::Real(  7.00)),
            ("42.0",    format!("/{}/Load", self_id),       Value::Real(  3.00)),
            ("43.0",    format!("/{}/Load", self_id),       Value::Real(  6.00)),
            ("44.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("45.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("46.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("47.0",    format!("/{}/Load", self_id),       Value::Real(  4.00)),
            ("47.1",    format!("/{}/Winch1.Load.Limiter.Trip", self_id),       Value::Bool(true)),
            ("48.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("49.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("50.0",    format!("/{}/Load", self_id),       Value::Real(  3.00)),
            ("51.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("52.0",    format!("/{}/Load", self_id),       Value::Real(  2.00)),
            ("53.0",    format!("/{}/Load", self_id),       Value::Real(  1.00)),
            ("54.0",    format!("/{}/Load", self_id),       Value::Real(  0.70)),
            ("55.0",    format!("/{}/Load", self_id),       Value::Real(  0.80)),
            ("56.0",    format!("/{}/Load", self_id),       Value::Real(  0.40)),
            ("57.0",    format!("/{}/Load", self_id),       Value::Real(  0.30)),
            ("58.0",    format!("/{}/Load", self_id),       Value::Real(  0.20)),
            ("59.0",    format!("/{}/Load", self_id),       Value::Real(  0.10)),
            ("60.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("61.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("62.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("63.0",    format!("/{}/Load", self_id),       Value::Real(  0.00)),
            ("128.0",    format!("/{}/Exit", self_id),       Value::String("exit".to_owned())),
        ]
    }
    ///
    /// Returns taget values
    fn targets<'a>() -> [&'a str; 4] {
        [
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_05-0_15-load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_05-0_15-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_05-0_15-load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_05-0_15-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_15-0_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_15-0_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_15-0_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_15-0_25-load';"),

            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_25-0_35-load';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_25-0_35_load-range';"),
            (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_25-0_35-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_35-0_45_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_35-0_45-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_35-0_45_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_35-0_45-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_45-0_55_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_45-0_55-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_45-0_55_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_45-0_55-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_55-0_65_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_55-0_65-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_55-0_65_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_55-0_65-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_65-0_75_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_65-0_75-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_65-0_75_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_65-0_75-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_75-0_85_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_75-0_85-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_75-0_85_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_75-0_85-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_85-0_95_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_85-0_95-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_85-0_95_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_85-0_95-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_95-1_05_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_95-1_05-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-0_95-1_05_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '0_95-1_05-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_05-1_15_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_05-1_15-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_05-1_15_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_05-1_15-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_15-1_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_15-1_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_15-1_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_15-1_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_25-_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'cycles-1_25-_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = '1_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_05-0_15-load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_05-0_15-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_05-0_15-load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_05-0_15-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_15-0_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_15-0_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_15-0_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_15-0_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_25-0_35_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_25-0_35-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_25-0_35_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_25-0_35-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_35-0_45_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_35-0_45-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_35-0_45_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_35-0_45-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_45-0_55_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_45-0_55-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_45-0_55_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_45-0_55-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_55-0_65_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_55-0_65-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_55-0_65_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_55-0_65-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_65-0_75_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_65-0_75-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_65-0_75_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_65-0_75-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_75-0_85_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_75-0_85-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_75-0_85_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_75-0_85-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_85-0_95_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_85-0_95-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_85-0_95_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_85-0_95-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_95-1_05_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_95-1_05-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-0_95-1_05_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-0_95-1_05-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_05-1_15_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_05-1_15-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_05-1_15_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_05-1_15-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_15-1_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_15-1_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_15-1_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_15-1_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_25-_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-cycles-1_25-_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch1-1_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_05-0_15-load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_05-0_15-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_05-0_15-load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_05-0_15-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_15-0_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_15-0_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_15-0_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_15-0_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_25-0_35_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_25-0_35-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_25-0_35_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_25-0_35-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_35-0_45_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_35-0_45-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_35-0_45_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_35-0_45-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_45-0_55_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_45-0_55-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_45-0_55_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_45-0_55-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_55-0_65_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_55-0_65-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_55-0_65_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_55-0_65-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_65-0_75_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_65-0_75-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_65-0_75_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_65-0_75-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_75-0_85_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_75-0_85-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_75-0_85_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_75-0_85-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_85-0_95_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_85-0_95-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_85-0_95_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_85-0_95-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_95-1_05_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_95-1_05-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-0_95-1_05_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-0_95-1_05-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_05-1_15_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_05-1_15-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_05-1_15_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_05-1_15-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_15-1_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_15-1_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_15-1_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_15-1_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_25-_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-cycles-1_25-_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch2-1_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_05-0_15-load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_05-0_15-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_05-0_15-load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_05-0_15-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_15-0_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_15-0_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_15-0_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_15-0_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_25-0_35_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_25-0_35-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_25-0_35_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_25-0_35-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_35-0_45_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_35-0_45-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_35-0_45_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_35-0_45-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_45-0_55_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_45-0_55-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_45-0_55_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_45-0_55-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_55-0_65_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_55-0_65-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_55-0_65_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_55-0_65-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_65-0_75_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_65-0_75-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_65-0_75_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_65-0_75-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_75-0_85_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_75-0_85-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_75-0_85_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_75-0_85-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_85-0_95_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_85-0_95-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_85-0_95_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_85-0_95-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_95-1_05_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_95-1_05-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-0_95-1_05_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-0_95-1_05-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_05-1_15_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_05-1_15-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_05-1_15_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_05-1_15-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_15-1_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_15-1_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_15-1_25_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_15-1_25-load';"),

            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_25-_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_25-load';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-cycles-1_25-_load-range';"),
            // (r"update public\.basic_metric set value = \d+(?:\.\d+)* where name = 'winch3-1_25-load';"),
        ]        
    }
}

