#[cfg(test)]

mod task {
    use log::trace;
    use sal_sync::services::{entity::name::Name, service::service::Service};
    use std::{env, sync::{Arc, Mutex, Once, RwLock}, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::task_config::TaskConfig,
        services::{safe_lock::SafeLock, services::Services, task::task::Task},
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
    fn points() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "test Task.points";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        trace!("dir: {:?}", env::current_dir());
        let path = "./src/tests/unit/services/task/task_test_points.yaml";
        let config = TaskConfig::read(&self_name, path);
        trace!("config: {:?}", &config);
        println!(" config points: {:?}", config.points());
        let services = Arc::new(RwLock::new(Services::new(self_id)));
        let task = Arc::new(Mutex::new(Task::new(config, services.clone())));
        services.wlock(self_id).insert(task.clone());
        let target  = 3;
        let points = task.lock().unwrap().points();
        let points_count = points.len();
        println!();
        println!(" points count: {:?}", points_count);
        for point in points {
            println!("\t {:?}", point);
        }
        assert!(points_count == target, "\nresult: {:?}\ntarget: {:?}", points_count, target);
        test_duration.exit();
    }
}

