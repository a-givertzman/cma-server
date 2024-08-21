#[cfg(test)]

mod history {
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
    /// Testing history functionality
    #[test]
    #[ignore = "To be implemented later"]
    fn basic() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "history_test";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        // test_duration.run().unwrap();
        let mut path = env::current_dir().unwrap();
        path.push("src/tests/unit/services/history/history.yaml");
        println!("working path: \n\t{:?}", env::current_dir().unwrap());
        let app = App::new(vec![path]);
        app.run().unwrap();
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        test_duration.exit();
    }
}
