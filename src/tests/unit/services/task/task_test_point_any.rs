#[cfg(test)]

use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::Name, Service, Services}, thread_pool::ThreadPool};
use std::{sync::{Arc, Once}, thread, time::{Duration, Instant}};
use testing::{entities::test_value::Value, stuff::{max_test_duration::TestDuration, random_test_values::RandomTestValues}};
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::services::task::{Task, TaskConf, TaskTestProducer, TaskTestReceiver};
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
fn point_any_structure() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let dbg = "task_test_point_any";
    let self_name = Name::new("", dbg);
    println!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(3));
    test_duration.run().unwrap();
    //
    // can be changed
    let iterations = 10;
    let conf = serde_yaml::from_str(&format!(r#"
        service Task TaskAny:
            cycle: 1 ms
            in queue in-queue:
                max-length: 10000
            fn ToApiQueue:
                queue: {}/TaskTestReceiver.in-queue
                input fn SqlMetric:
                    initial: 0.123      # начальное значение
                    table: table_name
                    sql: "insert into {{table}} (id, value, timestamp) values ({{id}}, {{input1.value}}, {{input1.value}});"
                    input1: point any every
    "#, self_name)).unwrap();
    let config = TaskConf::from_yaml(&self_name, &conf);
    log::trace!("config: {:?}", &config);
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), Some(tp.scheduler())));
    let receiver = Arc::new(TaskTestReceiver::new(
        &self_name.join(),
        "",
        "in-queue",
        iterations,
    ));
    services.insert(receiver.clone());
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
        &self_name.join(),
        &Name::new(self_name, "TaskAny.in-queue").join(),
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
    thread::sleep(Duration::from_millis(100));
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
    let sent = producer.sent().len();
    let result = receiver.received().len();
    println!(" elapsed: {:?}", time.elapsed());
    println!("    sent: {:?}", sent);
    println!("received: {:?}", result);
    assert!(sent == iterations, "\nresult: {:?}\ntarget: {:?}", sent, iterations);
    assert!(result == iterations, "\nresult: {:?}\ntarget: {:?}", result, iterations);
    test_duration.exit();
}

