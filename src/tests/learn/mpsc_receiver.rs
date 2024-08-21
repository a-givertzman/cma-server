#![allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use log::info;
    use std::sync::{Once, mpsc};
    use debugging ::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::core_::constants::constants::RECV_TIMEOUT; 
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
    /// Testing mpsc receiver behavior on recv_timeout method
    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn test_mpsc_receiver() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test mpsc::Receiver");
        let (send, recv) = mpsc::channel();
        let iterations = 10000;
        for value in 0..=iterations {
            send.send(value).unwrap();
        }
        drop(send);
        let mut value = -1;
        while value < iterations {
            value = recv.recv_timeout(RECV_TIMEOUT).unwrap();
            info!("value: {}", value);
            // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
    }
}
