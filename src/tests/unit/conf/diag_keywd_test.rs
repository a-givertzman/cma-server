#[cfg(test)]

mod diag_keywd {
    use strum::IntoEnumIterator;
    use std::{sync::Once, time::Duration};
    use testing::stuff::max_test_duration::TestDuration;
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};

    use crate::conf::diag_keywd::DiagKeywd;
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
    /// Testing from_str
    #[test]
    fn variants() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "diag_keywd_values_test";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            DiagKeywd::Status,
            DiagKeywd::Connection,
        ];
        for kewd in DiagKeywd::iter() {
            let result = test_data.contains(&kewd);
            assert!(result == true, "DiagKeywd variants was extended \nresult: {:?}\ntarget: {:?}", result, true);
        }
        test_duration.exit();
    }
    ///
    /// Testing from_str
    #[test]
    fn from_str() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        println!();
        let self_id = "diag_keywd_from_str_test";
        println!("\n{}", self_id);
        let test_duration = TestDuration::new(self_id, Duration::from_secs(10));
        test_duration.run().unwrap();
        let test_data = [
            ("Status".to_owned(), DiagKeywd::Status),
            ("Connection".to_owned(), DiagKeywd::Connection),
            ("/App/Service/Status".to_owned(), DiagKeywd::Status),
            ("/App/Service/Connection".to_owned(), DiagKeywd::Connection),
            ("/App/Service/Some.Status".to_owned(), DiagKeywd::Status),
            ("/App/Service/Some.Connection".to_owned(), DiagKeywd::Connection),
        ]
        .into_iter()
        .chain(
            DiagKeywd::iter().map(|kewd| (kewd.as_str().to_owned(), kewd)),
        );
        for (value, target) in test_data {
            let result = DiagKeywd::new(&value);
            assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        test_duration.exit();
    }
}
