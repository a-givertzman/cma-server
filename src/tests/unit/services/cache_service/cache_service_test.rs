#[cfg(test)]

mod cache_service {
    use std::{sync::Once, env, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
    use crate::services::app::app::App;
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
    /// Testing cache_service functionality
    #[test]
    #[ignore = "To be implemented later"]
    fn basic() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "cache_service_test";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        // test_duration.run().unwrap();
        let mut path = env::current_dir().unwrap();
        path.push("src/tests/unit/services/cache_service/cache_service.yaml");
        println!("working path: \n\t{:?}", env::current_dir().unwrap());
        let app = App::new(vec![path]);
        app.run().unwrap();
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        test_duration.exit();
    }
}
