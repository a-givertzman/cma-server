#[cfg(test)]

use sal_sync::{services::{conf::{ConfTree, ServicesConf}, entity::Name, Service, Services}, thread_pool::ThreadPool};
use std::{env, sync::{Arc, Once}, time::Duration};
use testing::stuff::max_test_duration::TestDuration;
use debugging::session::debug_session::{DebugSession, LogLevel};
use crate::services::task::{Task, TaskConf};
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
fn points() {
    DebugSession::new().filter(LogLevel::Info).init();
    init_once();
    init_each();
    let dbg = "test Task.points";
    let self_name = Name::new("", dbg);
    println!("\n{}", dbg);
    let test_duration = TestDuration::new(dbg, Duration::from_secs(10));
    test_duration.run().unwrap();
    log::trace!("dir: {:?}", env::current_dir());
    let path = "./src/tests/unit/services/task/task_test_points.yaml";
    let config = TaskConf::read(&self_name, path);
    log::trace!("config: {:?}", &config);
    println!(" config points: {:?}", config.points());
    let tp = ThreadPool::new(dbg, Some(8));
    let services = Arc::new(Services::new(dbg, ServicesConf::new(
        dbg, 
        ConfTree::new_root(serde_yaml::from_str(r#"
            retain:
        "#).unwrap()),
    ), Some(tp.scheduler())));
    let task = Arc::new(Task::new(config, services.clone(), tp.scheduler()));
    services.insert(task.clone());
    let target  = 3;
    let points = task.points();
    let points_count = points.len();
    println!(" points count: {:?}", points_count);
    for point in points {
        println!("\t {:?}", point);
    }
    assert!(points_count == target, "\nresult: {:?}\ntarget: {:?}", points_count, target);
    test_duration.exit();
}

