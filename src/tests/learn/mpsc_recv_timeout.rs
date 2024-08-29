#[cfg(test)]
mod tests {
    use log::{info, error};
    use testing::stuff::wait::WaitTread;
    use std::{sync::{Once, mpsc::{self, RecvTimeoutError}}, time::Duration, thread::{self}};
    use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
    use crate::core_::constants::constants::RECV_TIMEOUT;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    // use super::*;

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
    fn init_each() -> () {

    }

    #[ignore = "Learn - all must be ignored"]
    #[test]
    fn test_mpsc_receiver() {
        DebugSession::init(LogLevel::Debug, Backtrace::Short);
        init_once();
        init_each();
        println!();
        println!("test mpsc::Receiver");
        let self_id = "test";
        let (send, recv) = mpsc::channel();
        let iterations = 3;
        let _h = thread::Builder::new().name(self_id.to_string()).spawn(move || {
            for value in 0..=iterations {
                send.send(value).unwrap();
                thread::sleep(Duration::from_millis(1000))
            }
            drop(send);
        }).unwrap();

        let mut exit = false;
        while !exit {
            match recv.recv_timeout(RECV_TIMEOUT) {
                Ok(value) => {
                    info!("value: {}", value);
                }
                Err(err) => {
                    match err {
                        RecvTimeoutError::Timeout => {
                            error!("debug: {}", err);
                        }
                        RecvTimeoutError::Disconnected => {
                            error!("error: {}", err);
                            thread::sleep(Duration::from_millis(1000));
                            exit = true;
                        }
                    }
                }
            };
            // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
        }
        _h.wait().unwrap();
    }

}
