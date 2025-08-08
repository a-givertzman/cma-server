#[cfg(test)]

use regex::Regex;
use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::Name, MultiQueue, MultiQueueConf, Service, Services}, thread_pool::ThreadPool};
use std::{env, fs, sync::{Arc, Once}, thread, time::{Duration, Instant}};
use testing::{entities::test_value::Value, stuff::max_test_duration::TestDuration};
use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
use crate::{
    services::{
        task::{Task, TaskConf, TaskTestReceiver}, ApiClient, ApiClientConf
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
/// Testing the Recorder | Basic metric - 'distribution by load' metric only (count & load per cycle)
#[test]
fn operating_metric_cycles_distribution_by_load_test() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg = "AppTest";
    let self_name = Name::new("", dbg);
    println!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(60));
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
        "#).unwrap()),
    ), Some(tp.scheduler())));
    let mut tasks = vec![];
    let path = "./src/tests/unit/services/task/cma_recorder/basic-metric-cycles-distribution-by-load.yaml";
    match fs::read_to_string(path) {
        Ok(yaml_string) => {
            match serde_yaml::from_str(&yaml_string) {
                Ok(config) => {
                    let config: serde_yaml::Value = config;
                    for (key, config) in config.as_mapping().unwrap() {
                        let mut conf = serde_yaml::Mapping::new();
                        conf.insert(key.clone(), config.clone());
                        let config = TaskConf::from_yaml(&self_name, &serde_yaml::Value::Mapping(conf));
                        let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
                        services.insert(task.clone());
                        tasks.push(task);
                    }
                }
                Err(err) => panic!("{}.read | Error in config: {:?}\n\terror: {:?}", dbg, yaml_string, err),
            }
        }
        Err(err) => panic!("{}.read | File {} reading error: {:?}", dbg, path, err),
    }
    let conf = MultiQueueConf::from_yaml(
        dbg,
        &serde_yaml::from_str(r"service MultiQueue:
            in queue in-queue:
                max-length: 10000
        ").unwrap(),
    );
    let multi_queue = Arc::new(MultiQueue::new(conf, services.clone(), Some(tp.scheduler())));
    services.insert(multi_queue.clone());
    let conf = ApiClientConf::from_yaml(
        dbg,
        &serde_yaml::from_str(r"service ApiClient:
            cycle: 100 ms
            reconnect: 1 s  # default 3 s
            address: 127.0.0.1:8080
            database: crane_data_server
            in queue in-queue:
                max-length: 10000
            auth-token: 123!@#
            debug: true
        ").unwrap(),
    );
    let api_client = Arc::new(ApiClient::new(conf, services.clone(), tp.scheduler()));
    services.insert(api_client.clone());
    let test_data = test_data(dbg);
    let total_count = test_data.len();
    let receiver = Arc::new(TaskTestReceiver::new(
        dbg,
        "",
        "in-queue",
        total_count * 1000,
    ));
    services.insert(receiver.clone());
    let test_data: Vec<(String, Value)> = test_data.into_iter().map(|(_, name, value)| {
        (name, value)
    }).collect();
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
    api_client.run().unwrap();
    receiver.run().unwrap();
    log::info!("receiver runing - ok");
    for task in &tasks {
        task.run().unwrap();
    }
    log::info!("task runing - ok");
    thread::sleep(Duration::from_millis(300));
    producer.run().unwrap();
    log::info!("producer runing - ok");
    thread::sleep(Duration::from_millis(300));
    let time = Instant::now();
    receiver.wait().unwrap();
    producer.exit();
    multi_queue.exit();
    for task in &tasks {
        task.exit();
    }
    for task in tasks {
        task.wait().unwrap();
    }
    services.exit();
    api_client.exit();
    api_client.wait().unwrap();
    producer.wait().unwrap();
    // exit_producer_handle.wait().unwrap();
    multi_queue.wait().unwrap();
    services.wait().unwrap();
    let sent = producer.sent_len();
    let result = receiver.received_len();
    println!(" elapsed: {:?}", time.elapsed());
    println!("    sent: {:?}", sent);
    println!("received: {:?}", result);
    for (i, result) in receiver.received().iter().enumerate() {
        println!("received: {}\t|\t{}\t|\t{:?}", i, result.name(), result.value());
    };
    let targets = targets();
    let mut index = 0;
    for result in receiver.received() {
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

