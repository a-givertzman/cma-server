#[cfg(test)]

mod services_points {
    use sal_sync::services::{conf::{conf_tree::ConfTree, services_conf::ServicesConf}, entity::name::Name, safe_lock::rwlock::SafeLock, services::Services};
    use std::{env, sync::{Arc, Once, RwLock}, time::Duration};
    use testing::stuff::{max_test_duration::TestDuration, wait::WaitTread};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::{
        conf::task_config::TaskConfig,
        services::task::task::Task,
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
    /// Testing Services::points()
    #[test]
    fn services_points() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let self_id = "test Services.points";
        let self_name = Name::new("", self_id);
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        log::trace!("dir: {:?}", env::current_dir());
        let path = "./src/tests/unit/services/services/services_points_test.yaml";
        let config = TaskConfig::read(&self_name, path);
        log::trace!("config: {:?}", &config);
        println!(" points: {:?}", config.points());
        let services = Arc::new(RwLock::new(Services::new(self_id, ServicesConf::new(
            self_id, 
            ConfTree::new_root(serde_yaml::from_str(r#"
                retain:
                    path: assets/testing/retain/
                    point:
                        path: point/id.json
            "#).unwrap()),
        ))));
        let task = Arc::new(RwLock::new(Task::new(config, services.clone())));
        services.wlock(self_id).insert(task.clone());
        let services_handle = services.wlock(self_id).run().unwrap();
        let target  = 3;
        let points = services.rlock(self_id).points(self_id).then(|points| points, |err| {
            log::error!("{}.handle.Subscribe | Requesting points error: {:?}", self_id, err);
            vec![]
        });
        let points_count = points.len();
        println!(" points count: {:?}", points_count);
        for point in points {
            println!("\t {:?}", point);
        }
        assert!(points_count == target, "\nresult: {:?}\ntarget: {:?}", points_count, target);
        services.rlock(self_id).exit();
        services_handle.wait().unwrap();
        test_duration.exit();
    }
}

