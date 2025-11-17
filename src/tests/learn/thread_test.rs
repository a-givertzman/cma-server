#[cfg(test)]

mod thread_test {
    use std::{sync::Once, thread, time::Duration};
    use debugging::session::debug_session::{DebugSession, LogLevel};
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
        DebugSession::new().filter(LogLevel::Info).init();
        init_once();
        init_each();
        let self_id = "thread_test";
        println!("{}", self_id);
        let handler = thread::spawn(move|| {
            log::info!("thread | Started");
            for i in 0..10 {
                log::info!("thread | iteration: {}", i);
            }
            log::info!("thread | Finished");
        });
        thread::sleep(Duration::from_millis(3000));
        handler.join().unwrap();
        // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
    }
}

