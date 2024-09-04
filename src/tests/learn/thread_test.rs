#[cfg(test)]

mod thread_test {
    use log::info;
    use testing::stuff::wait::WaitTread;
    use std::{sync::Once, thread, time::Duration};
    use debugging::session::debug_session::{DebugSession, LogLevel, Backtrace};
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
    fn init_each() {}
    ///
    /// If thread is already finished, join() or wait() don't returns error
    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn exiting() {
        DebugSession::init(LogLevel::Info, Backtrace::Short);
        init_once();
        init_each();
        let self_id = "thread_test";
        println!("{}", self_id);
        let handler = thread::spawn(move|| {
            info!("thread | Started");
            for i in 0..10 {
                info!("thread | iteration: {}", i);
            }
            info!("thread | Finished");
        });
        thread::sleep(Duration::from_millis(3000));
        handler.wait().unwrap();
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
}

