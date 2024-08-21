#[cfg(test)]

mod services {
    use std::{sync::Once, env};
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
    ///
    #[test]
    #[ignore = "To be implemented and activated later"]
    fn run() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        let self_id = "app_test";
        println!("\n{}", self_id);
        // let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        // test_duration.run().unwrap();
        let mut path = env::current_dir().unwrap();
        path.push("src/tests/unit/services/app/app.yaml");
        println!("working path: \n\t{:?}", env::current_dir().unwrap());
        let app = App::new(vec![path]);
        app.run().unwrap();
        println!();
        // assert!(points_count == target, "\nresult: {:?}\ntarget: {:?}", points_count, target);
        // test_duration.exit();
    }
}

